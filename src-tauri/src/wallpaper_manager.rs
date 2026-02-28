use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(target_os = "windows")]
use log::info;
#[cfg(all(unix, not(target_os = "macos")))]
use log::info;
#[cfg(target_os = "macos")]
use log::{info, warn};
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
            // 规范化路径：使用 canonicalize 或至少展开符号链接
            let path = PathBuf::from(path_str);
            match path.canonicalize() {
                Ok(canonical) => Some(canonical),
                Err(_) => Some(path), // 如果规范化失败，返回原始路径
            }
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

            // 智能对比：只有不一致时才重新设置
            if let Ok(state) = WALLPAPER_STATE.lock()
                && let Some(expected) = &state.expected
            {
                let actual = get_all_desktop_images();
                let screen_orientations = get_screen_orientations();

                // 检查是否所有显示器的壁纸都与期望一致（考虑屏幕方向）
                let all_match = screen_orientations.iter().all(|screen| {
                    let expected_path = if screen.is_portrait {
                        // 尝试从横屏路径生成竖屏路径
                        expected.parent().and_then(|p| {
                        expected
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .map(|s| p.join(format!("{}r.jpg", s)))
                        })
                    } else {
                        Some(expected.clone())
                    };

                    if let Some(expected_path) = expected_path {
                        actual.get(&screen.screen_index)
                            .map(|actual_path| actual_path == &expected_path)
                            .unwrap_or(false)
                    } else {
                        !screen.is_portrait // 如果没有竖屏壁纸，跳过竖屏检查
                    }
                });

                if all_match {
                    // 壁纸一致，跳过设置
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
                let path = expected.clone();
                let portrait_path = path.parent().and_then(|p| {
                    path.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| p.join(format!("{}r.jpg", s)))
                        .filter(|p| p.exists())
                });
                drop(state);
                let _ = set_wallpaper_for_all_screens_by_orientation(&path, portrait_path.as_deref(), &screen_orientations);
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
/// * `image_path` - 壁纸图片的路径（横屏版本）
/// * `portrait_image_path` - 竖屏壁纸图片的路径（可选）
pub fn set_wallpaper(image_path: &Path, portrait_image_path: Option<&Path>) -> Result<()> {
    if !image_path.exists() {
        anyhow::bail!("Wallpaper image does not exist: {:?}", image_path);
    }

    // portrait_image_path 仅在 macOS 上使用（Windows/Linux 暂不支持竖屏壁纸）
    #[cfg(not(target_os = "macos"))]
    let _ = portrait_image_path;

    // macOS 使用 NSWorkspace API 来处理多显示器和全屏场景
    #[cfg(target_os = "macos")]
    {
        set_wallpaper_macos(image_path, portrait_image_path)
    }

    // Windows 平台实现
    #[cfg(windows)]
    {
        // 获取当前壁纸路径
        let current_wallpaper = wallpaper::get().unwrap_or_else(|_e| String::new());

        // Windows: 使用规范化路径进行比较
        let target_path = image_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;

        // Windows 文件系统不区分大小写，需要规范化路径
        // 1. 统一使用反斜杠
        // 2. 转换为小写
        // 3. 尝试规范化为绝对路径
        let normalize_windows_path = |path: &str| -> String {
            let normalized = path.replace('/', "\\").to_lowercase();
            // 尝试获取绝对路径
            if let Ok(abs_path) = std::path::Path::new(path).canonicalize() {
                abs_path.to_string_lossy().to_lowercase()
            } else {
                normalized
            }
        };

        let current_normalized = normalize_windows_path(&current_wallpaper);
        let target_normalized = normalize_windows_path(target_path);

        if !current_wallpaper.is_empty() && current_normalized == target_normalized {
            info!(target: "wallpaper", "壁纸已设置为 {:?}，跳过设置", target_path);
            return Ok(());
        }

        // 设置新壁纸
        info!(target: "wallpaper", "设置壁纸为 {:?} (current: {:?})",
            target_path, current_wallpaper
        );
        wallpaper::set_from_path(target_path)
            .map_err(|e| anyhow::anyhow!("设置壁纸失败: {}", e))?;
        Ok(())
    }

    // Linux 和其他 Unix 平台实现
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        // 获取当前壁纸路径
        let current_wallpaper = wallpaper::get().unwrap_or_else(|_| String::new());

        let target_path = image_path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;

        // Linux: 文件系统区分大小写，使用规范化的绝对路径比较
        let normalize_unix_path = |path: &str| -> String {
            // 尝试获取规范化的绝对路径
            if let Ok(canonical) = std::path::Path::new(path).canonicalize() {
                canonical.to_string_lossy().to_string()
            } else {
                // 如果无法规范化，至少确保路径格式一致
                path.to_string()
            }
        };

        let current_normalized = normalize_unix_path(&current_wallpaper);
        let target_normalized = normalize_unix_path(target_path);

        if !current_wallpaper.is_empty() && current_normalized == target_normalized {
            info!(target: "wallpaper", "壁纸已设置为 {:?}，跳过设置", target_path);
            return Ok(());
        }

        // 设置新壁纸
        info!(target: "wallpaper", "设置壁纸为 {:?} (current: {:?})",
            target_path, current_wallpaper
        );
        wallpaper::set_from_path(target_path)
            .map_err(|e| anyhow::anyhow!("Failed to set wallpaper: {}", e))?;
        Ok(())
    }
}

/// macOS 专用壁纸设置函数
///
/// 使用 NSWorkspace API 来设置壁纸，可以正确处理全屏应用场景
/// 遍历所有屏幕并根据屏幕方向为每个屏幕设置对应的壁纸，并验证设置结果
#[cfg(target_os = "macos")]
fn set_wallpaper_macos(image_path: &Path, portrait_image_path: Option<&Path>) -> Result<()> {
    // 获取屏幕方向信息
    let screen_orientations = get_screen_orientations();

    // 规范化目标路径以进行准确比较
    let target_path = match image_path.canonicalize() {
        Ok(canonical) => canonical,
        Err(_) => image_path.to_path_buf(),
    };

    let target_portrait_path = portrait_image_path.and_then(|p| p.canonicalize().ok());

    // 先检查当前所有显示器的壁纸是否已经是目标壁纸
    let current_wallpapers = get_all_desktop_images();

    // 检查是否所有屏幕的壁纸都已正确设置
    let all_match = !current_wallpapers.is_empty()
        && screen_orientations.iter().all(|screen| {
            let expected_path = if screen.is_portrait {
                target_portrait_path.as_ref()
            } else {
                Some(&target_path)
            };

            if let Some(expected) = expected_path {
                current_wallpapers
                    .get(&screen.screen_index)
                    .map(|actual| actual == expected)
                    .unwrap_or(false)
            } else {
                // 如果没有竖屏壁纸文件，跳过竖屏检查
                !screen.is_portrait
            }
        });

    if all_match {
        info!(target: "wallpaper", "所有显示器壁纸已正确设置，跳过设置");

        // 更新状态但不重新设置
        if let Ok(mut state) = WALLPAPER_STATE.lock() {
            state.expected = Some(target_path.clone());
            state.actual_per_screen = current_wallpapers;
            state.skipped_count += 1;
            if state.skipped_count % 10 == 0 {
                info!(target: "wallpaper", "已跳过 {} 次不必要的壁纸设置", state.skipped_count);
            }
        }
        return Ok(());
    }

    // 保存期望壁纸路径到全局变量
    if let Ok(mut state) = WALLPAPER_STATE.lock() {
        state.expected = Some(target_path.clone());
    }

    // 根据屏幕方向设置壁纸
    set_wallpaper_for_all_screens_by_orientation(
        image_path,
        portrait_image_path,
        &screen_orientations,
    )?;

    // 验证设置结果：读取各显示器实际壁纸并记录
    let actual = get_all_desktop_images();

    if let Ok(mut state) = WALLPAPER_STATE.lock() {
        state.actual_per_screen = actual.clone();

        // 检查是否所有显示器都设置成功
        let all_success = screen_orientations.iter().all(|screen| {
            let expected_path = if screen.is_portrait {
                target_portrait_path.as_ref()
            } else {
                Some(&target_path)
            };

            if let Some(expected) = expected_path {
                actual
                    .get(&screen.screen_index)
                    .map(|actual_path| actual_path == expected)
                    .unwrap_or(false)
            } else {
                // 如果没有竖屏壁纸文件，跳过竖屏检查
                !screen.is_portrait
            }
        });

        if all_success {
            info!(target: "wallpaper", "壁纸设置成功并已验证: 横屏={:?}, 竖屏={:?} (共 {} 个显示器)",
                  target_path, target_portrait_path, actual.len());
        } else {
            warn!(target: "wallpaper", "部分显示器壁纸设置可能失败: 期望横屏={:?}, 期望竖屏={:?}, 实际={:?}",
                  target_path, target_portrait_path, actual);
        }
    }

    Ok(())
}

/// 根据屏幕方向为所有屏幕设置壁纸
#[cfg(target_os = "macos")]
fn set_wallpaper_for_all_screens_by_orientation(
    landscape_path: &Path,
    portrait_path: Option<&Path>,
    screen_orientations: &[ScreenOrientation],
) -> Result<()> {
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

            // 根据屏幕方向选择对应的壁纸文件
            let wallpaper_path = screen_orientations
                .iter()
                .find(|s| s.screen_index == i)
                .map(|s| {
                    if s.is_portrait {
                        if let Some(portrait) = portrait_path {
                            portrait.to_path_buf()
                        } else {
                            warn!(
                                target: "wallpaper",
                                "屏幕 {} 是竖屏，但竖屏壁纸不存在，将使用横屏壁纸",
                                i
                            );
                            landscape_path.to_path_buf()
                        }
                    } else {
                        landscape_path.to_path_buf()
                    }
                })
                .unwrap_or_else(|| {
                    warn!(
                        target: "wallpaper",
                        "找不到屏幕 {} 的方向信息，使用横屏壁纸",
                        i
                    );
                    landscape_path.to_path_buf()
                }); // 如果找不到屏幕信息，默认使用横屏壁纸

            let path_str = wallpaper_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid path encoding"))?;

            // 创建 NSURL
            let ns_path = NSString::from_str(path_str);
            let url = NSURL::fileURLWithPath(&ns_path);

            // 创建空的 options dictionary
            let options = NSDictionary::new();

            // 设置壁纸
            match workspace.setDesktopImageURL_forScreen_options_error(&url, &screen, &options) {
                Ok(_) => {
                    _success_count += 1;
                    info!(
                        target: "wallpaper",
                        "屏幕 {} ({}) 壁纸设置成功: {:?}",
                        i,
                        if screen_orientations.iter().any(|s| s.screen_index == i && s.is_portrait) {
                            "竖屏"
                        } else {
                            "横屏"
                        },
                        wallpaper_path
                    );
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

/// 屏幕方向信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenOrientation {
    /// 屏幕索引
    pub screen_index: usize,
    /// 是否为竖屏（高度 > 宽度）
    pub is_portrait: bool,
    /// 屏幕宽度（像素）
    pub width: f64,
    /// 屏幕高度（像素）
    pub height: f64,
}

/// 获取所有屏幕的方向信息
#[cfg(target_os = "macos")]
pub fn get_screen_orientations() -> Vec<ScreenOrientation> {
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let screens = NSScreen::screens(mtm);
        let screen_count = screens.len();

        let mut result = Vec::new();
        for i in 0..screen_count {
            let screen = screens.objectAtIndex(i);
            let frame = screen.frame();

            // NSRect 包含 origin (x, y) 和 size (width, height)
            // 我们需要使用 size 来获取宽度和高度
            let width = frame.size.width;
            let height = frame.size.height;

            result.push(ScreenOrientation {
                screen_index: i,
                is_portrait: height > width,
                width,
                height,
            });
        }
        result
    }
}

/// 获取所有屏幕的方向信息（非 macOS 平台）
#[cfg(not(target_os = "macos"))]
pub fn get_screen_orientations() -> Vec<ScreenOrientation> {
    // Windows 和 Linux 平台的实现
    // 这里可以使用 wallpaper crate 或其他系统 API
    // 暂时返回空数组，后续可以根据需要实现
    vec![]
}

// (已移除 get_current_wallpaper 函数以消除未使用警告)
#[cfg(test)]
mod tests {
    // get_current_wallpaper 已移除，测试删除以避免引用不存在的函数
    // 保留空模块占位，后续可添加新的单元测试。
}
