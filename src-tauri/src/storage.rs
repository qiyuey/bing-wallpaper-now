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
/// 使用 end_date 作为文件名，因为 Bing 的壁纸 startdate 是昨天，enddate 才是今天
pub fn get_wallpaper_path(directory: &Path, end_date: &str) -> PathBuf {
    directory.join(format!("{}.jpg", end_date))
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

/// 验证壁纸数据的语言是否匹配
///
/// 检查 urlbase 字段中的语言代码是否与期望的语言匹配。
/// urlbase 格式通常为：/th?id=OHR.xxx_ZH-CN1234567890 或 /th?id=OHR.xxx_EN-US1234567890
///
/// # Arguments
/// * `wallpaper` - 要验证的壁纸数据
/// * `expected_language` - 期望的语言代码（如 "zh-CN", "en-US"）
///
/// # Returns
/// `true` 表示通过验证，`false` 表示语言不匹配
fn validate_wallpaper_language(wallpaper: &LocalWallpaper, expected_language: &str) -> bool {
    let expected_lang_in_url = match expected_language {
        "zh-CN" => "_ZH-CN",
        "en-US" => "_EN-US",
        _ => return true, // 其他语言不验证，直接通过
    };

    // 如果 urlbase 为空，不进行验证（向后兼容）
    if wallpaper.urlbase.is_empty() {
        return true;
    }

    // 检查是否包含其他语言的代码
    let contains_other_lang = match expected_language {
        "zh-CN" => wallpaper.urlbase.contains("_EN-US"),
        "en-US" => wallpaper.urlbase.contains("_ZH-CN"),
        _ => false,
    };

    // 如果包含其他语言代码，且不包含预期语言代码，则验证失败
    !contains_other_lang || wallpaper.urlbase.contains(expected_lang_in_url)
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
    // 验证数据语言匹配：过滤掉语言不匹配的条目
    let mut validated_wallpapers = Vec::new();
    let mut filtered_count = 0;

    for wallpaper in wallpapers {
        if !validate_wallpaper_language(&wallpaper, language) {
            // 检测到语言不匹配，记录警告并跳过
            log::warn!(
                "跳过语言不匹配的壁纸: end_date={}, urlbase={}, 期望语言={}",
                wallpaper.end_date,
                wallpaper.urlbase,
                language
            );
            filtered_count += 1;
            continue;
        }
        validated_wallpapers.push(wallpaper);
    }

    if filtered_count > 0 {
        log::warn!(
            "过滤了 {} 条语言不匹配的壁纸数据（期望语言: {}）",
            filtered_count,
            language
        );
    }

    let manager = get_index_manager(directory);
    manager
        .upsert_wallpapers(validated_wallpapers, language)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocalWallpaper;

    #[test]
    fn test_validate_wallpaper_language_zh_cn() {
        // 测试中文壁纸验证
        let wallpaper_zh = LocalWallpaper {
            title: "测试".to_string(),
            copyright: "测试版权".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_ZH-CN1234567890".to_string(),
            hsh: "".to_string(),
        };

        assert!(validate_wallpaper_language(&wallpaper_zh, "zh-CN"));
        assert!(!validate_wallpaper_language(&wallpaper_zh, "en-US"));
    }

    #[test]
    fn test_validate_wallpaper_language_en_us() {
        // 测试英文壁纸验证
        let wallpaper_en = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
            hsh: "".to_string(),
        };

        assert!(validate_wallpaper_language(&wallpaper_en, "en-US"));
        assert!(!validate_wallpaper_language(&wallpaper_en, "zh-CN"));
    }

    #[test]
    fn test_validate_wallpaper_language_empty_urlbase() {
        // 测试空 urlbase（向后兼容）
        let wallpaper_empty = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "".to_string(),
            hsh: "".to_string(),
        };

        assert!(validate_wallpaper_language(&wallpaper_empty, "zh-CN"));
        assert!(validate_wallpaper_language(&wallpaper_empty, "en-US"));
    }

    #[test]
    fn test_validate_wallpaper_language_no_lang_marker() {
        // 测试不包含语言标记的 urlbase
        let wallpaper_no_marker = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test1234567890".to_string(),
            hsh: "".to_string(),
        };

        assert!(validate_wallpaper_language(&wallpaper_no_marker, "zh-CN"));
        assert!(validate_wallpaper_language(&wallpaper_no_marker, "en-US"));
    }

    #[test]
    fn test_validate_wallpaper_language_unknown_language() {
        // 测试未知语言（应该始终通过验证）
        let wallpaper = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_ZH-CN1234567890".to_string(),
            hsh: "".to_string(),
        };

        assert!(validate_wallpaper_language(&wallpaper, "unknown"));
    }

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
}
