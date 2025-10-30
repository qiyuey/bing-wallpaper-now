//! 运行时状态持久化模块
//!
//! 使用 tauri-plugin-store 管理应用运行时状态的持久化存储
//! 与用户设置 (settings.json) 分离，存储在隐藏文件 .runtime.json 中

use crate::models::AppRuntimeState;
use anyhow::Result;
use chrono::Local;
use std::path::Path;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const RUNTIME_STATE_KEY: &str = "runtime_state";
const RUNTIME_STORE_FILE: &str = ".runtime.json";

/// 从 store 加载运行时状态
pub fn load_runtime_state(app: &AppHandle) -> Result<AppRuntimeState> {
    let store = app
        .store(RUNTIME_STORE_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to access runtime store: {}", e))?;

    match store.get(RUNTIME_STATE_KEY) {
        Some(value) => {
            let state: AppRuntimeState = serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("Failed to deserialize runtime state: {}", e))?;

            Ok(state)
        }
        None => Ok(AppRuntimeState::default()),
    }
}

/// 保存运行时状态
pub fn save_runtime_state(app: &AppHandle, state: &AppRuntimeState) -> Result<()> {
    let store = app
        .store(RUNTIME_STORE_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to access runtime store: {}", e))?;

    let value = serde_json::to_value(state)
        .map_err(|e| anyhow::anyhow!("Failed to serialize runtime state: {}", e))?;

    store.set(RUNTIME_STATE_KEY, value);

    store
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save runtime store to disk: {}", e))?;

    Ok(())
}

/// 检查今天是否需要更新
/// 返回 true 表示需要更新，false 表示可以跳过
pub fn should_update_today(state: &AppRuntimeState) -> bool {
    // 如果从未更新过，需要更新
    let Some(ref last_update) = state.last_successful_update else {
        log::info!(target: "runtime", "从未更新过，需要执行更新");
        return true;
    };

    // 解析最后更新时间
    let last_update_date = match chrono::DateTime::parse_from_rfc3339(last_update) {
        Ok(dt) => dt.with_timezone(&Local).date_naive(),
        Err(e) => {
            log::warn!(target: "runtime", "解析最后更新时间失败：{}，需要更新", e);
            return true;
        }
    };

    let today = Local::now().date_naive();

    // 如果最后更新不是今天，需要更新
    if last_update_date < today {
        log::info!(target: "runtime",
            "最后更新时间：{}，今天：{}，需要更新",
            last_update_date,
            today
        );
        true
    } else {
        false
    }
}

/// 检查本地是否已有今日壁纸
/// 通过检查本地壁纸列表的第一项的 end_date 是否匹配今天
pub async fn has_today_wallpaper(wallpaper_dir: &Path) -> bool {
    // 获取今天的日期字符串 (YYYYMMDD 格式)
    use chrono::Datelike;
    let today = Local::now().date_naive();
    let today_str = format!("{:04}{:02}{:02}", today.year(), today.month(), today.day());

    // 读取本地壁纸列表
    match crate::storage::get_local_wallpapers(wallpaper_dir).await {
        Ok(wallpapers) => {
            if let Some(first) = wallpapers.first() {
                // 使用 end_date 来判断这是否是今天的壁纸
                // 因为 Bing 的壁纸 startdate 是昨天，enddate 才是今天
                let has_today = first.end_date == today_str;
                if !has_today {
                    log::info!(target: "runtime",
                        "本地最新壁纸：{}，需要获取今日壁纸：{}",
                        first.end_date,
                        today_str
                    );
                }
                has_today
            } else {
                log::info!(target: "runtime", "本地没有任何壁纸，需要更新");
                false
            }
        }
        Err(e) => {
            log::warn!(target: "runtime", "读取本地壁纸失败：{}，假设需要更新", e);
            false
        }
    }
}

/// 更新最后成功更新时间
pub fn update_last_successful_time(app: &AppHandle, state: &mut AppRuntimeState) -> Result<()> {
    state.last_successful_update = Some(Local::now().to_rfc3339());
    save_runtime_state(app, state)?;
    Ok(())
}

/// 更新最后检查时间
pub fn update_last_check_time(app: &AppHandle, state: &mut AppRuntimeState) -> Result<()> {
    state.last_check_time = Some(Local::now().to_rfc3339());
    save_runtime_state(app, state)?;
    Ok(())
}

/// 检查是否可以跳过 API 请求（基于缓存策略）
/// 如果距离上次 API 请求不足 5 分钟，且本地有今日壁纸，可以跳过 API 请求
/// 注意：如果已经是新的一天，即使距离上次检查不足 5 分钟，也不能跳过（需要检查新壁纸）
pub async fn can_skip_api_request(state: &AppRuntimeState, wallpaper_dir: &Path) -> bool {
    // 检查是否有最后检查时间
    let Some(ref last_check_str) = state.last_check_time else {
        return false;
    };

    // 解析最后检查时间
    let last_check = match chrono::DateTime::parse_from_rfc3339(last_check_str) {
        Ok(dt) => dt.with_timezone(&Local),
        Err(_) => return false,
    };

    // 检查距离上次检查是否不足 5 分钟
    let now = Local::now();
    let duration_since_check = now.signed_duration_since(last_check);
    const CACHE_DURATION_MINUTES: i64 = 5;

    // 检查时间是否回退（系统时间可能被调整）
    if duration_since_check.num_minutes() < 0 {
        log::warn!(target: "runtime", 
            "检测到系统时间回退，重置缓存检查（last_check: {}, now: {}）", 
            last_check, now);
        return false;
    }

    // 重要：检查是否跨天了 - 如果跨天了，即使不足 5 分钟也不能跳过（需要检查新壁纸）
    let last_check_date = last_check.date_naive();
    let today = now.date_naive();
    if last_check_date < today {
        log::info!(target: "runtime",
            "检测到跨天（上次检查：{}，今天：{}），需要检查新壁纸，不能跳过 API 请求",
            last_check_date,
            today
        );
        return false;
    }

    if duration_since_check.num_minutes() < CACHE_DURATION_MINUTES {
        // 如果距离上次检查不足 5 分钟，检查本地是否有今日壁纸
        if has_today_wallpaper(wallpaper_dir).await {
            log::info!(target: "runtime", 
                "距离上次 API 请求不足 5 分钟且本地有今日壁纸，跳过 API 请求（缓存策略）");
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Local};

    #[test]
    fn test_should_update_today_never_updated() {
        let state = AppRuntimeState {
            last_successful_update: None,
            last_check_time: None,
        };

        assert!(should_update_today(&state));
    }

    #[test]
    fn test_should_update_today_updated_yesterday() {
        let yesterday = Local::now() - Duration::days(1);
        let state = AppRuntimeState {
            last_successful_update: Some(yesterday.to_rfc3339()),
            last_check_time: None,
        };

        assert!(should_update_today(&state));
    }

    #[test]
    fn test_should_update_today_updated_today() {
        let state = AppRuntimeState {
            last_successful_update: Some(Local::now().to_rfc3339()),
            last_check_time: None,
        };

        assert!(!should_update_today(&state));
    }

    #[test]
    fn test_should_update_today_invalid_timestamp() {
        let state = AppRuntimeState {
            last_successful_update: Some("invalid-timestamp".to_string()),
            last_check_time: None,
        };

        // Should return true when timestamp is invalid
        assert!(should_update_today(&state));
    }

    #[test]
    fn test_should_update_today_old_date() {
        let old_date = Local::now() - Duration::days(7);
        let state = AppRuntimeState {
            last_successful_update: Some(old_date.to_rfc3339()),
            last_check_time: None,
        };

        assert!(should_update_today(&state));
    }

    #[test]
    fn test_should_update_today_future_date() {
        // Edge case: if somehow the last update is in the future
        // (e.g., system clock changed), should still work correctly
        let future = Local::now() + Duration::days(1);
        let state = AppRuntimeState {
            last_successful_update: Some(future.to_rfc3339()),
            last_check_time: None,
        };

        // Future date should be considered "already updated today"
        assert!(!should_update_today(&state));
    }
}
