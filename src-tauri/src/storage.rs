use crate::models::LocalWallpaper;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

/// 获取默认的壁纸存储目录
pub fn get_default_wallpaper_directory() -> Result<PathBuf> {
    let pictures_dir = dirs::picture_dir()
        .context("Failed to get pictures directory")?;

    Ok(pictures_dir.join("Bing Wallpaper Now"))
}

/// 确保壁纸目录存在
pub async fn ensure_wallpaper_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)
            .await
            .context("Failed to create wallpaper directory")?;
    }
    Ok(())
}

/// 获取壁纸的保存路径
pub fn get_wallpaper_path(directory: &Path, start_date: &str) -> PathBuf {
    directory.join(format!("{}.jpg", start_date))
}

/// 获取所有已下载的壁纸
pub async fn get_local_wallpapers(directory: &Path) -> Result<Vec<LocalWallpaper>> {
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(directory)
        .await
        .context("Failed to read wallpaper directory")?;

    let mut wallpapers = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jpg") {
            // 尝试读取元数据文件
            let metadata_path = path.with_extension("json");
            if let Ok(metadata_content) = fs::read_to_string(&metadata_path).await {
                if let Ok(wallpaper) = serde_json::from_str::<LocalWallpaper>(&metadata_content) {
                    wallpapers.push(wallpaper);
                }
            }
        }
    }

    // 按日期排序,最新的在前
    wallpapers.sort_by(|a, b| b.start_date.cmp(&a.start_date));

    Ok(wallpapers)
}

/// 保存壁纸元数据
pub async fn save_wallpaper_metadata(wallpaper: &LocalWallpaper, directory: &Path) -> Result<()> {
    let metadata_path = directory.join(format!("{}.json", wallpaper.start_date));

    let json = serde_json::to_string_pretty(wallpaper)
        .context("Failed to serialize wallpaper metadata")?;

    fs::write(&metadata_path, json)
        .await
        .context("Failed to write wallpaper metadata")?;

    Ok(())
}

/// 删除旧的壁纸,只保留指定数量
pub async fn cleanup_old_wallpapers(directory: &Path, keep_count: usize) -> Result<usize> {
    let mut wallpapers = get_local_wallpapers(directory).await?;

    if wallpapers.len() <= keep_count {
        return Ok(0);
    }

    // 排序后删除旧的
    wallpapers.sort_by(|a, b| b.start_date.cmp(&a.start_date));

    let to_delete = wallpapers.split_off(keep_count);
    let deleted_count = to_delete.len();

    for wallpaper in to_delete {
        let image_path = Path::new(&wallpaper.file_path);
        let metadata_path = image_path.with_extension("json");

        // 删除图片文件
        if image_path.exists() {
            let _ = fs::remove_file(image_path).await;
        }

        // 删除元数据文件
        if metadata_path.exists() {
            let _ = fs::remove_file(&metadata_path).await;
        }
    }

    Ok(deleted_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_default_wallpaper_directory() {
        let dir = get_default_wallpaper_directory();
        assert!(dir.is_ok());
        let dir = dir.unwrap();
        assert!(dir.to_string_lossy().contains("Bing Wallpaper Now"));
    }

    #[test]
    fn test_get_wallpaper_path() {
        let dir = PathBuf::from("/tmp/wallpapers");
        let path = get_wallpaper_path(&dir, "20240315");
        assert_eq!(path, PathBuf::from("/tmp/wallpapers/20240315.jpg"));
    }
}
