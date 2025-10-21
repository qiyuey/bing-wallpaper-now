use anyhow::Result;
use std::path::Path;

#[cfg(target_os = "macos")]
use log::{debug, info, trace, warn};
#[cfg(target_os = "macos")]
use std::collections::HashMap;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use std::sync::{Arc, Mutex};

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::runtime::AnyObject;
#[cfg(target_os = "macos")]
use objc2::{ClassType, define_class, msg_send, sel};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSScreen, NSWorkspace};
#[cfg(target_os = "macos")]
use objc2_foundation::{MainThreadMarker, NSDictionary, NSString, NSURL};

#[cfg(target_os = "macos")]
use std::sync::LazyLock;

/// 壁纸状态：记录期望壁纸和各显示器实际壁纸
#[cfg(target_os = "macos")]
#[derive(Debug, Clone, Default)]
struct WallpaperState {
    /// 期望设置的壁纸路径
    expected: Option<PathBuf>,
    /// 各显示器实际成功设置的壁纸路径 (screen_index -> path)
    actual_per_screen: HashMap<usize, PathBuf>,
    /// 跳过的重复设置次数（性能统计）
    skipped_count: u64,
}

// 全局静态变量，用于存储壁纸状态
#[cfg(target_os = "macos")]
static WALLPAPER_STATE: LazyLock<Arc<Mutex<WallpaperState>>> =
    LazyLock::new(|| Arc::new(Mutex::new(WallpaperState::default())));

/// 获取指定显示器的当前壁纸路径
#[cfg(target_os = "macos")]
fn get_desktop_image_url_for_screen(screen_index: usize) -> Option<PathBuf> {
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let workspace = NSWorkspace::sharedWorkspace();
        let screens = NSScreen::screens(mtm);

        if screen_index >= screens.len() {
            return None;
        }

        let screen = screens.objectAtIndex(screen_index);

        // 调用 desktopImageURLForScreen: 方法（需要转换为 &AnyObject）
        let screen_obj: &AnyObject = &screen;
        let url: Option<Retained<NSURL>> =
            msg_send![&workspace, desktopImageURLForScreen: screen_obj];

        url.and_then(|nsurl| {
            let path_str = nsurl.path()?.to_string();
            Some(PathBuf::from(path_str))
        })
    }
}

/// 获取所有显示器的当前壁纸路径
#[cfg(target_os = "macos")]
fn get_all_desktop_images() -> HashMap<usize, PathBuf> {
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let screens = NSScreen::screens(mtm);
        let screen_count = screens.len();

        let mut result = HashMap::new();
        for i in 0..screen_count {
            if let Some(path) = get_desktop_image_url_for_screen(i) {
                result.insert(i, path);
            }
        }
        result
    }
}

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
            trace!(target: "wallpaper", "Space 切换事件触发");

            // 智能对比：只有不一致时才重新设置
            if let Ok(state) = WALLPAPER_STATE.lock()
                && let Some(expected) = &state.expected
            {
                let actual = get_all_desktop_images();

                // 检查是否所有显示器的壁纸都与期望一致
                let all_match = actual.values().all(|path| path == expected);

                if all_match {
                    // 壁纸一致，跳过设置
                    trace!(target: "wallpaper", "所有显示器壁纸已一致，跳过设置");
                    drop(state);
                    if let Ok(mut state) = WALLPAPER_STATE.lock() {
                        state.skipped_count += 1;
                        if state.skipped_count % 10 == 0 {
                            info!(target: "wallpaper", "已跳过 {} 次不必要的壁纸设置", state.skipped_count);
                        }
                    }
                    return;
                }

                // 壁纸不一致，需要重新设置
                debug!(target: "wallpaper", "检测到壁纸不一致，重新设置: 期望={:?}, 实际={:?}",
                       expected, actual);
                let path = expected.clone();
                drop(state);
                let _ = set_wallpaper_for_all_screens(&path);
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
    let observer_ref: &AnyObject = &observer;

    // Rust 2024: unsafe 函数内的 unsafe 操作需要显式 unsafe 块
    unsafe {
        notification_center.addObserver_selector_name_object(
            observer_ref,
            sel!(onSpaceChanged:),
            Some(&notification_name),
            None,
        );
    }

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
/// 遍历所有屏幕并为每个屏幕设置壁纸，并验证设置结果
#[cfg(target_os = "macos")]
fn set_wallpaper_macos(image_path: &Path) -> Result<()> {
    let target_path = image_path.to_path_buf();

    // 保存期望壁纸路径到全局变量
    if let Ok(mut state) = WALLPAPER_STATE.lock() {
        state.expected = Some(target_path.clone());
    }

    // 设置壁纸
    set_wallpaper_for_all_screens(image_path)?;

    // 验证设置结果：读取各显示器实际壁纸并记录
    let actual = get_all_desktop_images();

    if let Ok(mut state) = WALLPAPER_STATE.lock() {
        state.actual_per_screen = actual.clone();

        // 检查是否所有显示器都设置成功
        let all_success = actual.values().all(|path| path == &target_path);

        if all_success {
            info!(target: "wallpaper", "壁纸设置成功并已验证: {:?} (共 {} 个显示器)",
                  target_path, actual.len());
        } else {
            warn!(target: "wallpaper", "部分显示器壁纸设置可能失败: 期望={:?}, 实际={:?}",
                  target_path, actual);
        }
    }

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

/// (已移除 get_current_wallpaper 函数以消除未使用警告)

#[cfg(test)]
mod tests {
    // get_current_wallpaper 已移除，测试删除以避免引用不存在的函数
    // 保留空模块占位，后续可添加新的单元测试。
}
