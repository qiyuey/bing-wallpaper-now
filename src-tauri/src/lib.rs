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

use models::{AppRuntimeState, AppSettings, LocalWallpaper};
use serde::{Deserialize, Serialize};
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
fn set_autostart_notification_flag_if_needed(app: &AppHandle, log_target: &str) {
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

    // 从文件路径中提取 end_date（例如：20251031.jpg 或 20251031r.jpg -> 20251031）
    // 文件名使用 end_date，因为 Bing 的 startdate 是昨天，enddate 才是今天
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "无法从路径中提取文件名".to_string())?;

    // 检查是否为竖屏壁纸（r.jpg 后缀）
    let is_portrait = filename.ends_with("r.jpg");
    let end_date = if is_portrait {
        filename
            .strip_suffix("r.jpg")
            .ok_or_else(|| format!("文件名格式不正确，应为 YYYYMMDDr.jpg: {}", filename))?
    } else {
        filename
            .strip_suffix(".jpg")
            .ok_or_else(|| format!("文件名格式不正确，应为 YYYYMMDD.jpg: {}", filename))?
    };

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
    // 竖屏使用 1080x1920，横屏使用 UHD
    let resolution = if is_portrait { "1080x1920" } else { "UHD" };
    let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, resolution);

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
    // 获取当前语言和壁纸目录，用于记录手动设置时的最新壁纸
    let (language, wallpaper_dir_for_record) = {
        let settings = state.settings.lock().await;
        let lang = utils::get_bing_market_code(&settings.language).to_string();
        drop(settings);
        let dir = state.wallpaper_directory.lock().await.clone();
        (lang, dir)
    };
    // 从文件路径中提取 end_date（例如：20251031.jpg -> 20251031）
    let set_end_date = path
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|filename| filename.strip_suffix(".jpg"))
        .map(|s| s.to_string());

    tauri::async_runtime::spawn(async move {
        // 检测屏幕方向，获取竖屏壁纸路径
        let screen_orientations = wallpaper_manager::get_screen_orientations();
        let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);

        // 从横屏路径生成竖屏路径（例如：20251031.jpg -> 20251031r.jpg）
        let base_dir = target_for_spawn.parent().unwrap_or(Path::new(""));
        let portrait_file = target_for_spawn
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| base_dir.join(format!("{}r.jpg", s)));

        let mut portrait_path = None;

        // 处理竖屏壁纸：如果存在竖屏显示器，检查并下载竖屏壁纸
        if has_portrait_screen && let Some(ref portrait_file_path) = portrait_file {
            if portrait_file_path.exists() {
                // 竖屏壁纸已存在
                portrait_path = Some(portrait_file_path.clone());
            } else {
                // 如果竖屏壁纸不存在，尝试按需下载
                info!(
                    target: "wallpaper",
                    "竖屏壁纸文件不存在，尝试按需下载: {}",
                    portrait_file_path.display()
                );
                // 从文件路径中提取 end_date
                let end_date = target_for_spawn
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_string());

                if let Some(_end_date) = end_date {
                    // 获取壁纸元数据并下载
                    let wallpaper_dir = base_dir.to_path_buf();
                    info!(
                        target: "wallpaper",
                        "开始下载竖屏壁纸: {}",
                        portrait_file_path.display()
                    );
                    if let Err(e) =
                        download_wallpaper_if_needed(portrait_file_path, &wallpaper_dir, &app_clone)
                            .await
                    {
                        warn!(target: "wallpaper", "按需下载竖屏壁纸失败: {e}，将仅设置横屏壁纸");
                    } else {
                        // 下载成功后，使用竖屏壁纸
                        if portrait_file_path.exists() {
                            info!(
                                target: "wallpaper",
                                "竖屏壁纸下载成功，将使用竖屏壁纸: {}",
                                portrait_file_path.display()
                            );
                            portrait_path = Some(portrait_file_path.clone());
                        } else {
                            warn!(target: "wallpaper", "竖屏壁纸下载完成但文件不存在，将仅设置横屏壁纸");
                        }
                    }
                }
            }
        }

        if let Err(e) =
            wallpaper_manager::set_wallpaper(&target_for_spawn, portrait_path.as_deref())
        {
            error!(target: "wallpaper", "设置壁纸失败: {e}");
        } else {
            let state_clone = app_clone.state::<AppState>();
            let mut current_path = state_clone.current_wallpaper_path.lock().await;
            *current_path = Some(target_for_spawn.clone());
            drop(current_path);

            // 记录用户手动设置时的最新壁纸（按语言隔离）
            // 获取当前语言的最新壁纸的 end_date，记录到运行时状态
            if let Some(set_end_date) = set_end_date
                && let Ok(latest_wallpapers) =
                    storage::get_local_wallpapers(&wallpaper_dir_for_record, &language).await
                && let Some(latest) = latest_wallpapers.first()
            {
                let mut runtime_state =
                    runtime_state::load_runtime_state(&app_clone).unwrap_or_default();
                runtime_state
                    .manually_set_latest_wallpapers
                    .insert(language.clone(), latest.end_date.clone());
                if let Err(e) = runtime_state::save_runtime_state(&app_clone, &runtime_state) {
                    warn!(target: "wallpaper", "保存手动设置记录失败: {e}");
                } else {
                    info!(target: "wallpaper", 
                        "已记录用户手动设置时的最新壁纸：语言={}, 设置壁纸={}, 当时最新壁纸={}", 
                        language, set_end_date, latest.end_date);
                }
            }
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
    let current_autostart_enabled = autostart_manager.is_enabled().unwrap_or_else(|e| {
        warn!(target: "settings", "读取当前自启动状态失败: {}，假设为未启用", e);
        false
    });

    if new_settings.launch_at_startup != current_autostart_enabled {
        if new_settings.launch_at_startup {
            autostart_manager
                .enable()
                .map_err(|e| format!("启用开机自启动失败: {}", e))?;

            // 记录用户已启用自启动，macOS 系统会显示通知
            // 通过这个标志，我们可以知道用户已经看到过系统通知
            set_autostart_notification_flag_if_needed(&app, "settings");
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
mod lib_tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        // 基本版本比较
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert!(compare_versions("1.0.1", "1.0.0") > 0);

        // 不同长度的版本号
        assert_eq!(compare_versions("1.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert!(compare_versions("1.0.1", "1.0") > 0);

        // 主要版本号差异
        assert!(compare_versions("0.9.9", "1.0.0") < 0);
        assert!(compare_versions("1.0.0", "2.0.0") < 0);

        // 次要版本号差异
        assert!(compare_versions("1.0.0", "1.1.0") < 0);
        assert!(compare_versions("1.1.0", "1.0.0") > 0);

        // 无效版本号（应该被解析为 0）
        assert_eq!(compare_versions("invalid", "0.0.0"), 0);
        assert_eq!(compare_versions("1.0.invalid", "1.0.0"), 0);
    }

    #[test]
    fn test_has_platform_asset() {
        #[cfg(target_os = "windows")]
        {
            let assets = vec![
                GitHubAsset {
                    name: "Bing.Wallpaper.Now_0.4.6_x64_zh-CN.msi".to_string(),
                    _browser_download_url: "https://example.com/test.msi".to_string(),
                },
                GitHubAsset {
                    name: "Bing.Wallpaper.Now_0.4.6_x64-setup.exe".to_string(),
                    _browser_download_url: "https://example.com/test.exe".to_string(),
                },
                GitHubAsset {
                    name: "test.dmg".to_string(),
                    _browser_download_url: "https://example.com/test.dmg".to_string(),
                },
            ];
            assert!(has_platform_asset(&assets));

            // 测试空列表
            assert!(!has_platform_asset(&[]));
        }

        #[cfg(target_os = "macos")]
        {
            let assets = vec![GitHubAsset {
                name: "Bing.Wallpaper.Now_0.4.6_aarch64.dmg".to_string(),
                _browser_download_url: "https://example.com/test.dmg".to_string(),
            }];
            assert!(has_platform_asset(&assets));

            let assets_false = vec![GitHubAsset {
                name: "test.msi".to_string(),
                _browser_download_url: "https://example.com/test.msi".to_string(),
            }];
            assert!(!has_platform_asset(&assets_false));
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let assets = vec![GitHubAsset {
                name: "bing-wallpaper-now_0.4.6_amd64.deb".to_string(),
                _browser_download_url: "https://example.com/test.deb".to_string(),
            }];
            assert!(has_platform_asset(&assets));

            let assets_false = vec![GitHubAsset {
                name: "test.msi".to_string(),
                _browser_download_url: "https://example.com/test.msi".to_string(),
            }];
            assert!(!has_platform_asset(&assets_false));
        }
    }
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

/// 获取所有屏幕的方向信息
#[tauri::command]
async fn get_screen_orientations() -> Result<Vec<wallpaper_manager::ScreenOrientation>, String> {
    Ok(wallpaper_manager::get_screen_orientations())
}

/// GitHub Releases API 响应结构
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub Release Asset 结构
#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    #[serde(rename = "browser_download_url", skip)]
    _browser_download_url: String,
}

/// 版本检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionCheckResult {
    current_version: String,
    latest_version: Option<String>,
    has_update: bool,
    release_url: Option<String>,
    platform_available: bool,
}

/// 添加版本到"不再提醒"列表（保存最大版本）
#[tauri::command]
async fn add_ignored_update_version(app: AppHandle, version: String) -> Result<(), String> {
    let mut runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    // 如果当前忽略的版本为空，或者新版本更大，则更新
    let should_update = runtime_state
        .ignored_update_version
        .as_ref()
        .map(|ignored| compare_versions(ignored, &version) < 0)
        .unwrap_or(true);

    if should_update {
        runtime_state.ignored_update_version = Some(version.clone());
        runtime_state::save_runtime_state(&app, &runtime_state)
            .map_err(|e| format!("Failed to save runtime state: {}", e))?;
        info!(
            target: "version_check",
            "Updated ignored update version to: {}",
            version
        );
    }

    Ok(())
}

/// 检查版本是否应该被忽略（版本小于等于忽略的版本）
#[tauri::command]
async fn is_version_ignored(app: AppHandle, version: String) -> Result<bool, String> {
    let runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    match runtime_state.ignored_update_version {
        Some(ref ignored_version) => {
            // 如果当前版本小于等于忽略的版本，则忽略
            Ok(compare_versions(&version, ignored_version) <= 0)
        }
        None => Ok(false),
    }
}

/// 检查 GitHub Releases 是否有新版本
///
/// # Returns
/// 返回版本检查结果，包含当前版本、最新版本和是否有更新
#[tauri::command]
async fn check_for_updates() -> Result<VersionCheckResult, String> {
    const GITHUB_API_URL: &str =
        "https://api.github.com/repos/qiyuey/bing-wallpaper-now/releases/latest";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    // 移除开发版本后缀（例如：0.4.5-0 -> 0.4.5）
    let current_version = CURRENT_VERSION
        .split('-')
        .next()
        .unwrap_or(CURRENT_VERSION)
        .to_string();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Bing-Wallpaper-Now/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    match client.get(GITHUB_API_URL).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GitHubRelease>().await {
                    Ok(release) => {
                        // 移除 tag_name 中的 'v' 前缀（如果有）
                        let latest_version = release.tag_name.trim_start_matches('v').to_string();

                        // 检查是否有当前平台的安装包
                        let platform_available = has_platform_asset(&release.assets);

                        // 比较版本号（简单字符串比较，对于语义化版本号足够）
                        // 只有当平台安装包可用时才认为有更新
                        let has_update = platform_available
                            && compare_versions(&current_version, &latest_version) < 0;

                        info!(
                            target: "version_check",
                            "Version check completed: current={}, latest={}, has_update={}, platform_available={}",
                            current_version,
                            latest_version,
                            has_update,
                            platform_available
                        );

                        Ok(VersionCheckResult {
                            current_version,
                            latest_version: Some(latest_version),
                            has_update,
                            release_url: Some(release.html_url),
                            platform_available,
                        })
                    }
                    Err(e) => {
                        warn!(target: "version_check", "Failed to parse GitHub release response: {}", e);
                        Ok(VersionCheckResult {
                            current_version,
                            latest_version: None,
                            has_update: false,
                            release_url: None,
                            platform_available: false,
                        })
                    }
                }
            } else {
                warn!(
                    target: "version_check",
                    "GitHub API returned status: {}",
                    response.status()
                );
                Ok(VersionCheckResult {
                    current_version,
                    latest_version: None,
                    has_update: false,
                    release_url: None,
                    platform_available: false,
                })
            }
        }
        Err(e) => {
            warn!(target: "version_check", "Failed to check for updates: {}", e);
            Ok(VersionCheckResult {
                current_version,
                latest_version: None,
                has_update: false,
                release_url: None,
                platform_available: false,
            })
        }
    }
}

/// 获取当前平台应该使用的安装包文件扩展名
///
/// # Returns
/// 返回当前平台的安装包文件扩展名列表
fn get_platform_extensions() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        vec![".msi", ".exe"]
    }
    #[cfg(target_os = "macos")]
    {
        vec![".dmg"]
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        vec![".deb", ".rpm", ".AppImage"]
    }
}

/// 检查 assets 中是否有当前平台的安装包
fn has_platform_asset(assets: &[GitHubAsset]) -> bool {
    let extensions = get_platform_extensions();
    assets.iter().any(|asset| {
        extensions.iter().any(|ext| {
            // 检查文件名是否以扩展名结尾
            // 扩展名本身以 '.' 开头（如 ".dmg"），所以如果文件名以扩展名结尾，就已经有正确的分隔符了
            asset.name.ends_with(ext)
        })
    })
}

/// 比较两个版本号字符串
///
/// # Returns
/// - 负数：如果 version1 < version2
/// - 0：如果 version1 == version2
/// - 正数：如果 version1 > version2
fn compare_versions(version1: &str, version2: &str) -> i32 {
    let v1_parts: Vec<u32> = version1
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let v2_parts: Vec<u32> = version2
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    let max_len = v1_parts.len().max(v2_parts.len());

    for i in 0..max_len {
        let v1_part = v1_parts.get(i).copied().unwrap_or(0);
        let v2_part = v2_parts.get(i).copied().unwrap_or(0);

        match v1_part.cmp(&v2_part) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }

    0
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
        // 检查用户是否手动设置过壁纸，且当前最新壁纸和手动设置时的最新壁纸相同
        let runtime_state = runtime_state::load_runtime_state(app).unwrap_or_default();
        if runtime_state
            .manually_set_latest_wallpapers
            .get(&language)
            .is_some_and(|manually_set_end_date| manually_set_end_date == &first.end_date)
        {
            info!(
                target: "update",
                "跳过自动应用：当前语言 ({}) 的最新壁纸 ({}) 和用户手动设置时的最新壁纸相同",
                language,
                first.end_date
            );
            return;
        }

        let path = storage::get_wallpaper_path(wallpaper_dir, &first.end_date);

        // 检测屏幕方向，获取竖屏壁纸路径
        let screen_orientations = wallpaper_manager::get_screen_orientations();
        let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);
        let portrait_path = if has_portrait_screen {
            let portrait_file = wallpaper_dir.join(format!("{}r.jpg", first.end_date));
            if portrait_file.exists() {
                Some(portrait_file)
            } else {
                None
            }
        } else {
            None
        };

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

            // 如果竖屏壁纸不存在，尝试按需下载
            if let Some(ref portrait_file) = portrait_path
                && !portrait_file.exists()
            {
                info!(
                    target: "update",
                    "竖屏壁纸文件不存在，尝试按需下载: {}",
                    portrait_file.display()
                );
                if let Err(e) =
                    download_wallpaper_if_needed(portrait_file, wallpaper_dir, app).await
                {
                    warn!(target: "update", "按需下载竖屏壁纸失败: {e}，将仅设置横屏壁纸");
                }
            }

            if let Err(e) = wallpaper_manager::set_wallpaper(&path, portrait_path.as_deref()) {
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
    const MAX_RETRIES: u32 = 3;
    const MAX_BACKOFF_SECS: u64 = 16; // 最大延迟 16 秒

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
                    let base_backoff = 1 << attempt; // 指数退避：1, 2, 4
                    let backoff = base_backoff.min(MAX_BACKOFF_SECS); // 限制最大 16 秒
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

    // 在 metadata_list 被移动之前先克隆需要的数据（用于竖屏壁纸下载）
    let screen_orientations = wallpaper_manager::get_screen_orientations();
    let has_portrait_screen = screen_orientations.iter().any(|s| s.is_portrait);
    let latest_wallpaper_for_portrait = if has_portrait_screen && !metadata_list.is_empty() {
        Some(metadata_list[0].clone())
    } else {
        None
    };

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

    // 如果有竖屏显示器，异步下载竖屏壁纸
    if let Some(ref latest_wallpaper) = latest_wallpaper_for_portrait
        && !latest_wallpaper.urlbase.is_empty()
    {
        let portrait_file_path = dir.join(format!("{}r.jpg", latest_wallpaper.end_date));

        // 如果竖屏壁纸不存在，则下载
        if !portrait_file_path.exists() {
            let portrait_url = bing_api::get_wallpaper_url(&latest_wallpaper.urlbase, "1080x1920");
            let end_date = latest_wallpaper.end_date.clone();
            info!(
                target: "update",
                "检测到竖屏显示器，开始下载竖屏壁纸: {}",
                portrait_file_path.display()
            );

            // 异步下载，不阻塞主流程
            let app_clone = app.clone();
            let portrait_path_clone = portrait_file_path.clone();
            tauri::async_runtime::spawn(async move {
                match download_manager::download_image(&portrait_url, &portrait_path_clone).await {
                    Ok(()) => {
                        info!(
                            target: "update",
                            "竖屏壁纸下载成功: {}",
                            portrait_path_clone.display()
                        );
                        let _ = app_clone.emit("image-downloaded", end_date);
                    }
                    Err(e) => {
                        error!(
                            target: "update",
                            "竖屏壁纸下载失败: {}",
                            e
                        );
                    }
                }
            });
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
fn get_tray_menu_texts(language: &str) -> (&str, &str, &str, &str, &str, &str, &str) {
    match language {
        "zh-CN" => (
            "显示窗口",
            "更新壁纸",
            "打开保存目录",
            "打开设置",
            "关于",
            "检查更新",
            "退出",
        ),
        "en-US" => (
            "Show Window",
            "Refresh Wallpaper",
            "Open Save Directory",
            "Open Settings",
            "About",
            "Check for Updates",
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
                    "检查更新",
                    "退出",
                )
            } else {
                (
                    "Show Window",
                    "Refresh Wallpaper",
                    "Open Save Directory",
                    "Open Settings",
                    "About",
                    "Check for Updates",
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

        let (
            show_text,
            refresh_text,
            open_folder_text,
            settings_text,
            about_text,
            check_updates_text,
            quit_text,
        ) = get_tray_menu_texts(&language);

        let show_item = MenuItemBuilder::with_id("show", show_text).build(app)?;
        let refresh_item = MenuItemBuilder::with_id("refresh", refresh_text).build(app)?;
        let open_folder_item =
            MenuItemBuilder::with_id("open_folder", open_folder_text).build(app)?;
        let settings_item = MenuItemBuilder::with_id("settings", settings_text).build(app)?;
        let about_item = MenuItemBuilder::with_id("about", about_text).build(app)?;
        let check_updates_item =
            MenuItemBuilder::with_id("check_updates", check_updates_text).build(app)?;
        let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;

        let menu = MenuBuilder::new(app)
            .item(&show_item)
            .separator()
            .item(&refresh_item)
            .item(&open_folder_item)
            .item(&settings_item)
            .item(&check_updates_item)
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

    let (
        show_text,
        refresh_text,
        open_folder_text,
        settings_text,
        about_text,
        check_updates_text,
        quit_text,
    ) = get_tray_menu_texts(&language);

    let show_item = MenuItemBuilder::with_id("show", show_text).build(app)?;
    let refresh_item = MenuItemBuilder::with_id("refresh", refresh_text).build(app)?;
    let open_folder_item = MenuItemBuilder::with_id("open_folder", open_folder_text).build(app)?;
    let settings_item = MenuItemBuilder::with_id("settings", settings_text).build(app)?;
    let about_item = MenuItemBuilder::with_id("about", about_text).build(app)?;
    let check_updates_item =
        MenuItemBuilder::with_id("check_updates", check_updates_text).build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", quit_text).build(app)?;

    let menu = MenuBuilder::new(app)
        .item(&show_item)
        .separator()
        .item(&refresh_item)
        .item(&open_folder_item)
        .item(&settings_item)
        .item(&check_updates_item)
        .item(&about_item)
        .separator()
        .item(&quit_item)
        .build()?;

    info!(target: "tray", "菜单创建完成，正在创建托盘图标");

    // Windows 高 DPI 下托盘图标优化：使用完整大小的托盘图标
    // 使用 tray-icon-windows.png（从 icon-windows.svg 生成，无缩放）
    // 在 200% 缩放下，128x128 的图标可以提供清晰的显示效果（等效 64x64 物理像素）
    #[cfg(target_os = "windows")]
    let icon = {
        let icon_bytes = include_bytes!("../icons/tray-icon-windows.png");
        let icon_img = image::load_from_memory(icon_bytes)
            .map_err(|e| {
                tauri::Error::InvalidIcon(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
            })?
            .to_rgba8();
        tauri::image::Image::new_owned(icon_img.to_vec(), icon_img.width(), icon_img.height())
    };

    // macOS 使用黑白托盘图标（符合系统设计规范）
    // 图标应为黑色和透明，系统会根据深色/浅色模式自动调整颜色
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

    let tray_builder = {
        let builder = TrayIconBuilder::new()
            .menu(&menu)
            .icon(icon)
            .tooltip("Bing Wallpaper Now")
            .show_menu_on_left_click(false);

        // macOS 设置模板图标以支持深色/浅色模式自动切换
        #[cfg(target_os = "macos")]
        {
            builder.icon_as_template(true)
        }
        #[cfg(not(target_os = "macos"))]
        {
            builder
        }
    };

    let tray = tray_builder
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
                "check_updates" => {
                    // 手动触发更新检查（仅托盘菜单触发，自动检查不会进入这里）
                    // 注意：自动检查更新通过前端直接调用 check_for_updates 命令实现，
                    // 不会触发此事件处理，因此不会显示 toast
                    let app_handle = app.clone();
                    tauri::async_runtime::spawn(async move {
                        match check_for_updates().await {
                            Ok(result) => {
                                // 如果有更新且平台安装包可用，通知前端显示更新对话框
                                if result.has_update
                                    && result.latest_version.is_some()
                                    && result.release_url.is_some()
                                    && result.platform_available
                                {
                                    // 检查该版本是否已被用户忽略
                                    let is_ignored = if let Some(version) = &result.latest_version {
                                        is_version_ignored(app_handle.clone(), version.clone())
                                            .await
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    if !is_ignored {
                                        // 通过事件通知前端显示更新对话框
                                        if let Err(e) = app_handle.emit("check-updates-result", result) {
                                            warn!(target: "tray", "Failed to emit check-updates-result event: {}", e);
                                        }
                                    }
                                } else {
                                    // 手动检查时没有更新，发送事件通知前端显示 toast
                                    // 自动检查不会触发此事件，因此不会显示 toast
                                    if let Err(e) = app_handle.emit("check-updates-no-update", ()) {
                                        warn!(target: "tray", "Failed to emit check-updates-no-update event: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(target: "version_check", "手动检查更新失败: {}", e);
                            }
                        }
                    });
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
                .max_file_size(10_000_000) // 10MB
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepOne)
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
            check_for_updates,
            add_ignored_update_version,
            is_version_ignored,
            get_screen_orientations,
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

            // 从操作系统读取真实的自启动状态，并更新应用设置
            // 这样即使用户手动在系统设置中修改了自启动状态，应用也能获取到准确的值
            // 同时一次性加载 runtime_state，避免重复加载
            {
                let autostart_manager = app.handle().autolaunch();
                let system_autostart_enabled = autostart_manager.is_enabled().unwrap_or_else(|e| {
                    warn!(target: "startup", "读取系统自启动状态失败: {}，假设为未启用", e);
                    false
                });
                let settings_autostart_enabled = loaded_settings.launch_at_startup;

                // 一次性加载 runtime_state，后续复用
                let mut runtime_state = match runtime_state::load_runtime_state(app.handle()) {
                    Ok(state) => state,
                    Err(e) => {
                        warn!(target: "startup", "加载运行时状态失败: {}，使用默认值", e);
                        AppRuntimeState::default()
                    }
                };

                // 如果系统实际状态与设置不一致，更新设置以匹配系统状态
                // 这可能是因为用户手动在系统设置中修改了自启动状态
                if system_autostart_enabled != settings_autostart_enabled {
                    info!(target: "startup",
                        "检测到自启动状态不一致（设置: {}，系统: {}），更新设置为系统实际状态",
                        settings_autostart_enabled, system_autostart_enabled);

                    // 更新内存中的设置
                    tauri::async_runtime::block_on(async {
                        let mut settings = state.settings.lock().await;
                        settings.launch_at_startup = system_autostart_enabled;
                    });

                    // 更新持久化设置
                    let mut updated_settings = loaded_settings.clone();
                    updated_settings.launch_at_startup = system_autostart_enabled;
                    if let Err(e) = settings_store::save_settings(app.handle(), &updated_settings) {
                        warn!(target: "startup", "保存同步后的设置失败: {}", e);
                    } else {
                        // 同步更新后的设置到 watch channel
                        // 这样 auto_update_task 等监听者能获取到正确的自启动状态
                        if let Err(e) = state.settings_tx.send(updated_settings.clone()) {
                            warn!(target: "startup", "发送同步后的设置到 watch channel 失败: {}", e);
                        }
                    }
                }

                // 如果自启动已启用，但通知标志未设置，则自动设置标志
                // 这适用于在更新到 0.4.10 之前就已经启用自启动的用户
                if system_autostart_enabled && !runtime_state.autostart_notification_shown {
                    runtime_state.autostart_notification_shown = true;
                    if let Err(e) = runtime_state::save_runtime_state(app.handle(), &runtime_state) {
                        warn!(target: "startup", "保存自启动通知标志失败: {}", e);
                    } else {
                        info!(target: "startup", "检测到自启动已启用但通知标志未设置，已自动设置标志");
                    }
                }

                // 使用已加载的 runtime_state 恢复上次更新时间
                if let Some(ref last_update_str) = runtime_state.last_successful_update
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
            // 日志文件超过 10MB 时自动轮转，保留所有历史日志文件
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
