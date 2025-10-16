use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// macOS 原生 API 绑定
// 注意：cocoa 和 objc crate 存在以下已知警告：
// 1. `unexpected cfg` - 来自 objc 0.2 内部宏，该版本的已知问题
// 2. `deprecated` - cocoa crate 建议迁移到 objc2/objc2-foundation
// 这些警告来自依赖库内部，无法在当前代码中解决
// 完全解决需要重写为使用 objc2 生态系统
#[cfg(target_os = "macos")]
#[allow(deprecated)] // cocoa crate API 已弃用但仍然可用
use cocoa::base::{id, nil};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::foundation::{NSDictionary, NSString};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

#[cfg(target_os = "macos")]
use once_cell::sync::Lazy;

// 全局静态变量，用于存储当前设置的壁纸路径
#[cfg(target_os = "macos")]
static CURRENT_WALLPAPER: Lazy<Arc<Mutex<Option<PathBuf>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

/// 初始化 macOS 通知观察者
/// 必须在应用启动时调用一次
#[cfg(target_os = "macos")]
pub fn initialize_observer() {
    unsafe {
        // 创建观察者对象
        setup_workspace_observer();
    }
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

/// 设置全局监听器，监听 Space 切换事件
#[cfg(target_os = "macos")]
#[allow(deprecated)] // 使用 cocoa crate 的已弃用 API
unsafe fn setup_workspace_observer() {
    use objc::declare::ClassDecl;
    use objc::runtime::{Class, Object, Sel};

    // 创建回调函数
    extern "C" fn on_space_changed(_this: &Object, _cmd: Sel, _notification: id) {
        if let Some(path) = CURRENT_WALLPAPER.lock().unwrap().as_ref() {
            let _ = set_wallpaper_for_all_screens(path);
        }
    }

    // 检查类是否已经注册
    let observer_class = if let Some(cls) = Class::get("WallpaperObserver") {
        cls
    } else {
        // 创建新的观察者类
        let superclass = Class::get("NSObject").expect("NSObject not found");
        let mut decl =
            ClassDecl::new("WallpaperObserver", superclass).expect("Failed to create class");

        // 添加方法
        decl.add_method(
            sel!(onSpaceChanged:),
            on_space_changed as extern "C" fn(&Object, Sel, id),
        );

        decl.register()
    };

    // 创建观察者实例
    let observer: id = msg_send![observer_class, alloc];
    let observer: id = msg_send![observer, init];

    // 获取通知中心
    let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
    let notification_center: id = msg_send![workspace, notificationCenter];

    // 注册通知观察者
    let notification_name =
        NSString::alloc(nil).init_str("NSWorkspaceActiveSpaceDidChangeNotification");
    let _: () = msg_send![
        notification_center,
        addObserver: observer
        selector: sel!(onSpaceChanged:)
        name: notification_name
        object: nil
    ];
}

/// macOS 专用壁纸设置函数
///
/// 使用 NSWorkspace API 来设置壁纸，可以正确处理全屏应用场景
/// 遍历所有屏幕并为每个屏幕设置壁纸
#[cfg(target_os = "macos")]
#[allow(deprecated)] // 使用 cocoa crate 的已弃用 API
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
#[allow(deprecated)] // 使用 cocoa crate 的已弃用 API
fn set_wallpaper_for_all_screens(image_path: &Path) -> Result<()> {
    unsafe {
        let path_str = image_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;

        // 创建 NSURL
        let ns_path = NSString::alloc(nil).init_str(path_str);
        let url: id = msg_send![class!(NSURL), fileURLWithPath: ns_path];

        if url == nil {
            return Err(anyhow::anyhow!("Failed to create NSURL from path"));
        }

        // 获取共享的 NSWorkspace 实例
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];

        // 获取所有屏幕
        let screens: id = msg_send![class!(NSScreen), screens];
        let screen_count: usize = msg_send![screens, count];

        if screen_count == 0 {
            return Err(anyhow::anyhow!("No screens found"));
        }

        // 为每个屏幕设置壁纸
        let mut errors = Vec::new();
        let mut _success_count = 0;
        for i in 0..screen_count {
            let screen: id = msg_send![screens, objectAtIndex: i];

            // 创建空的 options dictionary
            let options = NSDictionary::dictionary(nil);

            // 设置壁纸
            let mut error: id = nil;
            let success: bool = msg_send![
                workspace,
                setDesktopImageURL: url
                forScreen: screen
                options: options
                error: &mut error
            ];

            if !success {
                if error != nil {
                    let error_desc: id = msg_send![error, localizedDescription];
                    let error_cstr: *const i8 = msg_send![error_desc, UTF8String];
                    if !error_cstr.is_null() {
                        let error_str = std::ffi::CStr::from_ptr(error_cstr)
                            .to_string_lossy()
                            .into_owned();
                        errors.push(format!("Screen {}: {}", i, error_str));
                    } else {
                        errors.push(format!("Screen {}: Unknown error", i));
                    }
                } else {
                    errors.push(format!("Screen {}: Failed without error details", i));
                }
            } else {
                _success_count += 1;
            }
        }

        if !errors.is_empty() {
            return Err(anyhow::anyhow!(
                "Failed to set wallpaper for some screens: {}",
                errors.join("; ")
            ));
        }

        Ok(())
    }
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
