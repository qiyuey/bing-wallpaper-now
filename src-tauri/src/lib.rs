mod bing_api;
mod download_manager;
mod models;
mod storage;
mod wallpaper_manager;

use chrono::{DateTime, Local, TimeZone, Timelike};
use log::{debug, error, info, trace, warn};

use futures::stream::{FuturesUnordered, StreamExt};
use models::{AppSettings, LocalWallpaper};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, Runtime,
};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::{watch, Mutex};

/// 全局状态管理
struct AppState {
    settings: Arc<Mutex<AppSettings>>,
    wallpaper_directory: Arc<Mutex<PathBuf>>,
    last_tray_click: Arc<Mutex<Option<Instant>>>,
    current_wallpaper_path: Arc<Mutex<Option<PathBuf>>>,
    last_update_time: Arc<Mutex<Option<DateTime<Local>>>>,
    settings_tx: watch::Sender<AppSettings>,
    settings_rx: watch::Receiver<AppSettings>,
    auto_update_handle: Arc<Mutex<tauri::async_runtime::JoinHandle<()>>>,
    update_in_progress: Arc<Mutex<bool>>,
}

// (removed) fetch_bing_images command; image retrieval now handled by background auto-update logic.

/// 下载壁纸
// (removed obsolete download_wallpaper command)

/// 设置桌面壁纸
#[tauri::command]
async fn set_desktop_wallpaper(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let path = PathBuf::from(&file_path);

    // 路径校验：必须位于当前壁纸目录内，防止设置任意系统文件为壁纸
    let base_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };
    let base_dir_can = base_dir
        .canonicalize()
        .map_err(|e| format!("无法解析壁纸目录: {e}"))?;
    let target_can = path
        .canonicalize()
        .map_err(|e| format!("无法解析目标路径: {e}"))?;

    if !target_can.starts_with(&base_dir_can) {
        return Err("目标文件不在壁纸目录下，拒绝设置".into());
    }
    if !target_can.is_file() {
        return Err("目标文件不存在或不是普通文件".into());
    }

    wallpaper_manager::set_wallpaper(&target_can).map_err(|e| e.to_string())?;

    // 保存当前壁纸路径
    let mut current_path = state.current_wallpaper_path.lock().await;
    *current_path = Some(target_can);

    Ok(())
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

/// 设置归一化（内部函数）
fn normalize_settings(mut s: AppSettings) -> AppSettings {
    if s.keep_image_count < 8 {
        s.keep_image_count = 8;
    }
    s
}

#[tauri::command]
async fn update_settings(
    new_settings: AppSettings,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;

    // 统一归一化逻辑
    let normalized = normalize_settings(new_settings);

    let autostart_manager = app.autolaunch();
    if normalized.launch_at_startup {
        autostart_manager
            .enable()
            .map_err(|e| format!("启用开机自启动失败: {}", e))?;
    } else {
        autostart_manager
            .disable()
            .map_err(|e| format!("禁用开机自启动失败: {}", e))?;
    }

    *settings = normalized.clone();
    drop(settings);

    // 更新壁纸目录
    {
        let mut wallpaper_dir = state.wallpaper_directory.lock().await;
        if let Some(ref new_dir) = normalized.save_directory {
            *wallpaper_dir = PathBuf::from(new_dir);
        } else {
            *wallpaper_dir =
                storage::get_default_wallpaper_directory().map_err(|e| e.to_string())?;
        }
    }

    // 广播设置变化
    state
        .settings_tx
        .send(normalized)
        .map_err(|e| format!("广播设置失败: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_normalize_settings_minimums() {
        let s = AppSettings {
            auto_update: true,
            save_directory: None,
            keep_image_count: 3,
            launch_at_startup: false,
        };
        let n = normalize_settings(s);
        assert_eq!(n.keep_image_count, 8);
    }
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
// (removed obsolete get_current_wallpaper command)

/// 获取默认壁纸目录
#[tauri::command]
async fn get_default_wallpaper_directory() -> Result<String, String> {
    storage::get_default_wallpaper_directory()
        .map_err(|e| e.to_string())
        .map(|p| p.to_string_lossy().to_string())
}

/// 获取最后一次成功更新时间（本地时区）
#[tauri::command]
async fn get_last_update_time(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let guard = state.last_update_time.lock().await;
    Ok(guard.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()))
}

#[tauri::command]
async fn get_update_in_progress(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let flag = state.update_in_progress.lock().await;
    Ok(*flag)
}

/// 确保壁纸目录存在
#[tauri::command]
async fn ensure_wallpaper_directory_exists(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    storage::ensure_wallpaper_directory(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前壁纸目录（用户自定义或默认）
#[tauri::command]
async fn get_wallpaper_directory(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;
    Ok(wallpaper_dir.to_string_lossy().to_string())
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

/// 单次更新循环：下载、保存、清理、可选应用最新壁纸（含重试与共享客户端）
async fn run_update_cycle(app: &AppHandle) {
    let state = app.state::<AppState>();

    // 并发保护：若已有更新在进行，直接跳过
    {
        let mut flag = state.update_in_progress.lock().await;
        if *flag {
            trace!(target: "auto_update", "已有更新在进行中，跳过本次触发");
            return;
        }
        *flag = true;
    }

    // 取消 scopeguard，改为在所有返回路径手动重置，在函数末尾统一释放

    let settings_snapshot = {
        let s = state.settings.lock().await;
        s.clone()
    };
    trace!(target: "auto_update", "开始一次更新循环");

    if !settings_snapshot.auto_update {
        // 未开启自动更新，重置标志后返回
        let mut flag = state.update_in_progress.lock().await;
        *flag = false;
        return;
    }

    let dir = {
        let d = state.wallpaper_directory.lock().await;
        d.clone()
    };

    if let Err(e) = storage::ensure_wallpaper_directory(&dir).await {
        error!(target: "auto_update", "创建目录失败: {e}");
        // 失败时重置标志
        let mut flag = state.update_in_progress.lock().await;
        *flag = false;
        return;
    }

    // 重试获取 Bing 图片（指数退避）
    let mut images_opt = None;
    for attempt in 0..3 {
        match bing_api::fetch_bing_images(8, 0).await {
            Ok(v) => {
                images_opt = Some(v);
                break;
            }
            Err(e) => {
                let backoff = 1 << attempt;
                warn!(target: "auto_update",
                    "获取 Bing 图片失败(第 {} 次): {}，{}s 后重试",
                    attempt + 1,
                    e,
                    backoff
                );
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
        }
    }

    let images = match images_opt {
        Some(v) => v,
        None => {
            error!(target: "auto_update", "多次重试仍失败，跳过本次循环");
            let mut flag = state.update_in_progress.lock().await;
            *flag = false;
            return;
        }
    };
    debug!(target: "auto_update", "获取到 {} 张图片用于本次处理", images.len());

    let mut tasks = FuturesUnordered::new();
    for image in images {
        let dir_clone = dir.clone();
        tasks.push(async move {
            let save_path = storage::get_wallpaper_path(&dir_clone, &image.startdate);
            if save_path.exists() {
                return Ok::<(), anyhow::Error>(());
            }
            let url = bing_api::get_wallpaper_url(&image.urlbase, "UHD");
            // 使用 download_manager 并加入简单重试 + 退避
            let mut attempt = 0;
            loop {
                match download_manager::download_image(&url, &save_path).await {
                    Ok(_) => break,
                    Err(e) => {
                        attempt += 1;
                        if attempt >= 3 {
                            anyhow::bail!("下载失败: {}", e);
                        }
                        let backoff = 1 << (attempt - 1);
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                    }
                }
            }
            let mut w = LocalWallpaper::from(image);
            w.file_path = save_path.to_string_lossy().to_string();
            storage::save_wallpaper_metadata(&w, &dir_clone).await?;
            Ok(())
        });
    }

    while let Some(result) = tasks.next().await {
        if let Err(e) = result {
            warn!(target: "auto_update", "下载错误: {e}");
        }
    }

    // 清理旧文件
    if let Err(e) =
        storage::cleanup_old_wallpapers(&dir, settings_snapshot.keep_image_count as usize).await
    {
        warn!(target: "auto_update", "清理旧壁纸失败: {e}");
    }

    // 自动应用最新壁纸：现在无条件执行
    if let Ok(list) = storage::get_local_wallpapers(&dir).await {
        if let Some(first) = list.first() {
            let path = PathBuf::from(&first.file_path);
            if let Err(e) = wallpaper_manager::set_wallpaper(&path) {
                error!(target: "auto_update", "设置壁纸失败: {e}");
            } else {
                let mut current_path = state.current_wallpaper_path.lock().await;
                *current_path = Some(path);
            }
        }
    }

    info!(target: "auto_update", "完成一次更新循环");
    // 记录最后更新时间
    {
        let mut last = state.last_update_time.lock().await;
        *last = Some(Local::now());
    }
    // 末尾重置 update_in_progress
    {
        let mut flag = state.update_in_progress.lock().await;
        *flag = false;
    }
}

/// 手动强制执行一次更新
#[tauri::command]
async fn force_update(app: tauri::AppHandle) -> Result<(), String> {
    trace!(target: "auto_update", "收到手动强制更新指令");
    // 直接调用 run_update_cycle，内部已做并发保护
    run_update_cycle(&app).await;
    Ok(())
}

/// 启动自动更新任务（响应设置变更，可取消）
fn start_auto_update_task(app: AppHandle) {
    let state = app.state::<AppState>();
    let mut rx = state.settings_rx.clone();

    // 如已有旧任务，先取消（不需要获取 runtime handle）
    tauri::async_runtime::block_on(async {
        let mut h = state.auto_update_handle.lock().await;
        h.abort();
        let app_clone = app.clone();
        let new_handle = tauri::async_runtime::spawn(async move {
            // 初始立即执行一次
            run_update_cycle(&app_clone).await;
            // 小时循环 + 零点对齐
            loop {
                // 计算距下一次本地零点（含 5 分钟缓冲）剩余时间
                let now = Local::now();
                let tomorrow = now.date_naive().succ_opt().unwrap();
                let naive_next = tomorrow.and_hms_opt(0, 5, 0).unwrap();
                let next_midnight = Local.from_local_datetime(&naive_next).unwrap();
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
                            trace!(target:"auto_update","零点窗口内执行每日对齐更新");
                            // 记录更新前的日期
                            run_update_cycle(&app_clone).await;
                            let today = after_sleep_now.date_naive();
                            // 判断是否成功（last_update_time 是否被更新为今日）
                            let mut need_retry = {
                                let state_ref = app_clone.state::<AppState>();
                                let guard = state_ref.last_update_time.lock().await;
                                guard.map(|dt| dt.date_naive()) != Some(today)
                            };
                            if need_retry {
                                warn!(target:"auto_update","零点窗口初次更新可能失败，开始快速重试");
                                // 最多重试到 01:00 或成功为止；每 10 分钟一次
                                for attempt in 1..=6 {
                                    let now_retry = Local::now();
                                    if now_retry.hour() >= 1 { break; }
                                    tokio::time::sleep(Duration::from_secs(600)).await; // 10 分钟
                                    run_update_cycle(&app_clone).await;
                                    let after_cycle_success = {
                                        let state_ref = app_clone.state::<AppState>();
                                        let guard = state_ref.last_update_time.lock().await;
                                        guard.map(|dt| dt.date_naive()) == Some(now_retry.date_naive())
                                    };
                                    if after_cycle_success {
                                        info!(target:"auto_update","快速重试第 {} 次成功", attempt);
                                        need_retry = false;
                                        break;
                                    } else {
                                        warn!(target:"auto_update","快速重试第 {} 次仍未获取到当日壁纸", attempt);
                                    }
                                }
                                if need_retry {
                                    warn!(target:"auto_update","快速重试结束，仍未成功获取当日壁纸，等待下一轮小时轮询");
                                }
                            }
                        } else {
                            // 普通每小时轮询
                            run_update_cycle(&app_clone).await;
                        }
                    }
                    changed = rx.changed() => {
                        if changed.is_err() {
                            error!(target: "auto_update", "settings watch channel closed");
                            break;
                        }
                        let latest = rx.borrow().clone();
                        if !latest.auto_update {
                            info!(target: "auto_update", "自动更新已关闭，等待重新开启...");
                            loop {
                                if rx.changed().await.is_err() { break; }
                                let s = rx.borrow().clone();
                                if s.auto_update {
                                    info!(target: "auto_update", "自动更新重新开启，立即执行一次");
                                    run_update_cycle(&app_clone).await;
                                    break;
                                }
                            }
                        } else {
                            info!(target: "auto_update", "设置改变，立即执行更新");
                            run_update_cycle(&app_clone).await;
                        }
                    }
                }
            }
        });
        *h = new_handle;
    });
}

/// 设置系统托盘
fn setup_tray<R: Runtime>(app: &tauri::AppHandle<R>) -> tauri::Result<()> {
    let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
    let refresh_item = MenuItemBuilder::with_id("refresh", "刷新壁纸").build(app)?;
    let open_folder_item = MenuItemBuilder::with_id("open_folder", "打开保存目录").build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", "打开设置").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&refresh_item)
        .item(&open_folder_item)
        .item(&settings_item)
        .separator()
        .item(&quit_item)
        .build()?;

    let icon = app.default_window_icon().unwrap().clone();

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip("Bing Wallpaper Now")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(state) = app.try_state::<AppState>() {
                        let now = Instant::now();
                        let mut last_click =
                            tauri::async_runtime::block_on(state.last_tray_click.lock());

                        if let Some(last) = *last_click {
                            if now.duration_since(last) < Duration::from_millis(300) {
                                return;
                            }
                        }

                        *last_click = Some(now);
                        drop(last_click);

                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            }
        })
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "refresh" => {
                // 异步触发一次强制更新
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = force_update(app_handle) {
                        warn!(target: "auto_update", "托盘刷新失败: {}", e);
                    }
                });
            }
            "open_folder" => {
                // 打开保存目录
                let app_handle = app.clone();
                tauri::async_runtime::spawn(async move {
                    if let Some(state) = app_handle.try_state::<AppState>() {
                        let dir = {
                            let guard = state.wallpaper_directory.lock().await;
                            guard.clone()
                        };
                        if let Err(e) = tauri_plugin_opener::open_path(&app_handle.shell(), &dir) {
                            warn!(target: "auto_update", "托盘打开目录失败: {}", e);
                        }
                    }
                });
            }
            "settings" => {
                // 显示主窗口并向前端发送事件，前端可监听此事件弹出设置
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit("open-settings", ());
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let default_dir =
        storage::get_default_wallpaper_directory().unwrap_or_else(|_| PathBuf::from("."));

    // 初始设置
    let initial_settings = AppSettings::default();
    let (tx, rx) = watch::channel(initial_settings.clone());

    let app_state = AppState {
        settings: Arc::new(Mutex::new(initial_settings)),
        wallpaper_directory: Arc::new(Mutex::new(default_dir)),
        last_tray_click: Arc::new(Mutex::new(None)),
        current_wallpaper_path: Arc::new(Mutex::new(None)),
        last_update_time: Arc::new(Mutex::new(None)),
        settings_tx: tx,
        settings_rx: rx,
        auto_update_handle: Arc::new(Mutex::new(tauri::async_runtime::spawn(async {}))),
        update_in_progress: Arc::new(Mutex::new(false)),
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
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            set_desktop_wallpaper,
            get_local_wallpapers,
            get_settings,
            update_settings,
            cleanup_wallpapers,
            get_wallpaper_directory,
            get_default_wallpaper_directory,
            get_last_update_time,
            get_update_in_progress,
            ensure_wallpaper_directory_exists,
            show_main_window,
            force_update,
        ])
        .setup(|app| {
            wallpaper_manager::initialize_observer();
            setup_tray(app.handle())?;
            // 使用 tauri-plugin-log 进行标准化日志输出（已在 Builder 中初始化）
            start_auto_update_task(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
