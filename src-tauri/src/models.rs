use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bing API 返回的图片条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BingImageEntry {
    pub url: String,
    pub urlbase: String,
    pub copyright: String,
    pub copyrightlink: String,
    pub title: String,
    pub startdate: String,
    pub enddate: String,
    pub hsh: String,
}

/// Bing API 响应结构
#[derive(Debug, Deserialize)]
pub struct BingImageArchive {
    pub images: Vec<BingImageEntry>,
}

/// 本地壁纸信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWallpaper {
    pub id: String,
    pub title: String,
    pub copyright: String,
    pub copyright_link: String,
    pub start_date: String,
    pub end_date: String,
    pub file_path: String,
    pub download_time: DateTime<Utc>,
}

impl From<BingImageEntry> for LocalWallpaper {
    fn from(entry: BingImageEntry) -> Self {
        Self {
            id: entry.hsh.clone(),
            title: entry.title.clone(),
            copyright: entry.copyright.clone(),
            copyright_link: entry.copyrightlink.clone(),
            start_date: entry.startdate.clone(),
            end_date: entry.enddate.clone(),
            file_path: String::new(), // 将在下载后设置
            download_time: Utc::now(),
        }
    }
}

/// 壁纸元数据索引（单一文件存储）
///
/// 索引版本号说明：
/// - v1: 初始版本，使用 MessagePack 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 壁纸列表（key = start_date）
    pub wallpapers: HashMap<String, LocalWallpaper>,
}

impl Default for WallpaperIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperIndex {
    /// 索引版本常量
    pub const VERSION: u32 = 1;

    /// 创建新索引
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            last_updated: Utc::now(),
            wallpapers: HashMap::new(),
        }
    }

    /// 获取壁纸数量
    pub fn len(&self) -> usize {
        self.wallpapers.len()
    }

    /// 判断是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.wallpapers.is_empty()
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_update: bool,
    pub save_directory: Option<String>,
    pub keep_image_count: u32,
    pub launch_at_startup: bool,
}

/// Migration helper: in future if more legacy fields are removed or value normalization is needed,
/// extend this method. Currently the legacy field `auto_apply_latest` is gone; serde silently ignores
/// it when deserializing persisted JSON, so we just return self unchanged.
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_update: true,
            save_directory: None,
            keep_image_count: 999,
            launch_at_startup: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.auto_update);
        assert_eq!(settings.save_directory, None);
        assert_eq!(settings.keep_image_count, 999);
        assert!(!settings.launch_at_startup);
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            auto_update: false,
            save_directory: Some("/custom/path".to_string()),
            keep_image_count: 20,
            launch_at_startup: true,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auto_update, settings.auto_update);
        assert_eq!(deserialized.save_directory, settings.save_directory);
        assert_eq!(deserialized.keep_image_count, settings.keep_image_count);
        assert_eq!(deserialized.launch_at_startup, settings.launch_at_startup);
    }

    #[test]
    fn test_bing_image_entry_to_local_wallpaper() {
        let entry = BingImageEntry {
            url: "https://example.com/image.jpg".to_string(),
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
            copyright: "Test Location (Test Author)".to_string(),
            copyrightlink: "https://example.com/details".to_string(),
            title: "Test Wallpaper".to_string(),
            startdate: "20240101".to_string(),
            enddate: "20240102".to_string(),
            hsh: "test_hash_123".to_string(),
        };

        let wallpaper = LocalWallpaper::from(entry.clone());

        assert_eq!(wallpaper.id, entry.hsh);
        assert_eq!(wallpaper.title, entry.title);
        assert_eq!(wallpaper.copyright, entry.copyright);
        assert_eq!(wallpaper.copyright_link, entry.copyrightlink);
        assert_eq!(wallpaper.start_date, entry.startdate);
        assert_eq!(wallpaper.end_date, entry.enddate);
        assert_eq!(wallpaper.file_path, ""); // Initially empty
    }

    #[test]
    fn test_local_wallpaper_serialization() {
        let now = Utc::now();
        let wallpaper = LocalWallpaper {
            id: "test_id".to_string(),
            title: "Test Title".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/path/to/wallpaper.jpg".to_string(),
            download_time: now,
        };

        let json = serde_json::to_string(&wallpaper).unwrap();
        let deserialized: LocalWallpaper = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, wallpaper.id);
        assert_eq!(deserialized.title, wallpaper.title);
        assert_eq!(deserialized.file_path, wallpaper.file_path);
    }

    #[test]
    fn test_app_settings_legacy_field_ignored() {
        // Simulate old JSON with removed field auto_apply_latest
        let json = r#"{
            "auto_update": true,
            "save_directory": null,
            "keep_image_count": 999,
            "launch_at_startup": false,
            "auto_apply_latest": true
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_update);
        assert_eq!(settings.keep_image_count, 999);
    }
}
