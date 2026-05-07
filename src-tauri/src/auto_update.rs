use crate::AppState;
use crate::update_cycle;
use chrono::{Duration as ChronoDuration, Local, TimeZone, Timelike};
use log::{error, info, warn};
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// 一小时（秒）。
const HOUR_SECS: u64 = 3600;

/// 计算下一次自动更新循环之前的睡眠时长。
///
/// 普通模式：每小时一次，距零点 ≤ 1h 时缩短以对齐零点。
///
/// 追赶模式（`needs_catchup = true`）：当日壁纸尚未成功获取时，按
/// "连续失败次数"走分级退避档位，比正常间隔更短：
/// - 前 3 次失败：每 15 分钟一次，捕捉短暂的网络恢复窗口；
/// - 第 4-6 次失败：每 30 分钟一次；
/// - 之后：回退到 60 分钟，与普通整点巡询一致，避免无限制写日志。
///
/// 抽出为纯函数以便单元测试覆盖各档位逻辑。
fn compute_sleep_duration(
    until_midnight: ChronoDuration,
    needs_catchup: bool,
    consecutive_today_failures: u32,
) -> Duration {
    let normal = if let Ok(rem) = until_midnight.to_std() {
        if rem <= Duration::from_secs(HOUR_SECS) {
            rem
        } else {
            Duration::from_secs(HOUR_SECS)
        }
    } else {
        Duration::from_secs(HOUR_SECS)
    };

    if !needs_catchup {
        return normal;
    }

    let catchup_secs: u64 = match consecutive_today_failures {
        0..=2 => 15 * 60,
        3..=5 => 30 * 60,
        _ => HOUR_SECS,
    };
    normal.min(Duration::from_secs(catchup_secs))
}

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
            // 当日壁纸尚未获取成功时的连续失败次数（追赶模式退避档位用）
            let mut consecutive_today_failures: u32 = 0;

            // 小时循环 + 零点对齐 + 失败追赶
            loop {
                // 计算距下一次本地零点（含 5 分钟缓冲）剩余时间
                let now = Local::now();
                let today = now.date_naive();
                // 安全处理日期计算，提供 fallback 避免 panic
                let tomorrow = today.succ_opt().unwrap_or_else(|| {
                    warn!(target: "auto_update", "日期计算失败，使用默认值（明天）");
                    today + ChronoDuration::days(1)
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

                // 检查"今日壁纸是否已成功获取"
                let needs_catchup = {
                    let state_ref = app_clone.state::<AppState>();
                    let guard = state_ref.last_update_time.lock().await;
                    guard.map(|dt| dt.date_naive()) != Some(today)
                };
                if !needs_catchup {
                    consecutive_today_failures = 0;
                }

                let sleep_dur = compute_sleep_duration(
                    until_midnight,
                    needs_catchup,
                    consecutive_today_failures,
                );

                if needs_catchup {
                    info!(
                        target: "auto_update",
                        "今日壁纸尚未获取成功（连续失败 {} 次），追赶模式：{}s 后重试",
                        consecutive_today_failures,
                        sleep_dur.as_secs()
                    );
                }

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
                                    warn!(target:"auto_update","零点重试结束，仍未成功获取当日壁纸，进入追赶模式等待下一轮重试");
                                }
                            }
                        } else {
                            // 普通每小时轮询 / 追赶模式重试
                            update_cycle::run_update_cycle(&app_clone).await;
                        }

                        // 统一更新追赶计数：cycle 完成后检查今日是否成功
                        let cycle_today = Local::now().date_naive();
                        let success_today = {
                            let state_ref = app_clone.state::<AppState>();
                            let guard = state_ref.last_update_time.lock().await;
                            guard.map(|dt| dt.date_naive()) == Some(cycle_today)
                        };
                        if success_today {
                            consecutive_today_failures = 0;
                        } else {
                            consecutive_today_failures =
                                consecutive_today_failures.saturating_add(1);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_mode_uses_full_hour_when_far_from_midnight() {
        // 距零点 5 小时，正常模式应当 sleep 1 小时
        let dur = compute_sleep_duration(ChronoDuration::hours(5), false, 0);
        assert_eq!(dur, Duration::from_secs(HOUR_SECS));
    }

    #[test]
    fn normal_mode_aligns_to_midnight_when_close() {
        // 距零点 30 分钟，正常模式应当 sleep 30 分钟（对齐零点）
        let dur = compute_sleep_duration(ChronoDuration::minutes(30), false, 0);
        assert_eq!(dur, Duration::from_secs(30 * 60));
    }

    #[test]
    fn normal_mode_handles_negative_duration() {
        // 时钟回拨等异常：fallback 到 1 小时
        let dur = compute_sleep_duration(ChronoDuration::seconds(-100), false, 0);
        assert_eq!(dur, Duration::from_secs(HOUR_SECS));
    }

    #[test]
    fn catchup_first_three_failures_use_15_minutes() {
        for failures in [0u32, 1, 2] {
            let dur = compute_sleep_duration(ChronoDuration::hours(5), true, failures);
            assert_eq!(
                dur,
                Duration::from_secs(15 * 60),
                "failures={failures} 应当 15 分钟"
            );
        }
    }

    #[test]
    fn catchup_mid_failures_use_30_minutes() {
        for failures in [3u32, 4, 5] {
            let dur = compute_sleep_duration(ChronoDuration::hours(5), true, failures);
            assert_eq!(
                dur,
                Duration::from_secs(30 * 60),
                "failures={failures} 应当 30 分钟"
            );
        }
    }

    #[test]
    fn catchup_long_failures_fall_back_to_hour() {
        for failures in [6u32, 7, 100, u32::MAX] {
            let dur = compute_sleep_duration(ChronoDuration::hours(5), true, failures);
            assert_eq!(
                dur,
                Duration::from_secs(HOUR_SECS),
                "failures={failures} 应当 60 分钟"
            );
        }
    }

    #[test]
    fn catchup_never_exceeds_until_midnight() {
        // 距零点仅 5 分钟，即使追赶模式想 sleep 15 分钟，也应缩短到 5 分钟以对齐零点
        let dur = compute_sleep_duration(ChronoDuration::minutes(5), true, 0);
        assert_eq!(dur, Duration::from_secs(5 * 60));
    }
}
