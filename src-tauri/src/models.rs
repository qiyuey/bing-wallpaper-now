use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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
///
/// 使用短字段名以节省存储空间：
/// - title -> t
/// - copyright -> c
/// - copyright_link -> l
/// - end_date -> d (保留，因为代码中广泛使用)
/// - urlbase -> u
/// - hsh -> h (MD5 哈希值，用于校验文件完整性，可选)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalWallpaper {
    #[serde(rename = "t")]
    pub title: String,
    #[serde(rename = "c")]
    pub copyright: String,
    #[serde(rename = "l")]
    pub copyright_link: String,
    #[serde(rename = "d")]
    pub end_date: String,
    #[serde(rename = "u", default)]
    pub urlbase: String,
    #[serde(rename = "h", default, skip_serializing_if = "String::is_empty")]
    pub hsh: String,
}

impl From<BingImageEntry> for LocalWallpaper {
    fn from(entry: BingImageEntry) -> Self {
        Self {
            title: entry.title.clone(),
            copyright: entry.copyright.clone(),
            copyright_link: entry.copyrightlink.clone(),
            end_date: entry.enddate.clone(),
            urlbase: entry.urlbase.clone(),
            hsh: entry.hsh.clone(),
        }
    }
}

/// 壁纸元数据索引（单一文件存储）
///
/// 索引版本号说明：
/// - v4: 使用短字段名和紧凑格式以节省存储空间
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 按语言分组的壁纸列表
    /// 外层 key = 语言代码（如 "zh-CN", "en-US"），内层 key = end_date
    /// 使用 end_date 作为 key，因为文件名也使用 end_date（Bing 的 startdate 是昨天，enddate 才是今天）
    /// 使用 IndexMap 以保持插入顺序，确保 JSON 序列化时按日期排序
    pub wallpapers_by_language: IndexMap<String, IndexMap<String, LocalWallpaper>>,
}

impl Default for WallpaperIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperIndex {
    /// 索引版本常量
    ///
    /// v4: 使用短字段名和紧凑格式
    pub const VERSION: u32 = 4;

    /// 创建新索引
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            last_updated: Utc::now(),
            wallpapers_by_language: IndexMap::new(),
        }
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

    /// 批量添加或更新指定语言的壁纸
    /// 插入时会按日期降序排序，确保 JSON 序列化时保持顺序
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

        // 先插入所有壁纸
        for wallpaper in wallpapers {
            lang_map.insert(wallpaper.end_date.clone(), wallpaper);
        }

        // 按日期降序排序（最新的在前）
        lang_map.sort_by(|k1, _, k2, _| k2.cmp(k1));

        // 对外层（语言）也按字典序排序，确保 JSON 中的语言顺序一致
        self.wallpapers_by_language.sort_keys();

        self.last_updated = Utc::now();
    }

    /// 对所有语言和日期进行排序，确保 JSON 序列化时保持顺序
    pub fn sort_all(&mut self) {
        // 对每个语言的壁纸按日期降序排序
        for lang_wallpapers in self.wallpapers_by_language.values_mut() {
            lang_wallpapers.sort_by(|k1, _, k2, _| k2.cmp(k1));
        }
        // 对外层（语言）按字典序排序
        self.wallpapers_by_language.sort_keys();
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

    /// 限制索引大小，保留最新的条目
    ///
    /// 如果索引总数超过 `max_count`，会删除最旧的条目。
    /// 优先保留最新的条目，按 end_date 降序排序。
    ///
    /// # Arguments
    /// * `max_count` - 最大索引数量
    pub fn limit_index_size(&mut self, max_count: usize) {
        // 获取所有唯一的 end_date，按降序排序（最新的在前）
        let all_unique = self.get_all_wallpapers_unique();

        // 如果总数不超过限制，不需要清理
        if all_unique.len() <= max_count {
            return;
        }

        // 需要删除的 end_date 列表（最旧的）
        let to_remove: Vec<String> = all_unique
            .iter()
            .skip(max_count)
            .map(|w| w.end_date.clone())
            .collect();

        log::info!(
            "索引数据超过限制 ({} > {})，删除 {} 条最旧的索引条目",
            all_unique.len(),
            max_count,
            to_remove.len()
        );

        // 从所有语言中删除这些 end_date
        for lang_wallpapers in self.wallpapers_by_language.values_mut() {
            for end_date in &to_remove {
                lang_wallpapers.shift_remove(end_date);
            }
        }

        // 移除空的语言分组
        self.wallpapers_by_language
            .retain(|_, lang_wallpapers| !lang_wallpapers.is_empty());

        self.last_updated = Utc::now();
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_update: bool,
    pub save_directory: Option<String>,
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
    /// 用户手动设置壁纸时，各语言的最新壁纸标识（key = 语言代码，value = end_date）
    /// 用于判断自动更新时是否需要跳过相同的壁纸
    #[serde(default)]
    pub manually_set_latest_wallpapers: std::collections::HashMap<String, String>,
    /// 用户选择"不再提醒"的最大版本号（如果最新版本小于等于此版本，则不提示）
    #[serde(default)]
    pub ignored_update_version: Option<String>,
    /// 自启动通知已显示标志（用于避免 macOS 系统重复显示自启动通知）
    /// 当用户首次启用自启动时设置为 true，表示用户已经看到过系统通知
    #[serde(default)]
    pub autostart_notification_shown: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.auto_update);
        assert_eq!(settings.save_directory, None);
        assert!(!settings.launch_at_startup);
    }

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            auto_update: false,
            save_directory: Some("/custom/path".to_string()),
            launch_at_startup: true,
            theme: "dark".to_string(),
            language: "auto".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auto_update, settings.auto_update);
        assert_eq!(deserialized.save_directory, settings.save_directory);
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

        assert_eq!(wallpaper.title, entry.title);
        assert_eq!(wallpaper.copyright, entry.copyright);
        assert_eq!(wallpaper.copyright_link, entry.copyrightlink);
        assert_eq!(wallpaper.end_date, entry.enddate);
        assert_eq!(wallpaper.hsh, entry.hsh);
    }

    #[test]
    fn test_local_wallpaper_serialization() {
        let wallpaper = LocalWallpaper {
            title: "Test Title".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
            hsh: "test_hash_456".to_string(),
        };

        let json = serde_json::to_string(&wallpaper).unwrap();
        let deserialized: LocalWallpaper = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.title, wallpaper.title);
        assert_eq!(deserialized.end_date, wallpaper.end_date);
    }

    #[test]
    fn test_app_settings_legacy_field_ignored() {
        // Simulate old JSON with removed field keep_image_count
        let json = r#"{
            "auto_update": true,
            "save_directory": null,
            "launch_at_startup": false,
            "theme": "system",
            "language": "auto"
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_update);
        assert_eq!(settings.theme, "system");
    }
}
