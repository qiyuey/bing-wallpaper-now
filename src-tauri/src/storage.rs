use crate::index_manager::IndexManager;
use crate::models::LocalWallpaper;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

#[cfg(not(test))]
use std::collections::HashMap;
#[cfg(not(test))]
use std::sync::{Mutex, OnceLock};

/// 全局索引管理器映射表（支持多目录）
/// Key: 目录路径的规范化字符串
/// Value: 对应目录的 IndexManager
#[cfg(not(test))]
static INDEX_MANAGERS: OnceLock<Mutex<HashMap<String, Arc<IndexManager>>>> = OnceLock::new();

/// 获取索引管理器
///
/// 在生产环境中使用全局映射表管理多个目录的 IndexManager；
/// 在测试环境中为每个目录创建新实例
fn get_index_manager(directory: &Path) -> Arc<IndexManager> {
    #[cfg(test)]
    {
        // 测试环境：为每个目录创建独立的 IndexManager 实例
        Arc::new(IndexManager::new(directory.to_path_buf()))
    }

    #[cfg(not(test))]
    {
        // 生产环境：使用全局映射表，支持多目录
        let managers = INDEX_MANAGERS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = managers.lock().unwrap();

        // 使用规范化的路径作为 key
        let key = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf())
            .to_string_lossy()
            .to_string();

        map.entry(key)
            .or_insert_with(|| Arc::new(IndexManager::new(directory.to_path_buf())))
            .clone()
    }
}

/// 获取默认的壁纸存储目录
pub fn get_default_wallpaper_directory() -> Result<PathBuf> {
    // Primary attempt: use OS-specific pictures directory
    if let Some(pictures) = dirs::picture_dir() {
        return Ok(pictures.join("Bing Wallpaper Now"));
    }

    // Fallback: construct ~/Pictures (cross-platform) then append app folder
    if let Some(home) = dirs::home_dir() {
        let pictures = home.join("Pictures");
        return Ok(pictures.join("Bing Wallpaper Now"));
    }

    // Both strategies failed
    anyhow::bail!(
        "Failed to resolve pictures directory (dirs::picture_dir() and home_dir() both returned None)"
    );
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

/// 获取所有已下载的壁纸（使用索引）
///
/// 优先从索引加载，大幅提升性能。
///
/// # Arguments
/// * `directory` - 壁纸存储目录
/// * `language` - 语言代码（如 "zh-CN", "en-US"）
pub async fn get_local_wallpapers(directory: &Path, language: &str) -> Result<Vec<LocalWallpaper>> {
    let manager = get_index_manager(directory);
    manager.get_all_wallpapers(language).await
}

/// 批量保存壁纸元数据（性能优化）
///
/// 一次性保存多个壁纸，比多次调用 `save_wallpaper_metadata` 效率高得多。
///
/// # Arguments
/// * `wallpapers` - 要保存的壁纸列表
/// * `directory` - 壁纸存储目录
/// * `language` - 语言代码（如 "zh-CN", "en-US"）
pub async fn save_wallpapers_metadata(
    wallpapers: Vec<LocalWallpaper>,
    directory: &Path,
    language: &str,
) -> Result<()> {
    let manager = get_index_manager(directory);
    manager.upsert_wallpapers(wallpapers, language).await
}

/// 删除旧的壁纸，只保留指定数量（使用索引）
///
/// 自动删除图片文件、旧 JSON 元数据文件，并更新索引。
/// 如果 keep_count 为 0，表示不限制数量，但至少保留 8 张。
/// 清理时会考虑所有语言的壁纸，只删除在所有语言中都不再需要的文件。
pub async fn cleanup_old_wallpapers(directory: &Path, keep_count: usize) -> Result<usize> {
    let manager = get_index_manager(directory);
    let mut wallpapers = manager.get_all_wallpapers_unique().await?;

    // 0 表示不限制数量，但至少保留 8 张
    if keep_count == 0 {
        // 如果总数小于等于 8，不删除任何壁纸
        if wallpapers.len() <= 8 {
            return Ok(0);
        }
        // 否则保留全部（不限制），不删除任何壁纸
        return Ok(0);
    }

    // 非 0 情况：如果总数小于等于保留数量，不删除
    if wallpapers.len() <= keep_count {
        return Ok(0);
    }

    // 排序后删除旧的（按 end_date 降序，最新的在前）
    wallpapers.sort_by(|a, b| b.end_date.cmp(&a.end_date));
    let to_delete = wallpapers.split_off(keep_count);

    // 收集要删除的 start_date，并跟踪成功删除的文件
    let mut failed_deletes = Vec::new();
    let mut successful_deletes = Vec::new();

    // 删除文件
    for wallpaper in &to_delete {
        let image_path = Path::new(&wallpaper.file_path);
        let mut delete_success = true;

        // 删除图片文件
        if image_path.exists()
            && let Err(e) = fs::remove_file(image_path).await
        {
            log::warn!("删除图片文件失败: {} - {}", image_path.display(), e);
            delete_success = false;
        }

        // 删除旧的 JSON 元数据文件（如果存在）
        let json_path = image_path.with_extension("json");
        if json_path.exists()
            && let Err(e) = fs::remove_file(&json_path).await
        {
            log::warn!("删除 JSON 元数据文件失败: {} - {}", json_path.display(), e);
            // JSON 文件删除失败不影响整体删除操作
        }

        if delete_success {
            successful_deletes.push(wallpaper.start_date.clone());
        } else {
            failed_deletes.push(wallpaper.start_date.clone());
        }
    }

    // 只从索引中删除成功删除文件的条目
    if !successful_deletes.is_empty() {
        manager.remove_wallpapers(&successful_deletes).await?;
    }

    if !failed_deletes.is_empty() {
        log::warn!(
            "部分文件删除失败，这些条目的索引未被更新: {:?}",
            failed_deletes
        );
    }

    Ok(successful_deletes.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocalWallpaper;
    use chrono::Utc;
    use std::time::SystemTime;
    use tokio::fs;

    #[test]
    fn test_get_default_wallpaper_directory() {
        let dir_result = get_default_wallpaper_directory();
        assert!(
            dir_result.is_ok(),
            "Failed to get default wallpaper directory. OS: {:?}, HOME: {:?}, Result: {:?}",
            std::env::consts::OS,
            std::env::var("HOME").ok(),
            dir_result.as_ref().err()
        );
        let dir = dir_result.unwrap();
        assert!(
            dir.to_string_lossy().contains("Bing Wallpaper Now"),
            "Directory path {:?} does not contain expected segment 'Bing Wallpaper Now'",
            dir
        );
    }

    #[test]
    fn test_get_wallpaper_path() {
        let dir = PathBuf::from("/tmp/wallpapers");
        let path = get_wallpaper_path(&dir, "20240315");
        assert_eq!(path, PathBuf::from("/tmp/wallpapers/20240315.jpg"));
    }

    // 创建若干假壁纸文件与元数据
    async fn create_fake_wallpaper(dir: &Path, start_date: &str) -> LocalWallpaper {
        let img_path = get_wallpaper_path(dir, start_date);
        fs::write(&img_path, b"").await.unwrap();

        LocalWallpaper {
            id: format!("id{}", start_date),
            title: format!("Title {}", start_date),
            copyright: "Copyright".into(),
            copyright_link: "https://example.com".into(),
            start_date: start_date.into(),
            end_date: start_date.into(),
            file_path: img_path.to_string_lossy().to_string(),
            download_time: Utc::now(),
            urlbase: format!("/th?id=OHR.Wallpaper{}", start_date),
        }
    }

    #[tokio::test]
    async fn test_cleanup_old_wallpapers_keeps_limit() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_keep_limit_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        // 创建 5 张壁纸
        let mut wallpapers = Vec::new();
        for d in ["20240101", "20240102", "20240103", "20240104", "20240105"] {
            wallpapers.push(create_fake_wallpaper(&temp_dir, d).await);
        }

        // 批量保存元数据到索引（使用默认语言 zh-CN）
        save_wallpapers_metadata(wallpapers, &temp_dir, "zh-CN")
            .await
            .unwrap();

        // 保留 3 张
        let deleted = cleanup_old_wallpapers(&temp_dir, 3).await.unwrap();
        assert_eq!(deleted, 2, "应删除 2 张旧壁纸");

        let remaining = get_local_wallpapers(&temp_dir, "zh-CN").await.unwrap();
        assert_eq!(remaining.len(), 3);

        // 最新的三个日期应该保留
        let dates: Vec<_> = remaining.iter().map(|w| w.start_date.clone()).collect();
        assert!(dates.contains(&"20240105".to_string()));
        assert!(dates.contains(&"20240104".to_string()));
        assert!(dates.contains(&"20240103".to_string()));
        assert!(!dates.contains(&"20240101".to_string()));
        assert!(!dates.contains(&"20240102".to_string()));
    }

    #[tokio::test]
    async fn test_cleanup_old_wallpapers_no_deletion_when_under_limit() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_under_limit_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        // 创建 2 张壁纸
        let mut wallpapers = Vec::new();
        for d in ["20240201", "20240202"] {
            wallpapers.push(create_fake_wallpaper(&temp_dir, d).await);
        }

        // 批量保存元数据到索引（使用默认语言 zh-CN）
        save_wallpapers_metadata(wallpapers, &temp_dir, "zh-CN")
            .await
            .unwrap();

        // 保留数量设置为 5，不应删除
        let deleted = cleanup_old_wallpapers(&temp_dir, 5).await.unwrap();
        assert_eq!(deleted, 0);

        let remaining = get_local_wallpapers(&temp_dir, "zh-CN").await.unwrap();
        assert_eq!(remaining.len(), 2);
    }
}
