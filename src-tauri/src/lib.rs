mod auto_update;
mod bing_api;
mod commands;
mod download_manager;
mod index_manager;
mod models;
mod runtime_state;
mod settings_store;
mod storage;
mod transfer;
mod tray;
mod update_cycle;
mod utils;
mod version_check;
mod wallpaper_manager;

use chrono::{DateTime, Local};
use log::{info, warn};

use models::{AppRuntimeState, AppSettings};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tauri::{Manager, tray::TrayIcon};
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
    /// Bing API 最近一次返回的实际 mkt（可能与 settings.mkt 不同）
    ///
    /// 当中国 Bing 强制返回 zh-CN 时，此字段会存储 "zh-CN"，
    /// 确保后续读取壁纸时使用与写入一致的 mkt key。
    /// 用户更改 mkt 设置时应清空此字段。
    last_actual_mkt: Arc<Mutex<Option<String>>>,
}

// (removed) fetch_bing_images command; image retrieval now handled by background auto-update logic.

/// 获取有效的 mkt（用于读取壁纸索引）
///
/// 委托给 `utils::effective_mkt`，从 AppState 中提取所需参数。
pub(crate) async fn get_effective_mkt(state: &AppState) -> String {
    let last_actual = state.last_actual_mkt.lock().await.clone();
    let settings_mkt = state.settings.lock().await.mkt.clone();
    utils::effective_mkt(last_actual.as_deref(), &settings_mkt)
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
        last_actual_mkt: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 当检测到第二个实例启动时，将第一个实例的窗口显示出来
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
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
        .plugin(tauri_plugin_process::init())
        .plugin({
            #[allow(unused_mut)]
            let mut updater_builder = tauri_plugin_updater::Builder::new();

            // Debug 构建：支持 DEV_OVERRIDE_VERSION 环境变量覆盖当前版本号
            #[cfg(debug_assertions)]
            {
                if let Ok(dev_version) = std::env::var("DEV_OVERRIDE_VERSION") {
                    if let Ok(dev_ver) = semver::Version::parse(&dev_version) {
                        info!(target: "updater", "DEV_OVERRIDE_VERSION={}, using custom version comparator", dev_version);
                        updater_builder = updater_builder
                            .default_version_comparator(move |_current, remote| remote.version > dev_ver);
                    } else {
                        warn!(target: "updater", "DEV_OVERRIDE_VERSION={} is not valid semver, ignoring", dev_version);
                    }
                }
            }

            updater_builder.build()
        })
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
            commands::wallpaper::set_desktop_wallpaper,
            commands::wallpaper::get_local_wallpapers,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::storage::get_wallpaper_directory,
            commands::storage::get_default_wallpaper_directory,
            commands::storage::get_last_update_time,
            commands::storage::get_update_in_progress,
            commands::storage::ensure_wallpaper_directory_exists,
            commands::window::show_main_window,
            update_cycle::force_update,
            version_check::add_ignored_update_version,
            version_check::is_version_ignored,
            commands::window::get_screen_orientations,
            commands::mkt::get_market_status,
            commands::mkt::get_supported_mkts,
            transfer::import_wallpapers,
            transfer::export_wallpapers,
        ])
        .setup(|app| {
            wallpaper_manager::initialize_observer();

            // macOS: Info.plist 的 LSUIElement=true 不足以在所有场景下阻止
            // Dock 运行状态点出现，运行时补充设置 Accessory 模式作为双重保障。
            #[cfg(target_os = "macos")]
            {
                use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
                use objc2_foundation::MainThreadMarker;
                if let Some(mtm) = MainThreadMarker::new() {
                    let ns_app = NSApplication::sharedApplication(mtm);
                    ns_app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
                } else {
                    warn!(target: "startup", "setup 未在主线程执行，跳过 setActivationPolicy");
                }
            }

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

                // 从持久化 runtime_state 恢复 last_actual_mkt
                // 解决重启后因 settings.mkt 与 index.json 中实际 key 不一致导致的短暂空白
                if let Some(ref actual_mkt) = runtime_state.last_actual_mkt {
                    tauri::async_runtime::block_on(async {
                        *state.last_actual_mkt.lock().await = Some(actual_mkt.clone());
                    });
                    info!(target: "startup", "从持久化状态恢复 last_actual_mkt: {}", actual_mkt);
                }
            }

            tray::setup_tray(app.handle())?;

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
            auto_update::start_auto_update_task(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();

                #[cfg(target_os = "macos")]
                {
                    use std::time::Duration;
                    // macOS fullscreen exit animation takes ~500ms;
                    // hiding before it completes causes visual glitches.
                    const MACOS_FULLSCREEN_EXIT_DELAY_MS: u64 = 700;

                    if window.is_fullscreen().unwrap_or(false) {
                        let _ = window.set_fullscreen(false);
                        let app_handle = window.app_handle().clone();
                        let label = window.label().to_string();
                        tauri::async_runtime::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(
                                MACOS_FULLSCREEN_EXIT_DELAY_MS,
                            ))
                            .await;
                            if let Some(win) = app_handle.get_webview_window(&label) {
                                let _ = win.hide();
                            }
                        });
                        return;
                    }
                }

                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
