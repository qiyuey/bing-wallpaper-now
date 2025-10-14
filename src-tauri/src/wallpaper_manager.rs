use anyhow::Result;
use std::path::Path;

/// 设置桌面壁纸(跨平台)
///
/// # Arguments
/// * `image_path` - 壁纸图片的路径
pub fn set_wallpaper(image_path: &Path) -> Result<()> {
    if !image_path.exists() {
        anyhow::bail!("Wallpaper image does not exist: {:?}", image_path);
    }

    wallpaper::set_from_path(image_path.to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Failed to set wallpaper: {}", e))?;

    Ok(())
}

/// 获取当前的桌面壁纸路径
pub fn get_current_wallpaper() -> Result<String> {
    wallpaper::get()
        .map_err(|e| anyhow::anyhow!("Failed to get current wallpaper: {}", e))
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
