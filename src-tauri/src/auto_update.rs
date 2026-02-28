use crate::AppState;
use crate::update_cycle;
use chrono::{Duration as ChronoDuration, Local, TimeZone, Timelike};
use log::{error, info, warn};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// 启动自动更新任务（响应设置变更，可取消）
pub(crate) fn start_auto_update_task(app: AppHandle) {
    let state = app.state::<AppState>();
    let mut rx = state.settings_rx.clone();

    // 如已有旧任务，先取消（不需要获取 runtime handle）
    tauri::async_runtime::block_on(async {
        let mut h = state.auto_update_handle.lock().await;
        h.abort();
        let app_clone = app.clone();
        let new_handle = tauri::async_runtime::spawn(async move {
            // 初始立即执行一次更新（强制更新，确保首次启动时能获取数据）
            // 检查索引是否为空，如果为空则强制更新
            update_cycle::check_and_trigger_update_if_needed(&app_clone).await;

            // 标记是否是第一次收到设置变更（启动时的初始化不算）
            let mut is_first_change = true;

            // 小时循环 + 零点对齐
            loop {
                // 计算距下一次本地零点（含 5 分钟缓冲）剩余时间
                let now = Local::now();
                // 安全处理日期计算，提供 fallback 避免 panic
                let tomorrow = now.date_naive().succ_opt().unwrap_or_else(|| {
                    warn!(target: "auto_update", "日期计算失败，使用默认值（明天）");
                    now.date_naive() + ChronoDuration::days(1)
                });
                let naive_next = tomorrow.and_hms_opt(0, 5, 0).unwrap_or_else(|| {
                    warn!(target: "auto_update", "时间创建失败，使用默认值（00:00:00）");
                    tomorrow.and_hms_opt(0, 0, 0).unwrap_or_else(|| {
                        warn!(target: "auto_update", "无法创建默认时间，使用当前日期时间");
                        now.naive_local()
                    })
                });
                let next_midnight = Local
                    .from_local_datetime(&naive_next)
                    .single()
                    .unwrap_or_else(|| {
                        warn!(target: "auto_update", "时区转换失败，使用首个匹配时间");
                        Local
                            .from_local_datetime(&naive_next)
                            .earliest()
                            .unwrap_or_else(|| {
                                warn!(target: "auto_update", "无法创建本地时间，使用当前时间 + 1小时");
                                now + ChronoDuration::hours(1)
                            })
                    });
                let until_midnight = next_midnight - now;

                // 每小时轮询，若距零点不足 1 小时则缩短睡眠以对齐零点
                let sleep_dur = if let Ok(rem) = until_midnight.to_std() {
                    if rem <= Duration::from_secs(3600) {
                        rem
                    } else {
                        Duration::from_secs(3600)
                    }
                } else {
                    Duration::from_secs(3600)
                };

                tokio::select! {
                    _ = tokio::time::sleep(sleep_dur) => {
                        let after_sleep_now = Local::now();
                        // 零点窗口（00:00~00:05）内执行每日对齐更新，并在失败时快速重试
                        if after_sleep_now.hour() == 0 && after_sleep_now.minute() <= 5 {
                            // 记录更新前的日期
                            update_cycle::run_update_cycle(&app_clone).await;
                            let today = after_sleep_now.date_naive();
                            // 判断是否成功（last_update_time 是否被更新为今日）
                            let mut need_retry = {
                                let state_ref = app_clone.state::<AppState>();
                                let guard = state_ref.last_update_time.lock().await;
                                guard.map(|dt| dt.date_naive()) != Some(today)
                            };
                            if need_retry {
                                warn!(target:"auto_update","零点窗口初次更新可能失败，开始指数退避重试");
                                // 优化：改进的指数退避重试策略，限制最大延迟
                                const MAX_MIDNIGHT_RETRIES: u32 = 10;
                                const MAX_BACKOFF_SECS: u64 = 60; // 最大延迟 60 秒
                                for attempt in 0..MAX_MIDNIGHT_RETRIES {
                                    // 优化：限制最大延迟时间，避免等待时间过长
                                    let base_backoff = 1 << attempt; // 指数退避：1, 2, 4, 8, 16, 32, 64, 128, 256, 512
                                    let backoff = base_backoff.min(MAX_BACKOFF_SECS); // 限制最大 60 秒
                                    warn!(target:"auto_update","零点重试第 {} 次，{}s 后执行", attempt + 1, backoff);
                                    tokio::time::sleep(Duration::from_secs(backoff)).await;

                                    update_cycle::run_update_cycle(&app_clone).await;
                                    let now_retry = Local::now();
                                    let after_cycle_success = {
                                        let state_ref = app_clone.state::<AppState>();
                                        let guard = state_ref.last_update_time.lock().await;
                                        guard.map(|dt| dt.date_naive()) == Some(now_retry.date_naive())
                                    };
                                    if after_cycle_success {
                                        info!(target:"auto_update","零点重试第 {} 次成功", attempt + 1);
                                        need_retry = false;
                                        break;
                                    } else {
                                        warn!(target:"auto_update","零点重试第 {} 次仍未获取到当日壁纸", attempt + 1);
                                    }
                                }
                                if need_retry {
                                    warn!(target:"auto_update","零点重试结束，仍未成功获取当日壁纸，等待下一轮小时轮询");
                                }
                            }
                        } else {
                            // 普通每小时轮询
                            update_cycle::run_update_cycle(&app_clone).await;
                        }
                    }
                    changed = rx.changed() => {
                        if changed.is_err() {
                            error!(target: "update", "settings watch channel closed");
                            break;
                        }

                        // 跳过第一次设置变更（启动时的初始化）
                        if is_first_change {
                            is_first_change = false;
                            continue;
                        }

                        let latest = rx.borrow().clone();
                        if !latest.auto_update {
                            info!(target: "update", "自动应用已关闭（仍会获取新壁纸），等待重新开启...");
                            loop {
                                if rx.changed().await.is_err() { break; }
                                let s = rx.borrow().clone();
                                if s.auto_update {
                                    info!(target: "update", "自动应用重新开启，立即执行一次");
                                    update_cycle::run_update_cycle(&app_clone).await;
                                    break;
                                }
                            }
                        } else {
                            info!(target: "update", "设置改变，立即执行更新");
                            update_cycle::run_update_cycle(&app_clone).await;
                        }
                    }
                }
            }
        });
        *h = new_handle;
    });
}
