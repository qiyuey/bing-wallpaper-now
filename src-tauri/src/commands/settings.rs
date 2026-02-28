use crate::models::AppSettings;
use crate::{AppState, runtime_state, settings_store, storage, tray};
use log::{error, info, warn};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// 设置自启动通知标志（如果尚未设置）
///
/// 当用户启用自启动时，macOS 系统会显示通知。
/// 通过这个标志，我们可以记录用户已经看到过系统通知。
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `log_target` - 日志目标（用于区分调用上下文，如 "settings" 或 "startup"）
///
/// # 测试覆盖
/// 此函数依赖于 Tauri AppHandle，难以直接进行单元测试。
/// 但底层逻辑（`runtime_state::load_runtime_state` 和 `runtime_state::save_runtime_state`）
/// 已在 `runtime_state.rs` 模块中有完整的测试覆盖。
pub(crate) fn set_autostart_notification_flag_if_needed(app: &AppHandle, log_target: &str) {
    match runtime_state::load_runtime_state(app) {
        Ok(mut runtime_state) => {
            if !runtime_state.autostart_notification_shown {
                runtime_state.autostart_notification_shown = true;
                if let Err(e) = runtime_state::save_runtime_state(app, &runtime_state) {
                    warn!(target: log_target, "保存自启动通知标志失败: {}", e);
                } else {
                    info!(target: log_target, "已记录自启动通知已显示标志");
                }
            }
        }
        Err(e) => {
            warn!(target: log_target, "加载运行时状态失败，无法记录通知标志: {}", e);
        }
    }
}

/// 获取应用设置
#[tauri::command]
pub(crate) async fn get_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    let stored_settings = settings_store::load_settings(&app).unwrap_or_else(|e| {
        warn!(target: "settings", "从 store 加载设置失败: {}，使用内存中的设置", e);
        tauri::async_runtime::block_on(async { state.settings.lock().await.clone() })
    });

    {
        let mut settings = state.settings.lock().await;
        *settings = stored_settings.clone();
    }

    let mut settings = stored_settings;

    let autostart_manager = app.autolaunch();
    let is_enabled = autostart_manager
        .is_enabled()
        .map_err(|e| format!("读取自启动状态失败: {}", e))?;

    settings.launch_at_startup = is_enabled;

    settings.compute_resolved_language();
    settings.normalize_mkt();

    {
        let mut app_settings = state.settings.lock().await;
        *app_settings = settings.clone();
    }

    Ok(settings)
}

#[tauri::command]
pub(crate) async fn update_settings(
    new_settings: AppSettings,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;

    let mut new_settings = new_settings;
    new_settings.normalize_language();
    new_settings.compute_resolved_language();
    new_settings.normalize_mkt();

    let old_language = settings.language.clone();
    let old_mkt = settings.mkt.clone();

    let autostart_manager = app.autolaunch();
    let current_autostart_enabled = autostart_manager.is_enabled().unwrap_or_else(|e| {
        warn!(target: "settings", "读取当前自启动状态失败: {}，假设为未启用", e);
        false
    });

    if new_settings.launch_at_startup != current_autostart_enabled {
        if new_settings.launch_at_startup {
            autostart_manager
                .enable()
                .map_err(|e| format!("启用开机自启动失败: {}", e))?;

            set_autostart_notification_flag_if_needed(&app, "settings");
        } else {
            autostart_manager
                .disable()
                .map_err(|e| format!("禁用开机自启动失败: {}", e))?;
        }
    }

    *settings = new_settings.clone();
    drop(settings);

    {
        let mut wallpaper_dir = state.wallpaper_directory.lock().await;
        if let Some(ref new_dir) = new_settings.save_directory {
            *wallpaper_dir = PathBuf::from(new_dir);
        } else {
            *wallpaper_dir =
                storage::get_default_wallpaper_directory().map_err(|e| e.to_string())?;
        }
    }

    settings_store::save_settings(&app, &new_settings)
        .map_err(|e| format!("保存设置到 store 失败: {}", e))?;

    state
        .settings_tx
        .send(new_settings.clone())
        .map_err(|e| format!("广播设置失败: {e}"))?;

    if new_settings.mkt != old_mkt {
        info!(target: "settings", "mkt 从 {} 切换到 {}，清空 last_actual_mkt", old_mkt, new_settings.mkt);
        *state.last_actual_mkt.lock().await = None;
        if let Ok(mut runtime_state) = runtime_state::load_runtime_state(&app) {
            runtime_state.last_actual_mkt = None;
            if let Err(e) = runtime_state::save_runtime_state(&app, &runtime_state) {
                warn!(target: "settings", "持久化清空 last_actual_mkt 失败: {}", e);
            }
        }
    }

    if new_settings.language != old_language {
        info!(target: "settings", "语言从 {} 切换到 {}，更新托盘菜单", old_language, new_settings.language);
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = tray::update_tray_menu(&app_clone).await {
                error!(target: "settings", "更新托盘菜单失败: {e}");
                warn!(target: "settings", "托盘菜单更新失败，可能需要重启应用");
            } else {
                info!(target: "settings", "托盘菜单更新成功");
            }
        });
    }

    Ok(())
}
