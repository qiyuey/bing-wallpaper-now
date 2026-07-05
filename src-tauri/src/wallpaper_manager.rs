use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[cfg(target_os = "windows")]
use log::{info, warn};
#[cfg(target_os = "macos")]
use log::{info, warn};
#[cfg(target_os = "macos")]
use std::collections::{HashMap, HashSet};
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

#[cfg(windows)]
use std::iter;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SPI_GETDESKWALLPAPER, SPI_SETDESKWALLPAPER, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    SystemParametersInfoW,
};

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

/// 记录"竖屏壁纸缺失，已 fallback 到横屏壁纸"的提示去重状态：
/// 每次切换横屏壁纸时清空已通知集合，同一张壁纸下每个屏幕索引最多通知一次。
#[cfg(target_os = "macos")]
#[derive(Default)]
struct PortraitFallbackNoticeState {
    landscape: Option<PathBuf>,
    notified_screens: HashSet<usize>,
}

/// 全局 fallback 提示去重状态。
#[cfg(target_os = "macos")]
static PORTRAIT_FALLBACK_NOTICE: LazyLock<Mutex<PortraitFallbackNoticeState>> =
    LazyLock::new(|| Mutex::new(PortraitFallbackNoticeState::default()));

/// 获取 Windows 当前桌面壁纸路径。
#[cfg(windows)]
fn get_current_wallpaper_windows() -> Result<String> {
    let mut buffer = [0u16; 260];
    let successful = unsafe {
        SystemParametersInfoW(
            SPI_GETDESKWALLPAPER,
            buffer.len() as u32,
            buffer.as_mut_ptr().cast(),
            0,
        ) == 1
    };

    if !successful {
        return Err(std::io::Error::last_os_error()).context("Failed to get current wallpaper");
    }

    let len = buffer
        .iter()
        .position(|&ch| ch == 0)
        .unwrap_or(buffer.len());
    Ok(String::from_utf16_lossy(&buffer[..len]))
}

/// 规范化 Windows 路径用于比较，避免大小写和分隔符差异导致重复设置。
#[cfg(windows)]
fn normalize_windows_path(path: &Path) -> String {
    let canonical = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('/', "\\");
    canonical.to_lowercase()
}

/// 使用 Win32 API 设置 Windows 桌面壁纸。
#[cfg(windows)]
fn set_wallpaper_windows(image_path: &Path) -> Result<()> {
    let current_wallpaper = get_current_wallpaper_windows().unwrap_or_else(|e| {
        warn!(target: "wallpaper", "读取当前 Windows 壁纸失败，将继续设置新壁纸: {e}");
        String::new()
    });

    let current_normalized = normalize_windows_path(Path::new(&current_wallpaper));
    let target_normalized = normalize_windows_path(image_path);

    if !current_wallpaper.is_empty() && current_normalized == target_normalized {
        info!(target: "wallpaper", "壁纸已设置为 {:?}，跳过设置", image_path);
        return Ok(());
    }

    info!(target: "wallpaper", "设置 Windows 壁纸为 {:?} (current: {:?})",
        image_path, current_wallpaper
    );

    let wide_path = image_path
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();

    let successful = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            wide_path.as_ptr() as *mut std::ffi::c_void,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        ) == 1
    };

    if successful {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("设置 Windows 壁纸失败")
    }
}

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

                // 计算实际可用的竖屏壁纸路径（不存在则视为 None，由 fallback 走横屏）
                let portrait_path = derive_portrait_path(expected).filter(|p| p.exists());

                // 检查是否所有显示器的壁纸都与期望一致（考虑屏幕方向 + 竖屏 fallback）
                let all_match = screen_orientations.iter().all(|screen| {
                    let expected_path = expected_path_for_screen(
                        screen,
                        expected.as_path(),
                        portrait_path.as_deref(),
                    );
                    actual.get(&screen.screen_index)
                        .map(|actual_path| actual_path.as_path() == expected_path)
                        .unwrap_or(false)
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

#[cfg(target_os = "windows")]
pub fn initialize_observer() {
    // Windows 不需要初始化
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

    // portrait_image_path 仅在 macOS 上使用（Windows 暂不支持竖屏壁纸）
    #[cfg(target_os = "windows")]
    let _ = portrait_image_path;

    // macOS 使用 NSWorkspace API 来处理多显示器和全屏场景
    #[cfg(target_os = "macos")]
    {
        set_wallpaper_macos(image_path, portrait_image_path)
    }

    // Windows 平台实现
    #[cfg(windows)]
    {
        set_wallpaper_windows(image_path)
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
    // 注意：当无竖屏壁纸时，竖屏屏幕的期望 = 横屏壁纸（fallback 行为），
    // 这样能将"竖屏 fallback 成功"识别为成功，不再误报失败。
    let all_match = !current_wallpapers.is_empty()
        && screen_orientations.iter().all(|screen| {
            let expected = expected_path_for_screen(
                screen,
                target_path.as_path(),
                target_portrait_path.as_deref(),
            );
            current_wallpapers
                .get(&screen.screen_index)
                .map(|actual| actual.as_path() == expected)
                .unwrap_or(false)
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
        // 注意：当无竖屏壁纸时，竖屏屏幕的期望 = 横屏壁纸（fallback 行为）。
        let all_success = screen_orientations.iter().all(|screen| {
            let expected = expected_path_for_screen(
                screen,
                target_path.as_path(),
                target_portrait_path.as_deref(),
            );
            actual
                .get(&screen.screen_index)
                .map(|actual_path| actual_path.as_path() == expected)
                .unwrap_or(false)
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
    // 重置 / 维护"竖屏 fallback 提示"去重状态：
    // 同一张横屏壁纸下，每个屏幕索引最多打一次 INFO（避免 observer 频繁触发刷屏）。
    if let Ok(mut state) = PORTRAIT_FALLBACK_NOTICE.lock() {
        let need_reset = state
            .landscape
            .as_deref()
            .map(|p| p != landscape_path)
            .unwrap_or(true);
        if need_reset {
            state.landscape = Some(landscape_path.to_path_buf());
            state.notified_screens.clear();
        }
    }

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
                            if should_emit_portrait_fallback_notice(i) {
                                info!(
                                    target: "wallpaper",
                                    "屏幕 {} 是竖屏，但竖屏壁纸不存在，将使用横屏壁纸",
                                    i
                                );
                            }
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

/// 获取所有屏幕的方向信息（Windows 平台）
#[cfg(target_os = "windows")]
pub fn get_screen_orientations() -> Vec<ScreenOrientation> {
    // Windows 平台暂时返回空数组
    vec![]
}

/// 根据屏幕方向计算"该屏幕期望显示的壁纸路径"。
///
/// - 横屏屏幕 → 横屏壁纸 (`landscape`)
/// - 竖屏屏幕：优先竖屏壁纸 (`portrait`)；若 `portrait = None` 则 fallback 到横屏壁纸
///
/// 抽出为纯函数以便：
/// 1. 在"是否需要重新设置"和"是否设置成功"两处校验之间共享同一份 fallback 语义；
/// 2. 单元测试覆盖 fallback 逻辑。
#[cfg(target_os = "macos")]
fn expected_path_for_screen<'a>(
    screen: &ScreenOrientation,
    landscape: &'a Path,
    portrait: Option<&'a Path>,
) -> &'a Path {
    if screen.is_portrait {
        portrait.unwrap_or(landscape)
    } else {
        landscape
    }
}

/// 由"横屏壁纸路径"派生"竖屏壁纸路径"（仅做路径推断，不检查文件是否存在）。
///
/// 规则：`/foo/20260326.jpg` -> `/foo/20260326r.jpg`
#[cfg(target_os = "macos")]
fn derive_portrait_path(landscape: &Path) -> Option<PathBuf> {
    let parent = landscape.parent()?;
    let stem = landscape.file_stem()?.to_str()?;
    Some(parent.join(format!("{}r.jpg", stem)))
}

/// 判断"竖屏 fallback 提示"是否应当输出（用于降噪）。
///
/// 同一张横屏壁纸下，每个屏幕索引最多触发一次。返回 true 表示本次需要打印。
#[cfg(target_os = "macos")]
fn should_emit_portrait_fallback_notice(screen_index: usize) -> bool {
    if let Ok(mut state) = PORTRAIT_FALLBACK_NOTICE.lock() {
        return state.notified_screens.insert(screen_index);
    }
    // 锁中毒等异常：保守起见允许输出
    true
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    use super::normalize_windows_path;
    #[cfg(target_os = "macos")]
    use super::*;
    #[cfg(windows)]
    use std::path::Path;

    #[cfg(windows)]
    #[test]
    fn windows_path_normalization_is_case_insensitive_and_uses_backslashes() {
        assert_eq!(
            normalize_windows_path(Path::new("C:/Temp/WallPaper.JPG")),
            r"c:\temp\wallpaper.jpg"
        );
    }

    #[cfg(target_os = "macos")]
    fn screen(index: usize, portrait: bool) -> ScreenOrientation {
        ScreenOrientation {
            screen_index: index,
            is_portrait: portrait,
            width: if portrait { 1080.0 } else { 1920.0 },
            height: if portrait { 1920.0 } else { 1080.0 },
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn landscape_screen_uses_landscape_path() {
        let landscape = PathBuf::from("/tmp/20260326.jpg");
        let portrait = PathBuf::from("/tmp/20260326r.jpg");
        let s = screen(0, false);
        assert_eq!(
            expected_path_for_screen(&s, landscape.as_path(), Some(portrait.as_path())),
            landscape.as_path()
        );
        assert_eq!(
            expected_path_for_screen(&s, landscape.as_path(), None),
            landscape.as_path()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn portrait_screen_prefers_portrait_path_when_available() {
        let landscape = PathBuf::from("/tmp/20260326.jpg");
        let portrait = PathBuf::from("/tmp/20260326r.jpg");
        let s = screen(2, true);
        assert_eq!(
            expected_path_for_screen(&s, landscape.as_path(), Some(portrait.as_path())),
            portrait.as_path()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn portrait_screen_falls_back_to_landscape_when_portrait_missing() {
        let landscape = PathBuf::from("/tmp/20260326.jpg");
        let s = screen(2, true);
        assert_eq!(
            expected_path_for_screen(&s, landscape.as_path(), None),
            landscape.as_path()
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn derive_portrait_path_appends_r_suffix() {
        assert_eq!(
            derive_portrait_path(Path::new("/foo/20260326.jpg")),
            Some(PathBuf::from("/foo/20260326r.jpg"))
        );
        assert_eq!(
            derive_portrait_path(Path::new("/Pictures/Bing/2026-04-11.png")),
            Some(PathBuf::from("/Pictures/Bing/2026-04-11r.jpg"))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn portrait_fallback_notice_emits_once_per_screen_per_landscape() {
        // 切换到一张新壁纸 -> 模拟 set_wallpaper_for_all_screens_by_orientation 入口的"重置"
        if let Ok(mut state) = PORTRAIT_FALLBACK_NOTICE.lock() {
            state.landscape = Some(PathBuf::from("/tmp/test-a.jpg"));
            state.notified_screens.clear();
        }
        assert!(should_emit_portrait_fallback_notice(2));
        // 同一壁纸 + 同一屏幕 -> 不再触发
        assert!(!should_emit_portrait_fallback_notice(2));
        // 不同屏幕 -> 触发一次
        assert!(should_emit_portrait_fallback_notice(3));
        assert!(!should_emit_portrait_fallback_notice(3));

        // 切换到另一张壁纸 -> 重置
        if let Ok(mut state) = PORTRAIT_FALLBACK_NOTICE.lock() {
            state.landscape = Some(PathBuf::from("/tmp/test-b.jpg"));
            state.notified_screens.clear();
        }
        assert!(should_emit_portrait_fallback_notice(2));
    }
}
