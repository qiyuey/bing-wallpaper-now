use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send, sel, ClassType};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSScreen, NSWorkspace};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSDictionary, NSString, NSURL};

#[cfg(target_os = "macos")]
use once_cell::sync::Lazy;

// 全局静态变量，用于存储当前设置的壁纸路径
#[cfg(target_os = "macos")]
static CURRENT_WALLPAPER: Lazy<Arc<Mutex<Option<PathBuf>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

// 声明 WallpaperObserver 类，用于监听 Space 切换通知
#[cfg(target_os = "macos")]
use objc2_foundation::NSObject;

#[cfg(target_os = "macos")]
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "WallpaperObserver"]
    struct WallpaperObserver;

    impl WallpaperObserver {
        #[unsafe(method(onSpaceChanged:))]
        fn on_space_changed(&self, _notification: &AnyObject) {
            if let Some(path) = CURRENT_WALLPAPER.lock().unwrap().as_ref() {
                let _ = set_wallpaper_for_all_screens(path);
            }
        }
    }
);

/// 初始化 macOS 通知观察者
/// 必须在应用启动时调用一次
///
/// 监听 NSWorkspaceActiveSpaceDidChangeNotification 通知
/// 当用户切换 Space 或退出全屏时自动重新应用壁纸
#[cfg(target_os = "macos")]
pub fn initialize_observer() {
    unsafe {
        setup_workspace_observer();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_observer() {
    // 其他平台不需要初始化
}

/// 设置 Workspace 观察者
#[cfg(target_os = "macos")]
unsafe fn setup_workspace_observer() {
    // 获取 NSWorkspace 和通知中心
    let workspace = NSWorkspace::sharedWorkspace();
    let notification_center = workspace.notificationCenter();

    // 创建观察者实例
    let observer: Retained<WallpaperObserver> = msg_send![WallpaperObserver::class(), new];

    // 注册 Space 切换通知
    // NSWorkspaceActiveSpaceDidChangeNotification 是 macOS 系统通知名称
    let notification_name = NSString::from_str("NSWorkspaceActiveSpaceDidChangeNotification");

    // 将观察者转换为 AnyObject 引用进行注册
    let observer_ref: &AnyObject = &**observer;

    notification_center.addObserver_selector_name_object(
        observer_ref,
        sel!(onSpaceChanged:),
        Some(&notification_name),
        None,
    );

    // 使用 std::mem::forget 防止观察者被释放
    // 这样观察者会一直存活，直到程序退出
    std::mem::forget(observer);
}

/// 设置桌面壁纸(跨平台)
///
/// # Arguments
/// * `image_path` - 壁纸图片的路径
pub fn set_wallpaper(image_path: &Path) -> Result<()> {
    if !image_path.exists() {
        anyhow::bail!("Wallpaper image does not exist: {:?}", image_path);
    }

    // macOS 使用 NSWorkspace API 来处理多显示器和全屏场景
    #[cfg(target_os = "macos")]
    {
        set_wallpaper_macos(image_path)
    }

    // 其他平台使用 wallpaper crate
    #[cfg(not(target_os = "macos"))]
    {
        wallpaper::set_from_path(image_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to set wallpaper: {}", e))?;
        Ok(())
    }
}

/// macOS 专用壁纸设置函数
///
/// 使用 NSWorkspace API 来设置壁纸，可以正确处理全屏应用场景
/// 遍历所有屏幕并为每个屏幕设置壁纸
#[cfg(target_os = "macos")]
fn set_wallpaper_macos(image_path: &Path) -> Result<()> {
    // 保存当前壁纸路径到全局变量
    if let Ok(mut current) = CURRENT_WALLPAPER.lock() {
        *current = Some(image_path.to_path_buf());
    }

    // 设置壁纸
    set_wallpaper_for_all_screens(image_path)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn set_wallpaper_for_all_screens(image_path: &Path) -> Result<()> {
    let path_str = image_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;

    // 创建 NSURL
    let ns_path = NSString::from_str(path_str);
    let url = NSURL::fileURLWithPath(&ns_path);

    // 获取共享的 NSWorkspace 实例和主线程标记
    // SAFETY: Tauri 在主线程上调用此函数，所有 Objective-C API 调用都是安全的
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let workspace = NSWorkspace::sharedWorkspace();

        // 获取所有屏幕
        let screens = NSScreen::screens(mtm);
        let screen_count = screens.len();

        if screen_count == 0 {
            return Err(anyhow::anyhow!("No screens found"));
        }

        // 为每个屏幕设置壁纸
        let mut errors = Vec::new();
        let mut _success_count = 0;

        for i in 0..screen_count {
            let screen = screens.objectAtIndex(i);

            // 创建空的 options dictionary
            let options = NSDictionary::new();

            // 设置壁纸
            match workspace.setDesktopImageURL_forScreen_options_error(&url, &screen, &options) {
                Ok(_) => {
                    _success_count += 1;
                }
                Err(error) => {
                    let error_str = error.localizedDescription().to_string();
                    errors.push(format!("Screen {}: {}", i, error_str));
                }
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to set wallpaper for some screens: {}",
                errors.join("; ")
            ));
        }
    }

    Ok(())
}

/// 获取当前的桌面壁纸路径
pub fn get_current_wallpaper() -> Result<String> {
    wallpaper::get().map_err(|e| anyhow::anyhow!("Failed to get current wallpaper: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // 这个测试会实际修改系统壁纸,所以默认忽略
    fn test_get_current_wallpaper() {
        let result = get_current_wallpaper();
        assert!(result.is_ok());
        println!("Current wallpaper: {}", result.unwrap());
    }
}
