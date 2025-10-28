mod bing_api;
mod download_manager;
mod index_manager;
mod macos_app;
mod models;
mod runtime_state;
mod settings_store;
mod storage;
mod wallpaper_manager;

use chrono::{DateTime, Local, TimeZone, Timelike};
use log::{error, info, warn};

use models::{AppSettings, LocalWallpaper};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::{Mutex, watch};

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

// 下载壁纸
// (removed obsolete download_wallpaper command)
/// 设置桌面壁纸（异步非阻塞）
#[tauri::command]
async fn set_desktop_wallpaper(
    file_path: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
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

    // 异步执行设置壁纸，避免阻塞 UI
    let target_for_spawn = target_can.clone();
    let app_clone = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = wallpaper_manager::set_wallpaper(&target_for_spawn) {
            error!(target: "wallpaper", "设置壁纸失败: {e}");
        } else {
            let state_clone = app_clone.state::<AppState>();
            let mut current_path = state_clone.current_wallpaper_path.lock().await;
            *current_path = Some(target_for_spawn);
        }
    });

    Ok(())
}

/// 重新下载缺失的壁纸文件
async fn redownload_missing_wallpapers(
    missing_wallpapers: Vec<LocalWallpaper>,
    wallpaper_dir: PathBuf,
    app: tauri::AppHandle,
) {
    info!(target: "commands", "开始重新下载 {} 张缺失的壁纸", missing_wallpapers.len());

    for wallpaper in missing_wallpapers {
        // 如果 urlbase 为空（旧数据），无法重新下载
        if wallpaper.urlbase.is_empty() {
            warn!(target: "commands", "壁纸缺少 urlbase 信息，无法重新下载: {}", wallpaper.start_date);
            continue;
        }

        // 构建完整的图片 URL
        let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, "UHD");

        // 构建保存路径
        let save_path = wallpaper_dir.join(format!("{}.jpg", wallpaper.start_date));

        // 下载图片
        match download_manager::download_image(&image_url, &save_path).await {
            Ok(()) => {
                info!(target: "commands", "成功重新下载壁纸: {}", save_path.display());
                // 发送事件通知前端
                let _ = app.emit("image-downloaded", &wallpaper.start_date);
            }
            Err(e) => {
                error!(target: "commands", "重新下载壁纸失败 {}: {}", wallpaper.start_date, e);
            }
        }
    }
}

/// 获取已下载的壁纸列表
#[tauri::command]
async fn get_local_wallpapers(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<Vec<LocalWallpaper>, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;

    let wallpapers = storage::get_local_wallpapers(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())?;

    // 检查文件是否存在，收集需要重新下载的壁纸
    let mut missing_wallpapers = Vec::new();
    for wallpaper in &wallpapers {
        let path = std::path::Path::new(&wallpaper.file_path);
        if !path.exists() {
            warn!(target: "commands", "壁纸文件不存在，将触发重新下载: {}", wallpaper.file_path);
            missing_wallpapers.push(wallpaper.clone());
        }
    }

    // 如果有缺失的文件，异步触发重新下载
    if !missing_wallpapers.is_empty() {
        let wallpaper_dir_clone = wallpaper_dir.clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            redownload_missing_wallpapers(missing_wallpapers, wallpaper_dir_clone, app_clone).await;
        });
    }

    info!(target: "commands", "成功获取 {} 张本地壁纸", wallpapers.len());
    Ok(wallpapers)
}

/// 获取应用设置
#[tauri::command]
async fn get_settings(
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<AppSettings, String> {
    // 从 store 加载设置
    let stored_settings = settings_store::load_settings(&app).unwrap_or_else(|e| {
        warn!(target: "settings", "从 store 加载设置失败: {}，使用内存中的设置", e);
        tauri::async_runtime::block_on(async { state.settings.lock().await.clone() })
    });

    // 更新内存中的设置
    {
        let mut settings = state.settings.lock().await;
        *settings = stored_settings.clone();
    }

    let mut settings = stored_settings;

    // 从系统读取真实的自启动状态
    let autostart_manager = app.autolaunch();
    let is_enabled = autostart_manager
        .is_enabled()
        .map_err(|e| format!("读取自启动状态失败: {}", e))?;

    // 更新设置中的自启动状态为系统实际状态
    settings.launch_at_startup = is_enabled;

    // 将同步后的设置写回 AppState
    {
        let mut app_settings = state.settings.lock().await;
        *app_settings = settings.clone();
    }

    // 注意：这里不应该发送设置更新通知，因为 get_settings 只是读取操作
    // 只有 update_settings 才应该触发更新通知

    Ok(settings)
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

    // 只在自启动状态改变时才调用系统 API，避免不必要的系统提示
    let autostart_manager = app.autolaunch();
    let current_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);

    if normalized.launch_at_startup != current_autostart_enabled {
        if normalized.launch_at_startup {
            autostart_manager
                .enable()
                .map_err(|e| format!("启用开机自启动失败: {}", e))?;
        } else {
            autostart_manager
                .disable()
                .map_err(|e| format!("禁用开机自启动失败: {}", e))?;
        }
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

    // 保存设置到 store
    settings_store::save_settings(&app, &normalized)
        .map_err(|e| format!("保存设置到 store 失败: {}", e))?;

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
            theme: "system".to_string(),
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

// 获取当前桌面壁纸路径
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
    run_update_cycle_internal(app, false).await;
}

/// 内部更新循环实现
/// @param force_update: 是否强制更新（忽略智能检查）
async fn run_update_cycle_internal(app: &AppHandle, force_update: bool) {
    let state = app.state::<AppState>();

    // 并发保护：若已有更新在进行，直接跳过
    {
        let mut flag = state.update_in_progress.lock().await;
        if *flag {
            return;
        }
        *flag = true;
    }

    // 取消 scopeguard，改为在所有返回路径手动重置，在函数末尾统一释放

    let settings_snapshot = {
        let s = state.settings.lock().await;
        s.clone()
    };

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

    // 智能更新检查（非强制更新时）
    if !force_update {
        // 加载运行时状态
        let runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();

        // 检查是否需要更新
        if !runtime_state::should_update_today(&runtime_state) {
            // 今天已经更新过，再检查本地是否真的有今日壁纸
            if runtime_state::has_today_wallpaper(&dir).await {
                info!(target: "update", "跳过更新：今天已更新且本地有今日壁纸");

                // 自动应用最新壁纸（用户可能更换过）
                if let Ok(list) = storage::get_local_wallpapers(&dir).await
                    && let Some(first) = list.first()
                {
                    let path = PathBuf::from(&first.file_path);
                    if let Err(e) = wallpaper_manager::set_wallpaper(&path) {
                        error!(target: "update", "设置壁纸失败：{}", e);
                    } else {
                        let mut current_path = state.current_wallpaper_path.lock().await;
                        *current_path = Some(path);
                    }
                }

                // 启动时跳过更新，不需要通知前端（前端会自己初始化加载）
                // 重置标志并返回
                let mut flag = state.update_in_progress.lock().await;
                *flag = false;
                return;
            }
            info!(target: "update", "今天已更新但本地没有今日壁纸，继续更新");
        }

        // 更新检查时间
        let mut runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
        let _ = runtime_state::update_last_check_time(app, &mut runtime_state);
    } else {
        info!(target: "update", "强制更新模式，跳过智能检查");
    }

    if let Err(e) = storage::ensure_wallpaper_directory(&dir).await {
        error!(target: "update", "创建目录失败: {e}");
        // 失败时重置标志
        let mut flag = state.update_in_progress.lock().await;
        *flag = false;
        return;
    }

    // 重试获取 Bing 图片（指数退避：最多10次，总计约17分钟）
    let mut images_opt = None;
    const MAX_RETRIES: u32 = 10;
    for attempt in 0..MAX_RETRIES {
        match bing_api::fetch_bing_images(8, 0).await {
            Ok(v) => {
                images_opt = Some(v);
                break;
            }
            Err(e) => {
                if attempt < MAX_RETRIES - 1 {
                    let backoff = 1 << attempt; // 指数退避：1, 2, 4, 8, 16, 32, 64, 128, 256, 512 秒
                    warn!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，{}s 后重试",
                        attempt + 1,
                        e,
                        backoff
                    );
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                } else {
                    error!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，已达最大重试次数（总计约17分钟）",
                        attempt + 1,
                        e
                    );
                }
            }
        }
    }

    let images = match images_opt {
        Some(v) => v,
        None => {
            error!(target: "update", "多次重试仍失败，跳过本次循环");
            let mut flag = state.update_in_progress.lock().await;
            *flag = false;
            return;
        }
    };

    // 首次启动优化：立即保存所有元信息，让前端可以马上展示列表
    let is_first_launch = storage::get_local_wallpapers(&dir)
        .await
        .map(|w| w.is_empty())
        .unwrap_or(false);

    if is_first_launch {
        info!(target: "update", "首次启动检测到，立即保存所有元信息供前端展示");

        // 立即为所有图片创建元信息（不下载图片）
        let metadata_list: Vec<LocalWallpaper> = images
            .iter()
            .map(|image| {
                let save_path = storage::get_wallpaper_path(&dir, &image.startdate);
                let mut w = LocalWallpaper::from(image.clone());
                w.file_path = save_path.to_string_lossy().to_string();
                w
            })
            .collect();

        // 批量保存元数据
        if let Err(e) = storage::save_wallpapers_metadata(metadata_list, &dir).await {
            error!(target: "update", "保存元数据失败: {e}");
        } else {
            // 立即通知前端刷新列表
            if let Err(e) = app.emit("wallpaper-updated", ()) {
                warn!(target: "update", "通知前端失败: {e}");
            }
            info!(target: "update", "元信息已保存并通知前端，开始后台下载图片");
        }
    }

    // 并发下载图片（元数据顺序已在首次启动时保证，下载顺序不影响显示顺序）
    let mut success_count = 0;
    let mut fail_count = 0;

    // 准备下载任务列表（仅下载不存在的文件）
    let download_tasks: Vec<_> = images
        .iter()
        .filter_map(|image| {
            let save_path = storage::get_wallpaper_path(&dir, &image.startdate);
            if save_path.exists() {
                None
            } else {
                let url = bing_api::get_wallpaper_url(&image.urlbase, "UHD");
                Some((url, save_path, image.clone()))
            }
        })
        .collect();

    if !download_tasks.is_empty() {
        info!(target: "update", "开始并发下载 {} 张图片", download_tasks.len());

        // 使用 futures 并发下载（最大并发数 4）
        use futures::stream::{self, StreamExt};
        let app_for_tasks = app.clone();
        let dir_for_tasks = dir.clone();

        let results: Vec<_> = stream::iter(download_tasks)
            .map(|(url, save_path, image)| {
                let app_clone = app_for_tasks.clone();
                let dir_clone = dir_for_tasks.clone();
                async move {
                    let startdate = image.startdate.clone();
                    match download_manager::download_image(&url, &save_path).await {
                        Ok(_) => {
                            info!(target: "update", "图片下载成功: {}", startdate);

                            // 非首次启动时，每下载一张就保存元数据
                            if !is_first_launch {
                                let mut w = LocalWallpaper::from(image.clone());
                                w.file_path = save_path.to_string_lossy().to_string();

                                if let Err(e) =
                                    storage::save_wallpapers_metadata(vec![w], &dir_clone).await
                                {
                                    warn!(target: "update", "保存元数据失败: {e}");
                                }
                            }

                            // 通知前端：单张图片下载完成
                            if let Err(e) = app_clone.emit("image-downloaded", startdate.clone()) {
                                warn!(target: "update", "通知前端图片下载完成失败: {e}");
                            }

                            Ok(startdate)
                        }
                        Err(e) => {
                            warn!(target: "update", "图片下载失败 {}: {}", startdate, e);
                            Err((startdate, e))
                        }
                    }
                }
            })
            .buffer_unordered(4) // 最大并发数 4
            .collect()
            .await;

        // 统计结果
        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(_) => fail_count += 1,
            }
        }
    }

    if success_count > 0 || fail_count > 0 {
        if fail_count > 0 {
            warn!(target: "update", "下载完成: 成功 {}, 失败 {}", success_count, fail_count);
        } else {
            info!(target: "update", "全部 {} 张图片下载成功", success_count);
        }
    }

    // 清理旧文件
    if let Err(e) =
        storage::cleanup_old_wallpapers(&dir, settings_snapshot.keep_image_count as usize).await
    {
        warn!(target: "update", "清理旧壁纸失败: {e}");
    }

    // 自动应用最新壁纸：现在无条件执行
    if let Ok(list) = storage::get_local_wallpapers(&dir).await
        && let Some(first) = list.first()
    {
        let path = PathBuf::from(&first.file_path);
        if let Err(e) = wallpaper_manager::set_wallpaper(&path) {
            error!(target: "update", "设置壁纸失败: {e}");
        } else {
            let mut current_path = state.current_wallpaper_path.lock().await;
            *current_path = Some(path);
        }
    }

    info!(target: "update", "完成一次更新循环");
    // 记录最后更新时间
    {
        let mut last = state.last_update_time.lock().await;
        *last = Some(Local::now());
    }

    // 保存运行时状态（更新成功）
    {
        let mut runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
        let _ = runtime_state::update_last_successful_time(app, &mut runtime_state);
    }

    // 通知前端刷新壁纸列表
    if let Err(e) = app.emit("wallpaper-updated", ()) {
        warn!(target: "update", "通知前端失败: {e}");
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
    // 调用强制更新版本，跳过智能检查
    run_update_cycle_internal(&app, true).await;
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

            // 标记是否是第一次收到设置变更（启动时的初始化不算）
            let mut is_first_change = true;

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
                                warn!(target:"auto_update","零点窗口初次更新可能失败，开始指数退避重试");
                                // 指数退避重试：最多10次，总计约17分钟
                                const MAX_MIDNIGHT_RETRIES: u32 = 10;
                                for attempt in 0..MAX_MIDNIGHT_RETRIES {
                                    let backoff = 1 << attempt; // 1, 2, 4, 8, 16, 32, 64, 128, 256, 512 秒
                                    warn!(target:"auto_update","零点重试第 {} 次，{}s 后执行", attempt + 1, backoff);
                                    tokio::time::sleep(Duration::from_secs(backoff)).await;

                                    run_update_cycle(&app_clone).await;
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
                                    warn!(target:"auto_update","零点重试结束（总计约17分钟），仍未成功获取当日壁纸，等待下一轮小时轮询");
                                }
                            }
                        } else {
                            // 普通每小时轮询
                            run_update_cycle(&app_clone).await;
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
                            info!(target: "update", "自动更新已关闭，等待重新开启...");
                            loop {
                                if rx.changed().await.is_err() { break; }
                                let s = rx.borrow().clone();
                                if s.auto_update {
                                    info!(target: "update", "自动更新重新开启，立即执行一次");
                                    run_update_cycle(&app_clone).await;
                                    break;
                                }
                            }
                        } else {
                            info!(target: "update", "设置改变，立即执行更新");
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
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let show_item = MenuItemBuilder::with_id("show", "显示窗口").build(app)?;
    let refresh_item = MenuItemBuilder::with_id("refresh", "更新壁纸").build(app)?;
    let open_folder_item = MenuItemBuilder::with_id("open_folder", "打开保存目录").build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", "打开设置").build(app)?;
    let about_item = MenuItemBuilder::with_id("about", "关于").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出").build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&refresh_item)
        .item(&open_folder_item)
        .item(&settings_item)
        .item(&about_item)
        .separator()
        .item(&quit_item)
        .build()?;

    // Windows 高 DPI 下托盘图标优化：使用更高分辨率的 PNG 图标
    // 在 200% 缩放下，128x128 的图标可以提供清晰的显示效果（等效 64x64 物理像素）
    #[cfg(target_os = "windows")]
    let icon = {
        let icon_bytes = include_bytes!("../icons/128x128.png");
        let icon_img = image::load_from_memory(icon_bytes)
            .map_err(|e| {
                tauri::Error::InvalidIcon(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?
            .to_rgba8();
        tauri::image::Image::new_owned(icon_img.to_vec(), icon_img.width(), icon_img.height())
    };

    // macOS 使用黑白托盘图标（符合系统设计规范）
    #[cfg(target_os = "macos")]
    let icon = {
        let icon_bytes = include_bytes!("../icons/tray-icon-macos@2x.png");
        let icon_img = image::load_from_memory(icon_bytes)
            .map_err(|e| {
                tauri::Error::InvalidIcon(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?
            .to_rgba8();
        tauri::image::Image::new_owned(icon_img.to_vec(), icon_img.width(), icon_img.height())
    };

    // Linux 和其他平台使用默认图标
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let icon = app.default_window_icon().unwrap().clone();

    let _tray = TrayIconBuilder::new()
        .menu(&menu)
        .icon(icon)
        .tooltip("Bing Wallpaper Now")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event
                && button == tauri::tray::MouseButton::Left
            {
                let app = tray.app_handle();
                if let Some(state) = app.try_state::<AppState>() {
                    let now = Instant::now();
                    let mut last_click =
                        tauri::async_runtime::block_on(state.last_tray_click.lock());

                    if let Some(last) = *last_click
                        && now.duration_since(last) < Duration::from_millis(300)
                    {
                        return;
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
                    if let Err(e) = force_update(app_handle).await {
                        warn!(target: "update", "托盘更新失败: {}", e);
                    }
                });
            }
            "open_folder" => {
                // 通过事件通知前端打开目录（复用前端已有逻辑）
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit("open-folder", ());
            }
            "settings" => {
                // 显示主窗口并向前端发送事件，前端可监听此事件弹出设置
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit("open-settings", ());
            }
            "about" => {
                // 显示主窗口并向前端发送事件，前端可监听此事件弹出关于对话框
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
                let _ = app.emit("open-about", ());
            }
            "quit" => {
                // Properly cleanup before exit to avoid Windows class registration errors
                // First, abort any running background tasks
                if let Some(state) = app.try_state::<AppState>() {
                    tauri::async_runtime::block_on(async {
                        let handle = state.auto_update_handle.lock().await;
                        handle.abort();
                    });
                }

                // Destroy all windows to ensure WebView2 cleanup
                if let Some(window) = app.get_webview_window("main") {
                    // Use destroy() instead of close() for complete cleanup
                    let _ = window.destroy();
                }

                // Small delay to ensure WebView2 cleanup completes
                std::thread::sleep(std::time::Duration::from_millis(150));

                // Use process::exit for immediate termination
                std::process::exit(0);
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
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当检测到第二个实例启动时，将第一个实例的窗口显示出来
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
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

            // 从 store 加载持久化设置
            let loaded_settings = settings_store::load_settings(app.handle()).unwrap_or_else(|e| {
                warn!(target: "settings", "从 store 加载设置失败: {}，使用默认设置", e);
                AppSettings::default()
            });

            // 更新 AppState 中的设置
            let state = app.state::<AppState>();
            tauri::async_runtime::block_on(async {
                let mut settings = state.settings.lock().await;
                *settings = loaded_settings.clone();
            });

            // 同步持久化设置到 settings_tx watch channel
            // 这样 auto_update_task 等监听者能获取到正确的初始设置
            if let Err(e) = state.settings_tx.send(loaded_settings.clone()) {
                warn!(target: "settings", "发送持久化设置到 watch channel 失败: {}", e);
            }

            // 更新壁纸目录
            let wallpaper_dir = if let Some(ref dir) = loaded_settings.save_directory {
                PathBuf::from(dir)
            } else {
                storage::get_default_wallpaper_directory().unwrap_or_else(|_| PathBuf::from("."))
            };
            tauri::async_runtime::block_on(async {
                let mut dir = state.wallpaper_directory.lock().await;
                *dir = wallpaper_dir;
            });

            info!(target: "settings", "成功加载持久化设置");

            // 从持久化状态加载上次更新时间
            {
                if let Ok(runtime_state) = runtime_state::load_runtime_state(app.handle()) {
                    if let Some(ref last_update_str) = runtime_state.last_successful_update {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_update_str) {
                            tauri::async_runtime::block_on(async {
                                let mut last_update = state.last_update_time.lock().await;
                                *last_update = Some(dt.with_timezone(&Local));
                            });
                            info!(target: "startup", "从持久化状态恢复上次更新时间: {}", last_update_str);
                        }
                    }
                }
            }

            setup_tray(app.handle())?;

            // macOS: 始终设置为 Accessory 模式（只显示托盘图标，不显示 Dock 图标）
            macos_app::set_activation_policy_accessory();

            // 检查是否是自启动（通过命令行参数）
            let is_autostart = std::env::args()
                .any(|arg| arg == "--minimized" || arg == "--hidden" || arg == "--startup");

            // 如果不是自启动，显示主窗口
            if !is_autostart && let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }

            // 使用 tauri-plugin-log 进行标准化日志输出（已在 Builder 中初始化）
            start_auto_update_task(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Check if this is a real quit request (from tray menu)
                // If not, just hide the window
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
