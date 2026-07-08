use crate::{AppState, wallpaper_manager};
use log::{error, info, warn};
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::Manager;

const FRONTEND_READY_TIMEOUT_SECS: u64 = 7;
const FRONTEND_RELOAD_GRACE_SECS: u64 = 8;
const FRONTEND_LOG_LIMIT: usize = 4000;

fn truncate_for_log(value: &str) -> String {
    if value.chars().count() <= FRONTEND_LOG_LIMIT {
        value.to_string()
    } else {
        let mut truncated: String = value.chars().take(FRONTEND_LOG_LIMIT).collect();
        truncated.push_str("...");
        truncated
    }
}

pub(crate) fn schedule_frontend_ready_watchdog(app: tauri::AppHandle, source: &'static str) {
    let state = app.state::<AppState>();
    let frontend_ready = state.frontend_ready.clone();
    let reload_attempted = state.frontend_reload_attempted.clone();

    tauri::async_runtime::spawn(async move {
        if frontend_ready.load(Ordering::SeqCst) {
            return;
        }

        tokio::time::sleep(Duration::from_secs(FRONTEND_READY_TIMEOUT_SECS)).await;

        if frontend_ready.load(Ordering::SeqCst) {
            return;
        }

        if reload_attempted.swap(true, Ordering::SeqCst) {
            warn!(target: "frontend", "前端 ready 超时，但已请求过 reload，来源: {}", source);
            return;
        }

        warn!(target: "frontend",
            "前端在 {} 秒内未完成 ready 握手，尝试 reload 主 WebView，来源: {}",
            FRONTEND_READY_TIMEOUT_SECS,
            source
        );

        if let Some(window) = app.get_webview_window("main") {
            if let Err(e) = window.reload() {
                warn!(target: "frontend", "reload 主 WebView 失败: {}", e);
                return;
            }

            tokio::time::sleep(Duration::from_secs(FRONTEND_RELOAD_GRACE_SECS)).await;
            if frontend_ready.load(Ordering::SeqCst) {
                info!(target: "frontend", "reload 后前端 ready 握手已恢复");
            } else {
                warn!(target: "frontend",
                    "reload 后 {} 秒仍未收到前端 ready 握手，可能仍为空白 WebView",
                    FRONTEND_RELOAD_GRACE_SECS
                );
            }
        } else {
            warn!(target: "frontend", "找不到 main WebView，无法执行 ready watchdog reload");
        }
    });
}

pub(crate) fn show_main_window_with_watchdog(
    app: &tauri::AppHandle,
    source: &'static str,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
        schedule_frontend_ready_watchdog(app.clone(), source);
    }
    Ok(())
}

/// 显示主窗口
#[tauri::command]
pub(crate) async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    show_main_window_with_watchdog(&app, "show_main_window")
}

/// 标记前端已经完成首屏挂载。
#[tauri::command]
pub(crate) async fn mark_frontend_ready(app: tauri::AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let was_ready = state.frontend_ready.swap(true, Ordering::SeqCst);
    state
        .frontend_reload_attempted
        .store(false, Ordering::SeqCst);

    if !was_ready {
        info!(target: "frontend", "前端已完成 ready 握手");
    }

    Ok(())
}

/// 将 WebView 中的前端错误写入 Tauri 日志。
#[tauri::command]
pub(crate) async fn report_frontend_error(
    source: String,
    message: String,
    stack: Option<String>,
    context: Option<String>,
) -> Result<(), String> {
    let source = truncate_for_log(&source);
    let message = truncate_for_log(&message);
    let stack = stack
        .as_deref()
        .map(truncate_for_log)
        .unwrap_or_else(|| "no stack".to_string());
    let context = context
        .as_deref()
        .map(truncate_for_log)
        .unwrap_or_else(|| "no context".to_string());

    error!(target: "frontend",
        "前端错误 [{}]: {}\ncontext: {}\n{}",
        source,
        message,
        context,
        stack
    );
    Ok(())
}

/// 获取所有屏幕的方向信息
#[tauri::command]
pub(crate) async fn get_screen_orientations()
-> Result<Vec<wallpaper_manager::ScreenOrientation>, String> {
    Ok(wallpaper_manager::get_screen_orientations())
}
