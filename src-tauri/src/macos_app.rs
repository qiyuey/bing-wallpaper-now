//! macOS 应用行为管理

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;

/// 设置应用为菜单栏应用（隐藏 Dock 图标）
#[cfg(target_os = "macos")]
pub fn set_activation_policy_accessory() {
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }
}

#[cfg(not(target_os = "macos"))]
pub fn set_activation_policy_accessory() {}
