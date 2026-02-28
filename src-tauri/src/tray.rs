use crate::{AppState, utils, version_check};
use log::{info, warn};
use std::time::{Duration, Instant};
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};

/// 根据 resolved_language 获取托盘菜单文本
///
/// 传入值应为 "zh-CN" 或 "en-US"（已在设置加载时归一化）
fn get_tray_menu_texts(resolved_language: &str) -> (&str, &str, &str, &str, &str, &str, &str) {
    if resolved_language == "zh-CN" {
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

/// 更新托盘菜单（仅更新菜单，不重新创建托盘图标）
pub(crate) async fn update_tray_menu(app: &AppHandle) -> tauri::Result<()> {
    info!(target: "tray", "开始更新托盘菜单");

    // 获取当前托盘图标
    let tray_icon_opt = {
        let state = app.state::<AppState>();
        let tray_icon_guard = state.tray_icon.lock().await;
        tray_icon_guard.clone()
    };

    if let Some(tray) = tray_icon_opt {
        // 获取 resolved_language（已归一化为 "zh-CN" 或 "en-US"）
        let language = {
            let state = app.state::<AppState>();
            let settings = state.settings.lock().await;
            settings.resolved_language.clone()
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
pub(crate) fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    info!(target: "tray", "开始设置托盘菜单");

    // 获取 resolved_language（同步方式，仅在初始化时使用）
    let language = {
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(settings) = state.settings.try_lock() {
                if settings.resolved_language.is_empty() {
                    // resolved_language 未计算时（理论上不应发生），回退到系统检测
                    utils::detect_system_language().to_string()
                } else {
                    settings.resolved_language.clone()
                }
            } else {
                utils::detect_system_language().to_string()
            }
        } else {
            utils::detect_system_language().to_string()
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
                        crate::update_cycle::run_update_cycle_internal(&app_handle, true).await;
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
                        match version_check::check_for_updates().await {
                            Ok(result) => {
                                // 如果有更新且平台安装包可用，通知前端显示更新对话框
                                if result.has_update
                                    && result.latest_version.is_some()
                                    && result.release_url.is_some()
                                    && result.platform_available
                                {
                                    // 检查该版本是否已被用户忽略
                                    let is_ignored = if let Some(version) = &result.latest_version {
                                        version_check::is_version_ignored(app_handle.clone(), version.clone())
                                            .await
                                            .unwrap_or(false)
                                    } else {
                                        false
                                    };

                                    if !is_ignored {
                                        // 手动检查到更新，先弹出主窗口
                                        if let Some(window) = app_handle.get_webview_window("main") {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }
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
