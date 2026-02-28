use crate::index_manager::IndexManager;
use crate::models::{LocalWallpaper, WallpaperIndex};
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

/// 移除指定目录的 IndexManager 缓存
///
/// 用于导入/导出完成后清理临时目录的缓存条目，防止全局映射表无限增长。
/// 测试环境下为空操作（测试不使用全局缓存）。
pub fn remove_index_manager(directory: &Path) {
    #[cfg(test)]
    {
        let _ = directory;
    }

    #[cfg(not(test))]
    {
        let managers = INDEX_MANAGERS.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = managers.lock().unwrap();
        let key = directory
            .canonicalize()
            .unwrap_or_else(|_| directory.to_path_buf())
            .to_string_lossy()
            .to_string();
        map.remove(&key);
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
/// * `mkt` - 市场代码（如 "zh-CN", "en-US", "ja-JP"）
pub async fn get_local_wallpapers(directory: &Path, mkt: &str) -> Result<Vec<LocalWallpaper>> {
    let manager = get_index_manager(directory);
    manager.get_all_wallpapers(mkt).await
}

/// 获取 index.json 中所有可用的 mkt key
///
/// 复用全局 IndexManager 缓存，避免重复磁盘 I/O。
/// 用于 fallback 场景：当 effective_mkt 对应的壁纸为空时查找可用 key。
pub async fn get_available_mkt_keys(directory: &Path) -> Result<Vec<String>> {
    let manager = get_index_manager(directory);
    manager.get_available_mkt_keys().await
}

/// 获取当前壁纸目录索引的快照（优先使用缓存）
///
/// 返回 `WallpaperIndex` 的克隆，适用于只读场景（如导出）。
pub async fn get_index_snapshot(directory: &Path) -> Result<WallpaperIndex> {
    let manager = get_index_manager(directory);
    manager.load_index().await
}

/// 验证壁纸数据的市场代码是否匹配
///
/// 检查 urlbase 字段中的市场代码是否与期望的 mkt 匹配。
/// urlbase 格式通常为：/th?id=OHR.xxx_ZH-CN1234567890 或 /th?id=OHR.xxx_EN-US1234567890
///
/// 验证规则：
/// - urlbase 为空：通过验证（向后兼容）
/// - urlbase 包含 `_XX-YY` 格式的 mkt 标记且与期望 mkt 的大写形式匹配：通过
/// - urlbase 包含其他 mkt 标记且不包含期望 mkt 标记：验证失败
/// - urlbase 不包含任何已知 mkt 标记：通过（不做限制）
///
/// # Arguments
/// * `wallpaper` - 要验证的壁纸数据
/// * `expected_mkt` - 期望的市场代码（如 "zh-CN", "en-US", "ja-JP"）
///
/// # Returns
/// `true` 表示通过验证，`false` 表示市场代码不匹配
/// 预计算的 mkt 大写标记集合（如 "_ZH-CN", "_EN-US" 等）
///
/// 使用 LazyLock 在首次访问时构建，避免每次调用 validate_wallpaper_mkt 时重复格式化。
static MKT_UPPERCASE_MARKERS: std::sync::LazyLock<Vec<String>> = std::sync::LazyLock::new(|| {
    crate::utils::SUPPORTED_MKTS
        .iter()
        .map(|mkt| format!("_{}", mkt.to_uppercase()))
        .collect()
});

fn validate_wallpaper_mkt(wallpaper: &LocalWallpaper, expected_mkt: &str) -> bool {
    // 如果 urlbase 为空，不进行验证（向后兼容）
    if wallpaper.urlbase.is_empty() {
        return true;
    }

    let expected_marker = format!("_{}", expected_mkt.to_uppercase());

    // 快速路径：如果包含期望的标记，直接通过
    if wallpaper.urlbase.contains(&expected_marker) {
        return true;
    }

    // 检查是否包含其他已知 mkt 标记
    let has_other_marker = MKT_UPPERCASE_MARKERS
        .iter()
        .any(|m| wallpaper.urlbase.contains(m.as_str()));

    // 没有任何 mkt 标记 → 通过（向后兼容）
    // 有其他 mkt 标记但不含期望的 → 失败
    !has_other_marker
}

/// 过滤 mkt 不匹配的壁纸，返回通过验证的壁纸列表
fn filter_wallpapers_by_mkt(wallpapers: Vec<LocalWallpaper>, mkt: &str) -> Vec<LocalWallpaper> {
    let mut validated = Vec::new();
    let mut filtered_count = 0;

    for wallpaper in wallpapers {
        if !validate_wallpaper_mkt(&wallpaper, mkt) {
            log::warn!(
                "跳过 mkt 不匹配的壁纸: end_date={}, urlbase={}, 期望 mkt={}",
                wallpaper.end_date,
                wallpaper.urlbase,
                mkt
            );
            filtered_count += 1;
            continue;
        }
        validated.push(wallpaper);
    }

    if filtered_count > 0 {
        log::warn!(
            "过滤了 {} 条 mkt 不匹配的壁纸数据（期望 mkt: {}）",
            filtered_count,
            mkt
        );
    }

    validated
}

/// 壁纸元数据保存结果
pub struct SaveMetadataResult {
    /// 通过 mkt 验证的条目数
    pub validated: usize,
    /// 实际新增的条目数（不含覆盖已存在的）
    pub new_count: usize,
}

/// 批量保存壁纸元数据（使用全局缓存的 IndexManager）
///
/// 一次性保存多个壁纸，比多次调用 `save_wallpaper_metadata` 效率高得多。
///
/// # Arguments
/// * `wallpapers` - 要保存的壁纸列表
/// * `directory` - 壁纸存储目录
/// * `mkt` - 市场代码（如 "zh-CN", "en-US", "ja-JP"）
pub async fn save_wallpapers_metadata(
    wallpapers: Vec<LocalWallpaper>,
    directory: &Path,
    mkt: &str,
) -> Result<SaveMetadataResult> {
    let validated = filter_wallpapers_by_mkt(wallpapers, mkt);
    let validated_count = validated.len();
    let manager = get_index_manager(directory);
    let new_count = manager.upsert_wallpapers(validated, mkt).await?;
    Ok(SaveMetadataResult {
        validated: validated_count,
        new_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocalWallpaper;

    #[test]
    fn test_validate_wallpaper_mkt_zh_cn() {
        let wallpaper_zh = LocalWallpaper {
            title: "测试".to_string(),
            copyright: "测试版权".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_ZH-CN1234567890".to_string(),
        };

        assert!(validate_wallpaper_mkt(&wallpaper_zh, "zh-CN"));
        assert!(!validate_wallpaper_mkt(&wallpaper_zh, "en-US"));
    }

    #[test]
    fn test_validate_wallpaper_mkt_en_us() {
        let wallpaper_en = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
        };

        assert!(validate_wallpaper_mkt(&wallpaper_en, "en-US"));
        assert!(!validate_wallpaper_mkt(&wallpaper_en, "zh-CN"));
    }

    #[test]
    fn test_validate_wallpaper_mkt_ja_jp() {
        // 测试日语市场壁纸验证
        let wallpaper_jp = LocalWallpaper {
            title: "テスト".to_string(),
            copyright: "テスト著作権".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test_JA-JP1234567890".to_string(),
        };

        assert!(validate_wallpaper_mkt(&wallpaper_jp, "ja-JP"));
        assert!(!validate_wallpaper_mkt(&wallpaper_jp, "zh-CN"));
        assert!(!validate_wallpaper_mkt(&wallpaper_jp, "en-US"));
    }

    #[test]
    fn test_validate_wallpaper_mkt_empty_urlbase() {
        // 空 urlbase（向后兼容）应通过所有验证
        let wallpaper_empty = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "".to_string(),
        };

        assert!(validate_wallpaper_mkt(&wallpaper_empty, "zh-CN"));
        assert!(validate_wallpaper_mkt(&wallpaper_empty, "en-US"));
        assert!(validate_wallpaper_mkt(&wallpaper_empty, "ja-JP"));
    }

    #[test]
    fn test_validate_wallpaper_mkt_no_marker() {
        // 不包含任何 mkt 标记的 urlbase 应通过验证
        let wallpaper_no_marker = LocalWallpaper {
            title: "Test".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20250102".to_string(),
            urlbase: "/th?id=OHR.Test1234567890".to_string(),
        };

        assert!(validate_wallpaper_mkt(&wallpaper_no_marker, "zh-CN"));
        assert!(validate_wallpaper_mkt(&wallpaper_no_marker, "en-US"));
        assert!(validate_wallpaper_mkt(&wallpaper_no_marker, "ja-JP"));
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
