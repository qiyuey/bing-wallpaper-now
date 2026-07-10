use std::path::PathBuf;

use log::warn;
use tauri::AppHandle;

#[cfg(windows)]
use notify_rust::Notification;

use crate::models::LocalWallpaper;

/// 在应用启动完成前初始化现代 macOS 通知中心的 delegate。
///
/// Apple 要求尽早设置 `UNUserNotificationCenterDelegate`，否则可能错过通知响应。
#[cfg(target_os = "macos")]
pub(crate) fn initialize_notification_center() {
    match mac_usernotifications::blocking::get_notification_settings() {
        Ok(settings) => {
            log::info!(
                target: "notification",
                "UNUserNotificationCenter 已初始化，授权状态: {:?}",
                settings.authorization_status
            );
        }
        Err(e) => {
            warn!(target: "notification", "初始化 UNUserNotificationCenter 失败: {}", e);
        }
    }
}

/// 通知中展示的本地化文本。
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct WallpaperNotificationContent {
    pub title: String,
    pub body: String,
}

/// 用户点击通知正文后执行的应用内动作。
#[derive(Debug)]
pub(crate) enum NotificationClickAction {
    None,
    ShowMainWindow,
}

/// 从服务器结果中找出相对本地基线真正更新的最新壁纸。
///
/// 本地没有任何壁纸时只建立基线，不发送首次同步通知。
pub(crate) fn find_new_latest_wallpaper<'a>(
    server_wallpapers: &'a [LocalWallpaper],
    existing_wallpapers: &[LocalWallpaper],
) -> Option<&'a LocalWallpaper> {
    let latest_server = server_wallpapers
        .iter()
        .max_by(|left, right| left.end_date.cmp(&right.end_date))?;
    let latest_existing = existing_wallpapers
        .iter()
        .max_by(|left, right| left.end_date.cmp(&right.end_date))?;

    (latest_server.end_date > latest_existing.end_date).then_some(latest_server)
}

/// 构建新壁纸通知的本地化标题和说明。
pub(crate) fn build_wallpaper_notification_content(
    wallpaper: &LocalWallpaper,
    resolved_language: &str,
) -> WallpaperNotificationContent {
    let is_chinese = resolved_language == "zh-CN";
    let title_prefix = if is_chinese {
        "新壁纸"
    } else {
        "New Wallpaper"
    };
    let title = format_wallpaper_date(&wallpaper.end_date)
        .map(|date| format!("{title_prefix} · {date}"))
        .unwrap_or_else(|| title_prefix.to_string());

    let subtitle = card_subtitle(&wallpaper.copyright);
    let body_parts = [wallpaper.title.trim(), subtitle.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    let body = if body_parts.is_empty() {
        if is_chinese {
            "今日壁纸已准备好"
        } else {
            "Today's wallpaper is ready"
        }
        .to_string()
    } else {
        body_parts.join("\n")
    };

    WallpaperNotificationContent { title, body }
}

/// 与 WallpaperCard 保持一致：版权括号外的部分作为副标题。
fn card_subtitle(copyright: &str) -> String {
    let copyright = copyright.trim();
    let Some(open_index) = copyright.find('(') else {
        return copyright.to_string();
    };

    let prefix = copyright[..open_index].trim();
    let parenthesized = &copyright[open_index..];
    let has_valid_parentheses = !prefix.is_empty()
        && parenthesized.len() > 2
        && parenthesized.starts_with('(')
        && parenthesized.ends_with(')')
        && parenthesized[1..parenthesized.len() - 1]
            .chars()
            .all(|c| c != ')')
        && !parenthesized[1..parenthesized.len() - 1].is_empty();

    if has_valid_parentheses {
        prefix.to_string()
    } else {
        copyright.to_string()
    }
}

fn format_wallpaper_date(end_date: &str) -> Option<String> {
    let bytes = end_date.as_bytes();
    if bytes.len() != 8 || !bytes.iter().all(u8::is_ascii_digit) {
        return None;
    }

    Some(format!(
        "{}-{}-{}",
        &end_date[0..4],
        &end_date[4..6],
        &end_date[6..8]
    ))
}

#[cfg(windows)]
fn show_windows_notification(
    app: &AppHandle,
    title: &str,
    body: &str,
    image_path: Option<&std::path::Path>,
    click_action: NotificationClickAction,
) -> anyhow::Result<()> {
    let mut notification = Notification::new();
    notification
        .appname(
            app.config()
                .product_name
                .as_deref()
                .unwrap_or("Bing Wallpaper Now"),
        )
        .summary(title)
        .body(body);

    if let Some(path) = image_path {
        notification.image_path(&path.to_string_lossy());
    }

    if !tauri::is_dev() {
        notification.app_id(&app.config().identifier);
    }

    let handle = notification.show()?;

    if let NotificationClickAction::ShowMainWindow = click_action {
        let app = app.clone();
        std::thread::spawn(move || {
            if let Err(e) = handle.wait_for_response(move |response| {
                if response.is_default_action()
                    && let Err(e) = crate::commands::window::show_main_window_from_notification(app)
                {
                    warn!(target: "notification", "处理通知点击失败: {}", e);
                }
            }) {
                warn!(target: "notification", "等待通知点击响应失败: {}", e);
            }
        });
    }

    Ok(())
}

#[cfg(target_os = "macos")]
async fn send_macos_notification(
    app: AppHandle,
    title: String,
    body: String,
    image_path: Option<PathBuf>,
    click_action: NotificationClickAction,
) -> Result<(), String> {
    let authorized = mac_usernotifications::request_auth()
        .await
        .map_err(|e| format!("请求系统通知权限失败: {e}"))?;
    if !authorized {
        return Err("用户未授予系统通知权限".to_string());
    }

    let mut notification = mac_usernotifications::Notification::new()
        .title(title)
        .message(body);
    if let Some(path) = image_path {
        notification = notification.image_path(path.to_string_lossy());
    }

    let handle = notification
        .send()
        .await
        .map_err(|e| format!("发送系统通知失败: {e}"))?;

    if let NotificationClickAction::ShowMainWindow = click_action {
        tauri::async_runtime::spawn(async move {
            match handle.response().await {
                Ok(response) if response.is_default_action() => {
                    if let Err(e) = crate::commands::window::show_main_window_from_notification(app)
                    {
                        warn!(target: "notification", "处理通知点击失败: {}", e);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    warn!(target: "notification", "等待通知点击响应失败: {}", e);
                }
            }
        });
    }

    Ok(())
}

/// 使用当前平台的原生通知实现发送系统通知。
pub(crate) async fn send_system_notification(
    app: AppHandle,
    title: String,
    body: String,
    image_path: Option<PathBuf>,
    click_action: NotificationClickAction,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        send_macos_notification(app, title, body, image_path, click_action).await
    }

    #[cfg(windows)]
    {
        tauri::async_runtime::spawn_blocking(move || {
            show_windows_notification(&app, &title, &body, image_path.as_deref(), click_action)
        })
        .await
        .map_err(|e| format!("通知任务执行失败: {e}"))?
        .map_err(|e| format!("发送系统通知失败: {e}"))
    }
}

/// 供前端现有文本通知调用的命令。
#[tauri::command]
pub(crate) async fn show_system_notification(
    app: AppHandle,
    title: String,
    body: String,
) -> Result<(), String> {
    send_system_notification(app, title, body, None, NotificationClickAction::None).await
}

/// 返回当前进程是否支持开发者通知测试。
///
/// 现代 macOS 通知要求进程从真实 `.app` bundle 启动；裸 `tauri dev`
/// 进程不满足该条件，因此不应展示一个必然失败的测试入口。
#[tauri::command]
pub(crate) fn notification_test_available() -> bool {
    if !cfg!(debug_assertions) {
        return false;
    }

    #[cfg(target_os = "macos")]
    {
        mac_usernotifications::check_bundle().is_ok()
    }

    #[cfg(windows)]
    {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wallpaper(date: &str, title: &str, copyright: &str) -> LocalWallpaper {
        LocalWallpaper {
            title: title.to_string(),
            copyright: copyright.to_string(),
            copyright_link: String::new(),
            end_date: date.to_string(),
            urlbase: String::new(),
        }
    }

    #[test]
    fn first_sync_only_establishes_baseline() {
        let server = vec![wallpaper("20260711", "New", "Copyright")];
        assert!(find_new_latest_wallpaper(&server, &[]).is_none());
    }

    #[test]
    fn only_a_newer_latest_date_triggers_notification() {
        let existing = vec![wallpaper("20260710", "Old", "Copyright")];
        let server = vec![
            wallpaper("20260709", "Backfill", "Copyright"),
            wallpaper("20260711", "Latest", "Copyright"),
        ];

        let latest = find_new_latest_wallpaper(&server, &existing).unwrap();
        assert_eq!(latest.end_date, "20260711");

        let unchanged = vec![wallpaper("20260710", "Same", "Copyright")];
        assert!(find_new_latest_wallpaper(&unchanged, &existing).is_none());
        let older = vec![wallpaper("20260709", "Older", "Copyright")];
        assert!(find_new_latest_wallpaper(&older, &existing).is_none());
    }

    #[test]
    fn builds_localized_wallpaper_content() {
        let item = wallpaper("20260711", "山谷", "摄影：测试");
        let zh = build_wallpaper_notification_content(&item, "zh-CN");
        assert_eq!(zh.title, "新壁纸 · 2026-07-11");
        assert_eq!(zh.body, "山谷\n摄影：测试");

        let en = build_wallpaper_notification_content(&item, "en-US");
        assert_eq!(en.title, "New Wallpaper · 2026-07-11");
        assert_eq!(en.body, "山谷\n摄影：测试");
    }

    #[test]
    fn omits_empty_fields_and_uses_fallback_body() {
        let partial = wallpaper("20260711", "  Landscape  ", "");
        assert_eq!(
            build_wallpaper_notification_content(&partial, "en-US").body,
            "Landscape"
        );

        let empty = wallpaper("", "", "  ");
        assert_eq!(
            build_wallpaper_notification_content(&empty, "zh-CN").body,
            "今日壁纸已准备好"
        );
    }

    #[test]
    fn omits_invalid_wallpaper_dates() {
        let item = wallpaper("invalid", "Landscape", "Copyright");
        let content = build_wallpaper_notification_content(&item, "en-US");
        assert_eq!(content.title, "New Wallpaper");
        assert_eq!(content.body, "Landscape\nCopyright");
    }

    #[test]
    fn notification_subtitle_matches_wallpaper_card() {
        let item = wallpaper(
            "20260711",
            "Mountain lake",
            "Banff National Park (© Test Photographer)",
        );
        assert_eq!(
            build_wallpaper_notification_content(&item, "en-US").body,
            "Mountain lake\nBanff National Park"
        );
    }
}
