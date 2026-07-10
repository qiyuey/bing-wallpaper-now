use crate::{AppState, utils};
use log::{info, warn};
use std::time::{Duration, Instant};
#[cfg(target_os = "windows")]
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use tauri::{
    AppHandle, Emitter, Manager,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::ERROR_SUCCESS,
    System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_NOTIFY, KEY_QUERY_VALUE, REG_NOTIFY_CHANGE_LAST_SET,
        RRF_RT_REG_DWORD, RegCloseKey, RegGetValueW, RegNotifyChangeKeyValue, RegOpenKeyExW,
    },
};

#[cfg(target_os = "windows")]
const WINDOWS_TRAY_ICON_LIGHT: &[u8] = include_bytes!("../icons/tray-icon-windows-light.png");
#[cfg(target_os = "windows")]
const WINDOWS_TRAY_ICON_DARK: &[u8] = include_bytes!("../icons/tray-icon-windows-dark.png");
#[cfg(target_os = "windows")]
const WINDOWS_PERSONALIZE_KEY: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
#[cfg(target_os = "windows")]
const WINDOWS_SYSTEM_THEME_VALUE: &str = "SystemUsesLightTheme";
#[cfg(target_os = "windows")]
static WINDOWS_THEME_WATCHER_STARTED: AtomicBool = AtomicBool::new(false);

fn load_tray_image(icon_bytes: &[u8]) -> tauri::Result<Image<'static>> {
    let icon_img = image::load_from_memory(icon_bytes)
        .map_err(|e| {
            tauri::Error::InvalidIcon(std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })?
        .to_rgba8();
    let width = icon_img.width();
    let height = icon_img.height();
    Ok(Image::new_owned(icon_img.into_raw(), width, height))
}

#[cfg(target_os = "windows")]
fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
fn windows_tray_icon_bytes(system_uses_light_theme: bool) -> &'static [u8] {
    if system_uses_light_theme {
        WINDOWS_TRAY_ICON_LIGHT
    } else {
        WINDOWS_TRAY_ICON_DARK
    }
}

#[cfg(target_os = "windows")]
fn windows_system_uses_light_theme() -> Option<bool> {
    let subkey = wide_null(WINDOWS_PERSONALIZE_KEY);
    let value_name = wide_null(WINDOWS_SYSTEM_THEME_VALUE);
    let mut value = 0_u32;
    let mut value_size = std::mem::size_of::<u32>() as u32;

    // SAFETY: Both UTF-16 strings are null-terminated and valid for the duration of the call.
    // The output pointers reference initialized writable `u32` values with the advertised size.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            value_name.as_ptr(),
            RRF_RT_REG_DWORD,
            std::ptr::null_mut(),
            std::ptr::addr_of_mut!(value).cast(),
            std::ptr::addr_of_mut!(value_size),
        )
    };

    (status == ERROR_SUCCESS).then_some(value != 0)
}

#[cfg(target_os = "windows")]
fn initial_windows_tray_theme(app: &AppHandle) -> bool {
    windows_system_uses_light_theme()
        .or_else(|| {
            app.get_webview_window("main")
                .and_then(|window| window.theme().ok())
                .map(|theme| matches!(theme, tauri::Theme::Light))
        })
        .unwrap_or(true)
}

#[cfg(target_os = "windows")]
fn set_windows_tray_icon(app: &AppHandle, system_uses_light_theme: bool) {
    let icon = match load_tray_image(windows_tray_icon_bytes(system_uses_light_theme)) {
        Ok(icon) => icon,
        Err(error) => {
            warn!(target: "tray", "无法加载 Windows 托盘主题图标: {}", error);
            return;
        }
    };

    let Some(state) = app.try_state::<AppState>() else {
        warn!(target: "tray", "切换 Windows 托盘主题时 AppState 不可用");
        return;
    };
    let tray = match state.tray_icon.try_lock() {
        Ok(guard) => guard.clone(),
        Err(_) => {
            warn!(target: "tray", "切换 Windows 托盘主题时无法获取托盘图标锁");
            return;
        }
    };

    if let Some(tray) = tray {
        if let Err(error) = tray.set_icon(Some(icon)) {
            warn!(target: "tray", "切换 Windows 托盘主题图标失败: {}", error);
        } else {
            let theme = if system_uses_light_theme {
                "light"
            } else {
                "dark"
            };
            info!(target: "tray", "Windows 托盘图标已切换为 {} 系统主题版本", theme);
        }
    }
}

#[cfg(target_os = "windows")]
fn watch_windows_system_theme(app: AppHandle) {
    let subkey = wide_null(WINDOWS_PERSONALIZE_KEY);
    let mut key: HKEY = std::ptr::null_mut();

    // SAFETY: `subkey` is a valid null-terminated UTF-16 path and `key` is a writable handle.
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE | KEY_NOTIFY,
            std::ptr::addr_of_mut!(key),
        )
    };
    if status != ERROR_SUCCESS {
        warn!(target: "tray", "无法监听 Windows 系统主题，注册表错误码: {}", status);
        WINDOWS_THEME_WATCHER_STARTED.store(false, Ordering::Release);
        return;
    }

    let mut previous_theme = windows_system_uses_light_theme();
    loop {
        // SAFETY: `key` is an open registry handle owned by this thread. A null event handle with
        // synchronous mode makes this call block until the watched key changes.
        let status = unsafe {
            RegNotifyChangeKeyValue(key, 0, REG_NOTIFY_CHANGE_LAST_SET, std::ptr::null_mut(), 0)
        };
        if status != ERROR_SUCCESS {
            warn!(target: "tray", "Windows 系统主题监听已停止，注册表错误码: {}", status);
            break;
        }

        let current_theme = windows_system_uses_light_theme();
        if current_theme != previous_theme {
            if let Some(system_uses_light_theme) = current_theme {
                set_windows_tray_icon(&app, system_uses_light_theme);
            }
            previous_theme = current_theme;
        }
    }

    // SAFETY: `key` was successfully opened above and is closed exactly once by this thread.
    unsafe {
        RegCloseKey(key);
    }
    WINDOWS_THEME_WATCHER_STARTED.store(false, Ordering::Release);
}

#[cfg(target_os = "windows")]
fn start_windows_theme_watcher(app: AppHandle) {
    if WINDOWS_THEME_WATCHER_STARTED.swap(true, Ordering::AcqRel) {
        return;
    }

    if let Err(error) = thread::Builder::new()
        .name("windows-tray-theme".to_string())
        .spawn(move || watch_windows_system_theme(app))
    {
        WINDOWS_THEME_WATCHER_STARTED.store(false, Ordering::Release);
        warn!(target: "tray", "无法启动 Windows 系统主题监听线程: {}", error);
    }
}

/// 当 Tauri 收到 Windows 主题事件时刷新托盘图标。
#[cfg(target_os = "windows")]
pub(crate) fn refresh_windows_tray_theme(app: &AppHandle, fallback_theme: tauri::Theme) {
    let system_uses_light_theme =
        windows_system_uses_light_theme().unwrap_or(matches!(fallback_theme, tauri::Theme::Light));
    set_windows_tray_icon(app, system_uses_light_theme);
}

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

    // Windows 使用与系统任务栏明暗模式匹配的高对比度双峰图标。
    #[cfg(target_os = "windows")]
    let icon = {
        let system_uses_light_theme = initial_windows_tray_theme(app);
        load_tray_image(windows_tray_icon_bytes(system_uses_light_theme))?
    };

    // macOS 使用黑白托盘图标（符合系统设计规范）
    // 图标应为黑色和透明，系统会根据深色/浅色模式自动调整颜色
    #[cfg(target_os = "macos")]
    let icon = {
        let icon_bytes = include_bytes!("../icons/tray-icon-macos@2x.png");
        load_tray_image(icon_bytes)?
    };

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
        #[cfg(target_os = "windows")]
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
                    // 通知前端执行更新检查。
                    // 实际检查逻辑由前端 useUpdateCheck 通过 @tauri-apps/plugin-updater 完成
                    if let Err(e) = app.emit("tray-check-updates", ()) {
                        warn!(target: "tray", "Failed to emit tray-check-updates event: {}", e);
                    }
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

    #[cfg(target_os = "windows")]
    start_windows_theme_watcher(app.clone());

    info!(target: "tray", "托盘菜单设置完成");
    Ok(())
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::*;

    #[test]
    fn windows_tray_theme_selects_the_matching_asset() {
        assert_eq!(windows_tray_icon_bytes(true), WINDOWS_TRAY_ICON_LIGHT);
        assert_eq!(windows_tray_icon_bytes(false), WINDOWS_TRAY_ICON_DARK);
    }

    #[test]
    fn windows_tray_assets_are_monochrome_and_high_dpi() {
        for (bytes, expected_channel) in [
            (WINDOWS_TRAY_ICON_LIGHT, 0_u8),
            (WINDOWS_TRAY_ICON_DARK, 255_u8),
        ] {
            let icon = image::load_from_memory(bytes)
                .expect("generated tray icon should be a valid PNG")
                .to_rgba8();
            assert_eq!(icon.dimensions(), (48, 48));

            let visible_pixels: Vec<_> = icon.pixels().filter(|pixel| pixel[3] > 0).collect();
            assert!(!visible_pixels.is_empty());
            assert!(visible_pixels.iter().all(|pixel| {
                pixel[0] == expected_channel
                    && pixel[1] == expected_channel
                    && pixel[2] == expected_channel
            }));
        }
    }

    #[test]
    fn wide_null_produces_a_single_null_terminator() {
        let encoded = wide_null("SystemUsesLightTheme");
        assert_eq!(encoded.last(), Some(&0));
        assert!(!encoded[..encoded.len() - 1].contains(&0));
    }
}
