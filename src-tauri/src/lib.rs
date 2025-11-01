mod bing_api;
mod download_manager;
mod index_manager;
mod macos_app;
mod models;
mod runtime_state;
mod settings_store;
mod storage;
mod utils;
mod wallpaper_manager;

use chrono::{DateTime, Duration as ChronoDuration, Local, TimeZone, Timelike};
use log::{error, info, warn};

use models::{AppSettings, LocalWallpaper};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIcon, TrayIconBuilder, TrayIconEvent},
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
    tray_icon: Arc<Mutex<Option<TrayIcon>>>,
}

// (removed) fetch_bing_images command; image retrieval now handled by background auto-update logic.

// 下载壁纸
// (removed obsolete download_wallpaper command)
/// 按需下载单个壁纸
///
/// 从文件路径中提取 end_date，查找对应的元数据并下载图片
///
/// # Arguments
/// * `file_path` - 壁纸文件路径（例如：/path/to/20251031.jpg）
/// * `wallpaper_dir` - 壁纸存储目录
/// * `app` - Tauri app handle
///
/// # Returns
/// `Ok(())` 如果下载成功或文件已存在，`Err` 如果下载失败
async fn download_wallpaper_if_needed(
    file_path: &Path,
    wallpaper_dir: &Path,
    app: &AppHandle,
) -> Result<(), String> {
    // 如果文件已存在，直接返回
    if file_path.exists() {
        return Ok(());
    }

    // 验证文件路径是否在壁纸目录下（安全性检查）
    // 注意：文件不存在时无法 canonicalize，所以使用父目录检查
    if let Some(parent) = file_path.parent() {
        // 尝试规范化父目录和壁纸目录进行比较
        if let (Ok(parent_can), Ok(dir_can)) = (parent.canonicalize(), wallpaper_dir.canonicalize())
        {
            if !parent_can.starts_with(&dir_can) {
                return Err(format!(
                    "文件路径不在壁纸目录下: {} (期望在: {})",
                    file_path.display(),
                    wallpaper_dir.display()
                ));
            }
        } else {
            // 如果无法规范化，至少检查父目录是否匹配（字符串比较）
            if parent != wallpaper_dir {
                return Err(format!(
                    "文件路径的父目录不匹配: {} (期望: {})",
                    parent.display(),
                    wallpaper_dir.display()
                ));
            }
        }
    } else {
        return Err(format!("无法确定文件路径的父目录: {}", file_path.display()));
    }

    // 从文件路径中提取 end_date（例如：20251031.jpg -> 20251031）
    // 文件名使用 end_date，因为 Bing 的 startdate 是昨天，enddate 才是今天
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "无法从路径中提取文件名".to_string())?;

    let end_date = filename
        .strip_suffix(".jpg")
        .ok_or_else(|| format!("文件名格式不正确，应为 YYYYMMDD.jpg: {}", filename))?;

    // 获取当前语言设置
    let state = app.state::<AppState>();
    let settings = state.settings.lock().await;
    let language = utils::get_bing_market_code(&settings.language);
    drop(settings);

    // 查找对应的壁纸元数据（使用 end_date 作为 key）
    let wallpapers = storage::get_local_wallpapers(wallpaper_dir, language)
        .await
        .map_err(|e| format!("获取壁纸列表失败: {}", e))?;

    let wallpaper = wallpapers
        .iter()
        .find(|w| w.end_date == end_date)
        .ok_or_else(|| format!("未找到 end_date 为 {} 的壁纸元数据", end_date))?;

    // 检查是否有 urlbase（可选字段）
    if wallpaper.urlbase.is_empty() {
        // 如果没有 urlbase，尝试从 Bing API 获取最新数据
        info!(
            target: "commands",
            "壁纸元数据缺少 urlbase，尝试从 API 获取: {}",
            end_date
        );
        // 这里可以添加从 API 获取的逻辑，但为了简化，先返回错误
        return Err(
            "壁纸元数据缺少 urlbase 信息，无法下载。请等待下次更新或手动刷新。".to_string(),
        );
    }

    // 构建完整的图片 URL
    let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, "UHD");

    // 下载图片
    info!(
        target: "commands",
        "开始按需下载壁纸: {} -> {}",
        end_date,
        file_path.display()
    );

    match download_manager::download_image(&image_url, file_path).await {
        Ok(()) => {
            info!(target: "commands", "成功按需下载壁纸: {}", file_path.display());
            // 发送事件通知前端
            let _ = app.emit("image-downloaded", end_date);
            Ok(())
        }
        Err(e) => {
            error!(
                target: "commands",
                "按需下载壁纸失败 {}: {}",
                end_date,
                e
            );
            Err(format!("下载失败: {}", e))
        }
    }
}

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

    // 如果文件不存在，尝试按需下载
    if !path.exists() {
        info!(
            target: "wallpaper",
            "壁纸文件不存在，尝试按需下载: {}",
            path.display()
        );
        if let Err(e) = download_wallpaper_if_needed(&path, &base_dir_can, &app).await {
            return Err(format!("文件不存在且下载失败: {}", e));
        }
    }

    // 再次检查文件是否存在（下载后）
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
        // 如果 urlbase 为空，无法重新下载
        if wallpaper.urlbase.is_empty() {
            warn!(target: "commands", "壁纸缺少 urlbase 信息，无法重新下载: {}", wallpaper.end_date);
            continue;
        }

        // 构建完整的图片 URL
        let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, "UHD");

        // 构建保存路径（使用 end_date，因为文件名使用 end_date）
        let save_path = wallpaper_dir.join(format!("{}.jpg", wallpaper.end_date));

        // 下载图片
        match download_manager::download_image(&image_url, &save_path).await {
            Ok(()) => {
                info!(target: "commands", "成功重新下载壁纸: {}", save_path.display());
                // 发送事件通知前端
                let _ = app.emit("image-downloaded", &wallpaper.end_date);
            }
            Err(e) => {
                error!(target: "commands", "重新下载壁纸失败 {}: {}", wallpaper.end_date, e);
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
    let settings = state.settings.lock().await;

    // 获取当前语言的市场代码
    let language = utils::get_bing_market_code(&settings.language);

    info!(
        target: "commands",
        "获取本地壁纸列表，语言设置: {} -> {}, 目录: {}",
        settings.language,
        language,
        wallpaper_dir.display()
    );

    let wallpapers = storage::get_local_wallpapers(&wallpaper_dir, language)
        .await
        .map_err(|e| {
            error!(target: "commands", "获取本地壁纸列表失败: {}", e);
            e.to_string()
        })?;

    info!(
        target: "commands",
        "成功获取 {} 张本地壁纸（语言: {}）",
        wallpapers.len(),
        language
    );

    // 如果当前语言的索引为空，触发一次更新（异步，不阻塞返回）
    // 但只有在没有更新正在进行时才触发，避免重复更新
    if wallpapers.is_empty() {
        warn!(
            target: "commands",
            "当前语言 ({}) 的壁纸列表为空，将触发异步更新",
            language
        );
        let app_clone = app.clone();
        let language_str = language.to_string();
        tauri::async_runtime::spawn(async move {
            let _ = try_trigger_update_if_empty(&app_clone, &language_str).await;
        });
    }

    // 检查文件是否存在，收集需要重新下载的壁纸
    let mut missing_wallpapers = Vec::new();
    for wallpaper in &wallpapers {
        let path = storage::get_wallpaper_path(&wallpaper_dir, &wallpaper.end_date);
        if !path.exists() {
            warn!(target: "commands", "壁纸文件不存在，将触发重新下载: {}", path.display());
            missing_wallpapers.push(wallpaper.clone());
        }
    }

    // 如果有缺失的文件，异步触发重新下载
    if !missing_wallpapers.is_empty() {
        warn!(
            target: "commands",
            "发现 {} 个缺失的壁纸文件，将触发重新下载",
            missing_wallpapers.len()
        );
        let wallpaper_dir_clone = wallpaper_dir.clone();
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            redownload_missing_wallpapers(missing_wallpapers, wallpaper_dir_clone, app_clone).await;
        });
    }

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

#[tauri::command]
async fn update_settings(
    new_settings: AppSettings,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut settings = state.settings.lock().await;

    // 在更新设置之前，先保存旧的语言设置
    let old_language = settings.language.clone();

    // 只在自启动状态改变时才调用系统 API，避免不必要的系统提示
    let autostart_manager = app.autolaunch();
    let current_autostart_enabled = autostart_manager.is_enabled().unwrap_or(false);

    if new_settings.launch_at_startup != current_autostart_enabled {
        if new_settings.launch_at_startup {
            autostart_manager
                .enable()
                .map_err(|e| format!("启用开机自启动失败: {}", e))?;
        } else {
            autostart_manager
                .disable()
                .map_err(|e| format!("禁用开机自启动失败: {}", e))?;
        }
    }

    *settings = new_settings.clone();
    drop(settings);

    // 更新壁纸目录
    {
        let mut wallpaper_dir = state.wallpaper_directory.lock().await;
        if let Some(ref new_dir) = new_settings.save_directory {
            *wallpaper_dir = PathBuf::from(new_dir);
        } else {
            *wallpaper_dir =
                storage::get_default_wallpaper_directory().map_err(|e| e.to_string())?;
        }
    }

    // 保存设置到 store
    settings_store::save_settings(&app, &new_settings)
        .map_err(|e| format!("保存设置到 store 失败: {}", e))?;

    // 广播设置变化
    state
        .settings_tx
        .send(new_settings.clone())
        .map_err(|e| format!("广播设置失败: {e}"))?;

    // 如果语言设置改变，更新托盘菜单
    if new_settings.language != old_language {
        info!(target: "settings", "语言从 {} 切换到 {}，更新托盘菜单", old_language, new_settings.language);
        // 使用异步方式更新菜单，避免阻塞和 panic
        let app_clone = app.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(e) = update_tray_menu(&app_clone).await {
                error!(target: "settings", "更新托盘菜单失败: {e}");
                warn!(target: "settings", "托盘菜单更新失败，可能需要重启应用");
            } else {
                info!(target: "settings", "托盘菜单更新成功");
            }
        });
    }

    Ok(())
}

#[cfg(test)]
mod lib_tests {}
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
/// 优先从内存状态读取，如果为空则从索引文件读取
#[tauri::command]
async fn get_last_update_time(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    // 优先从内存状态读取
    {
        let guard = state.last_update_time.lock().await;
        if let Some(dt) = *guard {
            return Ok(Some(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
        }
    }

    // 如果内存中没有，尝试从索引文件读取
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    let index_manager = index_manager::IndexManager::new(wallpaper_dir.clone());
    match index_manager.load_index().await {
        Ok(index) => {
            // 从索引文件的 last_updated 字段读取
            let local_time = index.last_updated.with_timezone(&Local);
            Ok(Some(local_time.format("%Y-%m-%d %H:%M:%S").to_string()))
        }
        Err(_) => Ok(None),
    }
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

/// 检查指定语言的索引是否为空，如果为空且没有更新正在进行，则触发强制更新
///
/// 这个函数用于处理首次启动或语言切换时索引为空的情况。
/// 通过先检查索引，再检查更新标志，避免不必要的锁竞争。
/// run_update_cycle_internal 内部有并发保护，会确保只有一个更新真正执行。
///
/// # Arguments
/// * `app` - Tauri app handle
/// * `language` - 语言代码（用于日志和索引检查）
///
/// # Returns
/// `true` 如果成功触发更新，`false` 如果索引不为空或更新已在进行中
async fn try_trigger_update_if_empty(app: &AppHandle, language: &str) -> bool {
    let state = app.state::<AppState>();

    // 先快速检查索引（不需要持有 update_in_progress 锁）
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    let existing_wallpapers = storage::get_local_wallpapers(&wallpaper_dir, language)
        .await
        .unwrap_or_default();

    if !existing_wallpapers.is_empty() {
        // 索引不为空，不需要更新
        return false;
    }

    // 索引为空，检查是否已有更新在进行
    // 注意：这里只检查，不设置标志，让 run_update_cycle_internal 来处理
    // 这样可以避免与 run_update_cycle_internal 内部的并发保护冲突
    let is_updating = {
        let flag = state.update_in_progress.lock().await;
        *flag
    };

    if is_updating {
        info!(
            target: "commands",
            "当前语言 ({}) 的索引为空，但已有更新在进行中，跳过触发",
            language
        );
        return false;
    }

    // 启动更新任务
    // run_update_cycle_internal 内部有并发保护，会确保只有一个更新真正执行
    let app_clone = app.clone();
    let language_clone = language.to_string();

    tauri::async_runtime::spawn(async move {
        info!(
            target: "commands",
            "当前语言 ({}) 的索引为空，触发更新",
            language_clone
        );
        run_update_cycle_internal(&app_clone, true).await;
    });

    true
}

/// 检查索引是否为空，如果为空则触发强制更新（用于启动时）
///
/// 这个函数用于启动时检查索引，确保首次启动时能正确加载数据。
///
/// # Arguments
/// * `app` - Tauri app handle
///
/// # Returns
/// `true` 如果索引为空且需要强制更新，`false` 如果索引不为空
async fn check_and_trigger_update_if_needed(app: &AppHandle) -> bool {
    let state = app.state::<AppState>();

    // 获取当前语言和壁纸目录
    let (wallpaper_dir, language) = {
        let dir = state.wallpaper_directory.lock().await.clone();
        let settings = state.settings.lock().await;
        let lang = utils::get_bing_market_code(&settings.language).to_string();
        (dir, lang)
    };

    let existing_wallpapers = storage::get_local_wallpapers(&wallpaper_dir, &language)
        .await
        .unwrap_or_default();

    if existing_wallpapers.is_empty() {
        info!(target: "auto_update", "启动时检测到索引为空，执行强制更新");
        run_update_cycle_internal(app, true).await;
        true
    } else {
        // 索引不为空，执行常规更新（可能因为智能检查而跳过）
        run_update_cycle(app).await;
        false
    }
}

/// 应用最新壁纸（如果需要）
/// 只有在 auto_update 设置开启时才会自动应用
async fn apply_latest_wallpaper_if_needed(app: &AppHandle, state: &AppState, wallpaper_dir: &Path) {
    // 一次性获取所有需要的设置，减少锁获取次数
    let (should_apply, language) = {
        let settings = state.settings.lock().await;
        (
            settings.auto_update,
            utils::get_bing_market_code(&settings.language).to_string(),
        )
    };

    if !should_apply {
        // 未开启自动应用，跳过
        return;
    }

    let latest_wallpapers = storage::get_local_wallpapers(wallpaper_dir, &language)
        .await
        .unwrap_or_default();
    if let Some(first) = latest_wallpapers.first() {
        let path = storage::get_wallpaper_path(wallpaper_dir, &first.end_date);
        // 检查当前壁纸是否已经是目标壁纸
        let current_path_guard = state.current_wallpaper_path.lock().await;
        let needs_set = current_path_guard
            .as_ref()
            .map(|p| p != &path)
            .unwrap_or(true);
        drop(current_path_guard);

        if needs_set {
            // 如果文件不存在，尝试按需下载
            if !path.exists() {
                info!(
                    target: "update",
                    "最新壁纸文件不存在，尝试按需下载: {}",
                    path.display()
                );
                if let Err(e) = download_wallpaper_if_needed(&path, wallpaper_dir, app).await {
                    error!(target: "update", "按需下载壁纸失败: {e}，跳过设置壁纸");
                    return; // 下载失败，不设置壁纸
                }
            }

            if let Err(e) = wallpaper_manager::set_wallpaper(&path) {
                error!(target: "update", "设置壁纸失败: {e}");
            } else {
                let mut current_path = state.current_wallpaper_path.lock().await;
                *current_path = Some(path);
            }
        }
    }
    // app 参数保留用于未来可能的扩展（如发送事件通知）
    let _ = app;
}

/// 带重试的 Bing 图片获取
async fn fetch_bing_images_with_retry(mkt: &str) -> Option<Vec<models::BingImageEntry>> {
    let mut images_opt = None;
    const MAX_RETRIES: u32 = 10;
    const MAX_BACKOFF_SECS: u64 = 60; // 最大延迟 60 秒

    info!(target: "update", "开始获取 Bing 图片（市场代码: {}, 最大重试次数: {}）", mkt, MAX_RETRIES);

    for attempt in 0..MAX_RETRIES {
        info!(target: "update", "Bing API 请求第 {} 次尝试（共 {} 次）", attempt + 1, MAX_RETRIES);

        match bing_api::fetch_bing_images(8, 0, mkt).await {
            Ok(v) => {
                info!(target: "update", "Bing API 请求成功（第 {} 次尝试）: 获取到 {} 张图片", attempt + 1, v.len());
                images_opt = Some(v);
                break;
            }
            Err(e) => {
                if attempt < MAX_RETRIES - 1 {
                    // 优化：限制最大延迟时间，避免等待时间过长
                    let base_backoff = 1 << attempt; // 指数退避：1, 2, 4, 8, 16, 32, 64, 128, 256, 512
                    let backoff = base_backoff.min(MAX_BACKOFF_SECS); // 限制最大 60 秒
                    warn!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，{}s 后重试",
                        attempt + 1,
                        e,
                        backoff
                    );
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                } else {
                    error!(target: "update",
                        "获取 Bing 图片失败(第 {} 次): {}，已达最大重试次数",
                        attempt + 1,
                        e
                    );
                }
            }
        }
    }

    match &images_opt {
        Some(images) => {
            info!(target: "update", "Bing API 获取完成: 成功获取 {} 张图片", images.len());
        }
        None => {
            error!(target: "update", "Bing API 获取失败: 所有重试均失败");
        }
    }

    images_opt
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

    // 注意：即使 auto_update 关闭，也要获取新壁纸（只获取不自动应用）
    // 自动应用由 apply_latest_wallpaper_if_needed 函数根据 auto_update 设置决定

    let dir = {
        let d = state.wallpaper_directory.lock().await;
        d.clone()
    };

    // 获取语言设置，用于 Bing API 请求和索引存储
    let mkt = utils::get_bing_market_code(&settings_snapshot.language);

    // 优化：在开始时读取一次本地壁纸列表，后续复用
    // 用于判断是否首次启动（首次启动时 existing_wallpapers 为空）
    let existing_wallpapers = storage::get_local_wallpapers(&dir, mkt)
        .await
        .unwrap_or_default();

    // 智能更新检查（非强制更新时）
    if !force_update {
        // 加载运行时状态
        let runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();

        // 优化：API 请求缓存 - 如果距离上次 API 请求不足 5 分钟，且本地有今日壁纸，跳过 API 请求
        if runtime_state::can_skip_api_request(&runtime_state, &dir, mkt).await {
            info!(target: "update", "使用缓存策略跳过 API 请求，直接使用本地壁纸");
            apply_latest_wallpaper_if_needed(app, &state, &dir).await;
            // 重置标志并返回
            let mut flag = state.update_in_progress.lock().await;
            *flag = false;
            return;
        }

        // 检查是否需要更新
        if !runtime_state::should_update_today(&runtime_state) {
            // 今天已经更新过，再检查本地是否真的有今日壁纸
            if runtime_state::has_today_wallpaper(&dir, mkt).await {
                info!(target: "update", "跳过更新：今天已更新且本地有今日壁纸");
                apply_latest_wallpaper_if_needed(app, &state, &dir).await;
                // 启动时跳过更新，不需要通知前端（前端会自己初始化加载）
                // 重置标志并返回
                let mut flag = state.update_in_progress.lock().await;
                *flag = false;
                return;
            }
            info!(target: "update", "今天已更新但本地没有今日壁纸，继续更新");
        }

        // 更新检查时间（在 API 请求之前更新，用于缓存判断）
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

    // 带重试的 Bing 图片获取
    let images = match fetch_bing_images_with_retry(mkt).await {
        Some(v) => v,
        None => {
            error!(target: "update", "多次重试仍失败，跳过本次循环");
            let mut flag = state.update_in_progress.lock().await;
            *flag = false;
            return;
        }
    };

    // 优化：按需下载策略
    // JPG 文件不区分语言，理论上应该一次下载之后不再需要重新下载
    // 包括索引重建、切换语言等场景，只更新元数据，不批量下载图片
    // 图片只在真正需要时（如用户查看、设置壁纸）才按需下载

    // 更新所有壁纸的元数据（包括已存在的图片）
    // 这确保了语言切换时，已存在图片的标题和描述也会更新
    // 首次启动和非首次启动都统一在这里批量保存元数据
    // 注意：保存所有 API 返回的图片的元数据，不管文件是否存在（支持按需下载）
    // 使用 end_date 作为文件名，因为 Bing 的 startdate 是昨天，enddate 才是今天
    // file_path 不再存储，而是根据 end_date 和目录动态生成
    let metadata_list: Vec<LocalWallpaper> = images
        .iter()
        .map(|image| LocalWallpaper::from(image.clone()))
        .collect();

    let is_first_launch = existing_wallpapers.is_empty();
    if !metadata_list.is_empty() {
        let count = metadata_list.len();
        if let Err(e) = storage::save_wallpapers_metadata(metadata_list, &dir, mkt).await {
            if is_first_launch {
                error!(target: "update", "保存元数据失败: {e}");
            } else {
                warn!(target: "update", "更新元数据失败: {e}");
            }
        } else {
            info!(
                target: "update",
                "已{}所有壁纸元数据（{} 条）",
                if is_first_launch { "保存" } else { "更新" },
                count
            );
            // 首次启动时立即通知前端刷新列表
            if is_first_launch {
                if let Err(e) = app.emit("wallpaper-updated", ()) {
                    warn!(target: "update", "通知前端失败: {e}");
                }
                info!(target: "update", "元信息已保存并通知前端，图片将按需下载");
            }
        }
    }

    // 自动应用最新壁纸：检查是否需要设置
    // 优化：重新读取壁纸列表（下载完成后列表可能已更新），但仅在需要设置时检查
    apply_latest_wallpaper_if_needed(app, &state, &dir).await;

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

    // 优化：统一在最后发送一次通知（首次启动时已在533行单独发送）
    // 避免重复通知导致前端不必要的刷新
    if !is_first_launch && let Err(e) = app.emit("wallpaper-updated", ()) {
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
            // 初始立即执行一次更新（强制更新，确保首次启动时能获取数据）
            // 检查索引是否为空，如果为空则强制更新
            check_and_trigger_update_if_needed(&app_clone).await;

            // 标记是否是第一次收到设置变更（启动时的初始化不算）
            let mut is_first_change = true;

            // 小时循环 + 零点对齐
            loop {
                // 计算距下一次本地零点（含 5 分钟缓冲）剩余时间
                let now = Local::now();
                // 安全处理日期计算，提供 fallback 避免 panic
                let tomorrow = now.date_naive().succ_opt().unwrap_or_else(|| {
                    warn!(target: "auto_update", "日期计算失败，使用默认值（明天）");
                    now.date_naive() + ChronoDuration::days(1)
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
                                // 优化：改进的指数退避重试策略，限制最大延迟
                                const MAX_MIDNIGHT_RETRIES: u32 = 10;
                                const MAX_BACKOFF_SECS: u64 = 60; // 最大延迟 60 秒
                                for attempt in 0..MAX_MIDNIGHT_RETRIES {
                                    // 优化：限制最大延迟时间，避免等待时间过长
                                    let base_backoff = 1 << attempt; // 指数退避：1, 2, 4, 8, 16, 32, 64, 128, 256, 512
                                    let backoff = base_backoff.min(MAX_BACKOFF_SECS); // 限制最大 60 秒
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
                                    warn!(target:"auto_update","零点重试结束，仍未成功获取当日壁纸，等待下一轮小时轮询");
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
                            info!(target: "update", "自动应用已关闭（仍会获取新壁纸），等待重新开启...");
                            loop {
                                if rx.changed().await.is_err() { break; }
                                let s = rx.borrow().clone();
                                if s.auto_update {
                                    info!(target: "update", "自动应用重新开启，立即执行一次");
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

/// 根据语言获取托盘菜单文本
fn get_tray_menu_texts(language: &str) -> (&str, &str, &str, &str, &str, &str) {
    match language {
        "zh-CN" => (
            "显示窗口",
            "更新壁纸",
            "打开保存目录",
            "打开设置",
            "关于",
            "退出",
        ),
        "en-US" => (
            "Show Window",
            "Refresh Wallpaper",
            "Open Save Directory",
            "Open Settings",
            "About",
            "Quit",
        ),
        _ => {
            // 自动模式：使用系统语言检测
            let detected_lang = utils::detect_system_language();
            if detected_lang == "zh-CN" {
                (
                    "显示窗口",
                    "更新壁纸",
                    "打开保存目录",
                    "打开设置",
                    "关于",
                    "退出",
                )
            } else {
                (
                    "Show Window",
                    "Refresh Wallpaper",
                    "Open Save Directory",
                    "Open Settings",
                    "About",
                    "Quit",
                )
            }
        }
    }
}

/// 更新托盘菜单（仅更新菜单，不重新创建托盘图标）
async fn update_tray_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    info!(target: "tray", "开始更新托盘菜单");

    // 获取当前托盘图标
    let tray_icon_opt = {
        let state = app.state::<AppState>();
        let tray_icon_guard = state.tray_icon.lock().await;
        tray_icon_guard.clone()
    };

    if let Some(tray) = tray_icon_opt {
        // 获取当前语言设置
        let language = {
            let state = app.state::<AppState>();
            let settings = state.settings.lock().await;
            settings.language.clone()
        };

        info!(target: "tray", "更新托盘菜单，使用语言: {}", language);

        let (show_text, refresh_text, open_folder_text, settings_text, about_text, quit_text) =
            get_tray_menu_texts(&language);

        let show_item = MenuItemBuilder::with_id("show", show_text).build(app)?;
        let refresh_item = MenuItemBuilder::with_id("refresh", refresh_text).build(app)?;
        let open_folder_item =
            MenuItemBuilder::with_id("open_folder", open_folder_text).build(app)?;
        let settings_item = MenuItemBuilder::with_id("settings", settings_text).build(app)?;
        let about_item = MenuItemBuilder::with_id("about", about_text).build(app)?;
        let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;

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

        // 使用 set_menu 直接更新菜单（不重新创建托盘图标）
        // set_menu 需要 Option<M>，其中 M 实现 ContextMenu trait
        tray.set_menu(Some(menu))?;
        info!(target: "tray", "托盘菜单更新成功");
        Ok(())
    } else {
        // 如果托盘图标不存在，创建新的
        warn!(target: "tray", "托盘图标不存在，尝试创建新的");
        setup_tray(app)
    }
}

/// 设置系统托盘（初始创建）
fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    info!(target: "tray", "开始设置托盘菜单");

    // 获取当前语言设置（同步方式，仅在初始化时使用）
    let language = {
        // 尝试从 AppState 获取，如果失败则使用默认值
        if let Some(state) = app.try_state::<AppState>() {
            // 使用 try_lock 避免阻塞，如果失败则使用默认值
            if let Ok(settings) = state.settings.try_lock() {
                settings.language.clone()
            } else {
                "auto".to_string()
            }
        } else {
            "auto".to_string()
        }
    };

    info!(target: "tray", "使用语言: {}", language);

    let (show_text, refresh_text, open_folder_text, settings_text, about_text, quit_text) =
        get_tray_menu_texts(&language);

    let show_item = MenuItemBuilder::with_id("show", show_text).build(app)?;
    let refresh_item = MenuItemBuilder::with_id("refresh", refresh_text).build(app)?;
    let open_folder_item = MenuItemBuilder::with_id("open_folder", open_folder_text).build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", settings_text).build(app)?;
    let about_item = MenuItemBuilder::with_id("about", about_text).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;

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

    info!(target: "tray", "菜单创建完成，正在创建托盘图标");

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
    let icon = app
        .default_window_icon()
        .ok_or_else(|| {
            tauri::Error::InvalidIcon(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Default window icon not found",
            ))
        })?
        .clone();

    let tray = TrayIconBuilder::new()
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

                    // 使用 try_lock 避免阻塞，如果失败则跳过防抖检查
                    if let Ok(mut last_click) = state.last_tray_click.try_lock() {
                        if let Some(last) = *last_click
                            && now.duration_since(last) < Duration::from_millis(300)
                        {
                            return;
                        }
                        *last_click = Some(now);
                    }

                    if let Some(window) = app.get_webview_window("main") {
                        // hide() 可能失败，但失败时忽略错误（窗口可能已经关闭）
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
        .on_menu_event(|app, event| {
            info!(target: "tray", "托盘菜单事件: {}", event.id().as_ref());
            match event.id().as_ref() {
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
                    // 优雅退出应用
                    app.exit(0);
                }
                _ => {
                    warn!(target: "tray", "未知的托盘菜单事件: {}", event.id().as_ref());
                }
            }
        })
        .build(app)?;

    // 保存托盘图标引用到 AppState（使用 try_lock，避免阻塞）
    {
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut tray_icon_guard) = state.tray_icon.try_lock() {
                *tray_icon_guard = Some(tray);
            } else {
                warn!(target: "tray", "无法获取托盘图标锁，托盘图标可能无法保存");
            }
        }
    }

    info!(target: "tray", "托盘菜单设置完成");
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
        tray_icon: Arc::new(Mutex::new(None)),
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
                if let Ok(runtime_state) = runtime_state::load_runtime_state(app.handle())
                    && let Some(ref last_update_str) = runtime_state.last_successful_update
                    && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_update_str)
                {
                    tauri::async_runtime::block_on(async {
                        let mut last_update = state.last_update_time.lock().await;
                        *last_update = Some(dt.with_timezone(&Local));
                    });
                    info!(target: "startup", "从持久化状态恢复上次更新时间: {}", last_update_str);
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
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
