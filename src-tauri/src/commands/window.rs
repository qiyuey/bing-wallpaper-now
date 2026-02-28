use crate::wallpaper_manager;
use tauri::Manager;

/// 显示主窗口
#[tauri::command]
pub(crate) async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 获取所有屏幕的方向信息
#[tauri::command]
pub(crate) async fn get_screen_orientations()
-> Result<Vec<wallpaper_manager::ScreenOrientation>, String> {
    Ok(wallpaper_manager::get_screen_orientations())
}
