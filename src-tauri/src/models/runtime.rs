use serde::{Deserialize, Serialize};

/// Market 状态统一结构
///
/// 将分散的 mkt 相关状态收敛为一个语义清晰的结构体，
/// 作为前端获取 mkt 状态的唯一接口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
    /// 用户设置的 mkt
    pub requested_mkt: String,
    /// 实际生效的 mkt（可能被 Bing 重定向）
    pub effective_mkt: String,
    /// 是否存在 mismatch
    pub is_mismatch: bool,
}

/// 应用内部运行时状态（不展示给用户）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppRuntimeState {
    /// 最后成功更新时间（ISO 8601 格式）
    pub last_successful_update: Option<String>,
    /// 最后检查更新时间（ISO 8601 格式）
    pub last_check_time: Option<String>,
    /// 用户手动设置壁纸时，各语言的最新壁纸标识（key = 语言代码，value = end_date）
    /// 用于判断自动更新时是否需要跳过相同的壁纸
    #[serde(default)]
    pub manually_set_latest_wallpapers: std::collections::HashMap<String, String>,
    /// 用户选择"不再提醒"的最大版本号（如果最新版本小于等于此版本，则不提示）
    #[serde(default)]
    pub ignored_update_version: Option<String>,
    /// 自启动通知已显示标志（用于避免 macOS 系统重复显示自启动通知）
    /// 当用户首次启用自启动时设置为 true，表示用户已经看到过系统通知
    #[serde(default)]
    pub autostart_notification_shown: bool,
    /// Bing API 最近一次返回的实际 mkt（持久化，解决重启后读不到壁纸的问题）
    ///
    /// 当用户设置的 mkt（如 "en-US"）被 Bing 重定向到其他市场（如 "zh-CN"）时，
    /// 壁纸元数据保存在实际 mkt 下。此字段持久化后，重启时能立即用正确的 key 读取。
    #[serde(default)]
    pub last_actual_mkt: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_status_serialization() {
        let status = MarketStatus {
            requested_mkt: "en-US".to_string(),
            effective_mkt: "zh-CN".to_string(),
            is_mismatch: true,
        };

        let json = serde_json::to_string(&status).unwrap();
        let deserialized: MarketStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.requested_mkt, "en-US");
        assert_eq!(deserialized.effective_mkt, "zh-CN");
        assert!(deserialized.is_mismatch);
    }

    #[test]
    fn test_app_runtime_state_default() {
        let state = AppRuntimeState::default();
        assert!(state.last_successful_update.is_none());
        assert!(state.last_check_time.is_none());
        assert!(state.manually_set_latest_wallpapers.is_empty());
        assert!(state.ignored_update_version.is_none());
        assert!(!state.autostart_notification_shown);
        assert!(state.last_actual_mkt.is_none());
    }

    #[test]
    fn test_app_runtime_state_serialization() {
        let state = AppRuntimeState {
            last_successful_update: Some("2024-01-01T12:00:00+08:00".to_string()),
            last_actual_mkt: Some("zh-CN".to_string()),
            autostart_notification_shown: true,
            ignored_update_version: Some("1.0.0".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: AppRuntimeState = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.last_successful_update,
            Some("2024-01-01T12:00:00+08:00".to_string())
        );
        assert_eq!(deserialized.last_actual_mkt, Some("zh-CN".to_string()));
        assert!(deserialized.autostart_notification_shown);
        assert_eq!(
            deserialized.ignored_update_version,
            Some("1.0.0".to_string())
        );
    }
}
