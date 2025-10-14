mod bing_api;
mod download_manager;
mod models;
mod storage;
mod wallpaper_manager;

use models::{AppSettings, BingImageEntry, LocalWallpaper};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tauri_plugin_autostart::ManagerExt;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager, Runtime,
};

// 全局状态管理
struct AppState {
    settings: Arc<Mutex<AppSettings>>,
    wallpaper_directory: Arc<Mutex<PathBuf>>,
    last_tray_click: Arc<Mutex<Option<Instant>>>,
}

/// 获取必应壁纸列表
#[tauri::command]
async fn fetch_bing_images(count: u8) -> Result<Vec<BingImageEntry>, String> {
    bing_api::fetch_bing_images(count, 0)
        .await
        .map_err(|e| e.to_string())
}

/// 下载壁纸
#[tauri::command]
async fn download_wallpaper(
    image_entry: BingImageEntry,
    state: tauri::State<'_, AppState>,
) -> Result<LocalWallpaper, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;

    // 确保目录存在
    storage::ensure_wallpaper_directory(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())?;

    // 检查文件是否已存在
    let save_path = storage::get_wallpaper_path(&wallpaper_dir, &image_entry.startdate);
    let metadata_path = save_path.with_extension("json");

    // 如果文件已存在，直接返回已有的壁纸信息
    if save_path.exists() && metadata_path.exists() {
        if let Ok(metadata_content) = tokio::fs::read_to_string(&metadata_path).await {
            if let Ok(wallpaper) = serde_json::from_str::<LocalWallpaper>(&metadata_content) {
                return Ok(wallpaper);
            }
        }
    }

    // 获取高分辨率图片 URL
    let image_url = bing_api::get_wallpaper_url(&image_entry.urlbase, "UHD");

    // 下载图片
    download_manager::download_image(&image_url, &save_path)
        .await
        .map_err(|e| e.to_string())?;

    // 创建本地壁纸记录
    let mut wallpaper = LocalWallpaper::from(image_entry);
    wallpaper.file_path = save_path.to_string_lossy().to_string();

    // 保存元数据
    storage::save_wallpaper_metadata(&wallpaper, &wallpaper_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(wallpaper)
}

/// 设置桌面壁纸
#[tauri::command]
async fn set_desktop_wallpaper(file_path: String) -> Result<(), String> {
    let path = PathBuf::from(file_path);
    wallpaper_manager::set_wallpaper(&path).map_err(|e| e.to_string())
}

/// 获取已下载的壁纸列表
#[tauri::command]
async fn get_local_wallpapers(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<LocalWallpaper>, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;
    storage::get_local_wallpapers(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())
}

/// 获取应用设置
#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

/// 更新应用设置
#[tauri::command]
async fn update_settings(
    new_settings: AppSettings,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;

    // 处理开机自启动设置
    let autostart_manager = app.autolaunch();
    if new_settings.launch_at_startup {
        autostart_manager.enable().map_err(|e| format!("启用开机自启动失败: {}", e))?;
    } else {
        autostart_manager.disable().map_err(|e| format!("禁用开机自启动失败: {}", e))?;
    }

    *settings = new_settings.clone();

    // 如果保存目录改变了,更新状态
    if let Some(ref new_dir) = new_settings.save_directory {
        let mut wallpaper_dir = state.wallpaper_directory.lock().await;
        *wallpaper_dir = PathBuf::from(new_dir);
    }

    Ok(())
}

/// 清理旧壁纸
#[tauri::command]
async fn cleanup_wallpapers(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;
    let settings = state.settings.lock().await;

    storage::cleanup_old_wallpapers(&wallpaper_dir, settings.keep_image_count as usize)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前桌面壁纸路径
#[tauri::command]
async fn get_current_wallpaper() -> Result<String, String> {
    wallpaper_manager::get_current_wallpaper().map_err(|e| e.to_string())
}

/// 获取默认壁纸目录
#[tauri::command]
async fn get_default_wallpaper_directory() -> Result<String, String> {
    storage::get_default_wallpaper_directory()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// 确保壁纸目录存在
#[tauri::command]
async fn ensure_wallpaper_directory_exists() -> Result<(), String> {
    let wallpaper_dir = storage::get_default_wallpaper_directory()
        .map_err(|e| e.to_string())?;
    storage::ensure_wallpaper_directory(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())
}

/// 显示主窗口
#[tauri::command]
async fn show_main_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.show().map_err(|e| e.to_string())?;
        window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// 设置系统托盘
fn setup_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    // 创建托盘菜单（只包含显示窗口和退出）
    let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // 使用默认窗口图标作为托盘图标
    let icon = app.default_window_icon().unwrap().clone();

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip("Bing Wallpaper Now")
        .show_menu_on_left_click(false) // 左键点击不显示菜单
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    // 左键点击切换窗口显示/隐藏
                    let app = tray.app_handle();

                    // 获取 AppState 进行防抖检查
                    if let Some(state) = app.try_state::<AppState>() {
                        let now = Instant::now();
                        let mut last_click = tauri::async_runtime::block_on(state.last_tray_click.lock());

                        // 防抖：如果距离上次点击少于 300ms，则忽略
                        if let Some(last) = *last_click {
                            if now.duration_since(last) < Duration::from_millis(300) {
                                return;
                            }
                        }

                        // 更新最后点击时间
                        *last_click = Some(now);
                        drop(last_click); // 显式释放锁

                        // 切换窗口显示状态
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                // 如果窗口可见，则隐藏
                                let _ = window.hide();
                            } else {
                                // 如果窗口隐藏，则显示并聚焦
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            }
        })
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化应用状态
    let default_dir = storage::get_default_wallpaper_directory()
        .unwrap_or_else(|_| PathBuf::from("."));

    let app_state = AppState {
        settings: Arc::new(Mutex::new(AppSettings::default())),
        wallpaper_directory: Arc::new(Mutex::new(default_dir)),
        last_tray_click: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            fetch_bing_images,
            download_wallpaper,
            set_desktop_wallpaper,
            get_local_wallpapers,
            get_settings,
            update_settings,
            cleanup_wallpapers,
            get_current_wallpaper,
            get_default_wallpaper_directory,
            ensure_wallpaper_directory_exists,
            show_main_window,
        ])
        .setup(|app| {
            // 设置系统托盘
            setup_tray(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            // 处理窗口关闭事件，隐藏而不是退出
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
