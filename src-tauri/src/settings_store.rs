//! 设置持久化模块
//!
//! 使用 tauri-plugin-store 管理应用设置的持久化存储

use crate::models::AppSettings;
use log::info;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const SETTINGS_STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";

/// 从 store 加载设置
pub fn load_settings(app: &AppHandle) -> anyhow::Result<AppSettings> {
    let store = app
        .store(SETTINGS_STORE_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to access store: {}", e))?;

    match store.get(SETTINGS_KEY) {
        Some(value) => {
            let mut settings: AppSettings = serde_json::from_value(value.clone())
                .map_err(|e| anyhow::anyhow!("Failed to deserialize settings: {}", e))?;

            // 归一化语言设置：非中文/英文的值一律走系统语言检测
            settings.normalize_language();
            // 先计算 resolved_language，再归一化 mkt（mkt 回退依赖 resolved_language）
            settings.compute_resolved_language();
            settings.normalize_mkt();

            Ok(settings)
        }
        None => {
            info!(target: "settings_store", "Store 中没有设置，使用默认设置");
            Ok(AppSettings::default())
        }
    }
}

/// 保存设置到 store
pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> anyhow::Result<()> {
    let store = app
        .store(SETTINGS_STORE_FILE)
        .map_err(|e| anyhow::anyhow!("Failed to access store: {}", e))?;

    let value = serde_json::to_value(settings)
        .map_err(|e| anyhow::anyhow!("Failed to serialize settings: {}", e))?;

    store.set(SETTINGS_KEY, value);

    store
        .save()
        .map_err(|e| anyhow::anyhow!("Failed to save store to disk: {}", e))?;

    info!(target: "settings_store", "成功保存设置到 store");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_store_constants() {
        assert_eq!(SETTINGS_STORE_FILE, "settings.json");
        assert_eq!(SETTINGS_KEY, "app_settings");
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let value = serde_json::to_value(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_value(value).unwrap();

        assert_eq!(deserialized.auto_update, settings.auto_update);
    }
}
