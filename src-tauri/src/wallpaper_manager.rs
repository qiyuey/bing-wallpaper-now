use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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

/// 初始化 macOS 通知观察者
/// 必须在应用启动时调用一次
///
/// 注意：objc2 的动态类创建API相对复杂，Space observer功能暂时禁用
/// 基本的壁纸设置功能不受影响
#[cfg(target_os = "macos")]
pub fn initialize_observer() {
    // Space observer 功能在 objc2 迁移中暂时禁用
    // 基本壁纸设置功能仍然正常工作
}

#[cfg(not(target_os = "macos"))]
pub fn initialize_observer() {
    // 其他平台不需要初始化
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
    let url = unsafe { NSURL::fileURLWithPath(&ns_path) };

    // 获取共享的 NSWorkspace 实例和主线程标记
    // SAFETY: Tauri 在主线程上调用此函数，所有 Objective-C API 调用都是安全的
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let workspace = NSWorkspace::sharedWorkspace();

        // 获取所有屏幕
        let screens = NSScreen::screens(mtm);
        let screen_count = screens.count();

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
