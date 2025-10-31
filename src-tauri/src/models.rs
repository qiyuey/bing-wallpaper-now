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
    #[serde(default)] // 为了兼容旧数据
    pub urlbase: String,
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
            urlbase: entry.urlbase.clone(),
        }
    }
}

/// 壁纸元数据索引（单一文件存储）
///
/// 索引版本号说明：
/// - v3: 支持多语言存储（按语言分组），内层 key 使用 end_date，与文件名保持一致
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 按语言分组的壁纸列表
    /// 外层 key = 语言代码（如 "zh-CN", "en-US"），内层 key = end_date
    /// 使用 end_date 作为 key，因为文件名也使用 end_date（Bing 的 startdate 是昨天，enddate 才是今天）
    pub wallpapers_by_language: HashMap<String, HashMap<String, LocalWallpaper>>,
}

impl Default for WallpaperIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperIndex {
    /// 索引版本常量
    pub const VERSION: u32 = 3;

    /// 创建新索引
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            last_updated: Utc::now(),
            wallpapers_by_language: HashMap::new(),
        }
    }

    /// 判断是否为空
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.wallpapers_by_language.is_empty()
    }

    /// 获取指定语言的壁纸列表
    pub fn get_wallpapers_for_language(&self, language: &str) -> Vec<LocalWallpaper> {
        self.wallpapers_by_language
            .get(language)
            .map(|wp_map| {
                let mut wallpapers: Vec<_> = wp_map.values().cloned().collect();
                wallpapers.sort_by(|a, b| b.end_date.cmp(&a.end_date));
                wallpapers
            })
            .unwrap_or_default()
    }

    /// 添加或更新指定语言的壁纸
    #[allow(dead_code)]
    pub fn upsert_wallpaper_for_language(&mut self, language: &str, wallpaper: LocalWallpaper) {
        self.wallpapers_by_language
            .entry(language.to_string())
            .or_default()
            .insert(wallpaper.end_date.clone(), wallpaper);
        self.last_updated = Utc::now();
    }

    /// 批量添加或更新指定语言的壁纸
    pub fn upsert_wallpapers_for_language(
        &mut self,
        language: &str,
        wallpapers: Vec<LocalWallpaper>,
    ) {
        if wallpapers.is_empty() {
            return;
        }
        let lang_map = self
            .wallpapers_by_language
            .entry(language.to_string())
            .or_default();
        for wallpaper in wallpapers {
            lang_map.insert(wallpaper.end_date.clone(), wallpaper);
        }
        self.last_updated = Utc::now();
    }

    /// 获取所有语言的壁纸（用于清理操作）
    /// 返回所有语言中唯一的 end_date 对应的壁纸列表
    /// 如果有多个语言存在相同 end_date，优先选择字典序靠前的语言
    pub fn get_all_wallpapers_unique(&self) -> Vec<LocalWallpaper> {
        use std::collections::{BTreeMap, HashSet};
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        // 使用 BTreeMap 按语言代码排序，确保一致性
        let lang_order: BTreeMap<_, _> = self.wallpapers_by_language.iter().collect();

        // 按语言代码顺序遍历，优先选择字典序靠前的语言
        for (_, lang_wallpapers) in lang_order {
            for wallpaper in lang_wallpapers.values() {
                if seen.insert(wallpaper.end_date.clone()) {
                    result.push(wallpaper.clone());
                }
            }
        }

        // 按 end_date 降序排序（最新的在前）
        result.sort_by(|a, b| b.end_date.cmp(&a.end_date));
        result
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_update: bool,
    pub save_directory: Option<String>,
    pub keep_image_count: u32,
    pub launch_at_startup: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
}

/// 默认主题设置
fn default_theme() -> String {
    "system".to_string()
}

/// 默认语言设置
fn default_language() -> String {
    "auto".to_string()
}

/// Migration helper: in future if more legacy fields are removed or value normalization is needed,
/// extend this method. Currently the legacy field `auto_apply_latest` is gone; serde silently ignores
/// it when deserializing persisted JSON, so we just return self unchanged.
impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_update: true,
            save_directory: None,
            keep_image_count: 0,
            launch_at_startup: false,
            theme: default_theme(),
            language: default_language(),
        }
    }
}

/// 应用内部运行时状态（不展示给用户）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppRuntimeState {
    /// 最后成功更新时间（ISO 8601 格式）
    pub last_successful_update: Option<String>,
    /// 最后检查更新时间（ISO 8601 格式）
    pub last_check_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.auto_update);
        assert_eq!(settings.save_directory, None);
        assert_eq!(settings.keep_image_count, 0);
        assert!(!settings.launch_at_startup);
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            auto_update: false,
            save_directory: Some("/custom/path".to_string()),
            keep_image_count: 20,
            launch_at_startup: true,
            theme: "dark".to_string(),
            language: "auto".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auto_update, settings.auto_update);
        assert_eq!(deserialized.save_directory, settings.save_directory);
        assert_eq!(deserialized.keep_image_count, settings.keep_image_count);
        assert_eq!(deserialized.launch_at_startup, settings.launch_at_startup);
        assert_eq!(deserialized.theme, settings.theme);
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
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
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
            "keep_image_count": 10000,
            "launch_at_startup": false,
            "auto_apply_latest": true
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_update);
        assert_eq!(settings.keep_image_count, 10000);
    }
}
