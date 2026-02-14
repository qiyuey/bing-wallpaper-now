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
}

impl From<BingImageEntry> for LocalWallpaper {
    fn from(entry: BingImageEntry) -> Self {
        Self {
            title: entry.title.clone(),
            copyright: entry.copyright.clone(),
            copyright_link: entry.copyrightlink.clone(),
            end_date: entry.enddate.clone(),
            urlbase: entry.urlbase.clone(),
        }
    }
}

/// 壁纸元数据索引（单一文件存储）
///
/// 索引版本号说明：
/// - v4: 使用短字段名和紧凑格式，壁纸按 `wallpapers_by_language` 分组
/// - v5: 将 `wallpapers_by_language` 重命名为 `mkt`，语义更准确
///
/// 迁移说明：
/// - v4 → v5：自动备份旧文件为 `index.json.v4.bak`，将 `wallpapers_by_language` 迁移为 `mkt`
/// - 通过 `#[serde(alias = "wallpapers_by_language")]` 保证反序列化兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
    /// 按市场代码（mkt）分组的壁纸列表
    /// 外层 key = mkt（如 "zh-CN", "en-US", "ja-JP"），内层 key = end_date
    /// 使用 end_date 作为 key，因为文件名也使用 end_date（Bing 的 startdate 是昨天，enddate 才是今天）
    /// 使用 IndexMap 以保持插入顺序，确保 JSON 序列化时按日期排序
    #[serde(alias = "wallpapers_by_language")]
    pub mkt: IndexMap<String, IndexMap<String, LocalWallpaper>>,
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
    /// v5: wallpapers_by_language → mkt
    pub const VERSION: u32 = 5;

    /// 支持从此版本迁移升级（v4 → v5）
    pub const MIGRATE_FROM_VERSION: u32 = 4;

    /// 创建新索引
    pub fn new() -> Self {
        Self {
            version: Self::VERSION,
            last_updated: Utc::now(),
            mkt: IndexMap::new(),
        }
    }

    /// 获取指定 mkt 的壁纸列表
    pub fn get_wallpapers_for_mkt(&self, mkt: &str) -> Vec<LocalWallpaper> {
        self.mkt
            .get(mkt)
            .map(|wp_map| {
                let mut wallpapers: Vec<_> = wp_map.values().cloned().collect();
                wallpapers.sort_by(|a, b| b.end_date.cmp(&a.end_date));
                wallpapers
            })
            .unwrap_or_default()
    }

    /// 批量添加或更新指定 mkt 的壁纸
    /// 插入时会按日期降序排序，确保 JSON 序列化时保持顺序
    pub fn upsert_wallpapers_for_mkt(
        &mut self,
        mkt: &str,
        wallpapers: Vec<LocalWallpaper>,
    ) {
        if wallpapers.is_empty() {
            return;
        }
        let mkt_map = self
            .mkt
            .entry(mkt.to_string())
            .or_default();

        // 先插入所有壁纸
        for wallpaper in wallpapers {
            mkt_map.insert(wallpaper.end_date.clone(), wallpaper);
        }

        // 按日期降序排序（最新的在前）
        mkt_map.sort_by(|k1, _, k2, _| k2.cmp(k1));

        // 对外层（mkt）也按字典序排序，确保 JSON 中的 mkt 顺序一致
        self.mkt.sort_keys();

        self.last_updated = Utc::now();
    }

    /// 对所有 mkt 和日期进行排序，确保 JSON 序列化时保持顺序
    pub fn sort_all(&mut self) {
        // 对每个 mkt 的壁纸按日期降序排序
        for mkt_wallpapers in self.mkt.values_mut() {
            mkt_wallpapers.sort_by(|k1, _, k2, _| k2.cmp(k1));
        }
        // 对外层（mkt）按字典序排序
        self.mkt.sort_keys();
    }

    /// 获取所有语言的壁纸（用于清理操作）
    /// 返回所有语言中唯一的 end_date 对应的壁纸列表
    /// 如果有多个语言存在相同 end_date，优先选择字典序靠前的语言
    pub fn get_all_wallpapers_unique(&self) -> Vec<LocalWallpaper> {
        use std::collections::{BTreeMap, HashSet};
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        // 使用 BTreeMap 按语言代码排序，确保一致性
        let lang_order: BTreeMap<_, _> = self.mkt.iter().collect();

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
        for lang_wallpapers in self.mkt.values_mut() {
            for end_date in &to_remove {
                lang_wallpapers.shift_remove(end_date);
            }
        }

        // 移除空的语言分组
        self.mkt
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
    /// 解析后的语言（"auto" 被解析为具体语言 "zh-CN" 或 "en-US"）
    ///
    /// 此字段由 get_settings 命令计算填充，不需要前端传入。
    /// 前端 i18n 应使用此字段，而 language 字段仅用于设置 UI 回显。
    #[serde(default)]
    pub resolved_language: String,
    /// Bing API 市场代码（如 "zh-CN", "en-US", "ja-JP" 等）
    ///
    /// 与 UI 语言 (language) 独立，决定从 Bing 获取哪个地区的壁纸内容。
    /// 默认为空字符串，normalize_mkt() 会将其回退到 resolved_language。
    #[serde(default)]
    pub mkt: String,
}

/// 默认主题设置
fn default_theme() -> String {
    "system".to_string()
}

/// 默认语言设置
///
/// 默认为 "auto"，运行时通过系统语言检测决定使用中文还是英文
fn default_language() -> String {
    "auto".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        let lang = default_language();
        let resolved = crate::utils::resolve_language(&lang).to_string();
        let mkt = resolved.clone(); // mkt 默认跟随 resolved_language
        Self {
            auto_update: true,
            save_directory: None,
            launch_at_startup: false,
            theme: default_theme(),
            language: lang,
            resolved_language: resolved,
            mkt,
        }
    }
}

impl AppSettings {
    /// 归一化语言设置
    ///
    /// "auto"、"zh-CN"、"en-US" 是有效值，保持不变。
    /// 其他无效值（如旧版本遗留的非标准语言代码）通过系统语言检测归一化。
    pub fn normalize_language(&mut self) {
        match self.language.as_str() {
            "auto" | "zh-CN" | "en-US" => {} // 有效值，不变
            _ => {
                self.language = crate::utils::resolve_language(&self.language).to_string();
            }
        }
    }

    /// 计算 resolved_language 字段
    ///
    /// 将 language 通过 resolve_language 统一解析为具体语言。
    /// 这是整个项目中 "auto" → 具体语言 的唯一解析入口。
    pub fn compute_resolved_language(&mut self) {
        self.resolved_language = crate::utils::resolve_language(&self.language).to_string();
    }

    /// 归一化 mkt 设置
    ///
    /// 如果 mkt 为空或不在 SUPPORTED_MKTS 中，回退到 resolved_language。
    /// 如果 resolved_language 也无效，最终回退到 "en-US"。
    ///
    /// 应在 compute_resolved_language() 之后调用，确保 resolved_language 已填充。
    pub fn normalize_mkt(&mut self) {
        self.mkt = crate::utils::resolve_mkt(&self.mkt, &self.resolved_language).to_string();
    }
}

/// Market 状态统一结构
///
/// 将分散的 mkt 相关状态收敛为一个语义清晰的结构体，
/// 作为前端获取 mkt 状态的唯一接口。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStatus {
    /// 用户设置的 mkt
    pub requested_mkt: String,
    /// 实际生效的 mkt（可能被 Bing 重定向）
    pub effective_mkt: String,
    /// 是否存在 mismatch
    pub is_mismatch: bool,
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
    /// Bing API 最近一次返回的实际 mkt（持久化，解决重启后读不到壁纸的问题）
    ///
    /// 当用户设置的 mkt（如 "en-US"）被 Bing 重定向到其他市场（如 "zh-CN"）时，
    /// 壁纸元数据保存在实际 mkt 下。此字段持久化后，重启时能立即用正确的 key 读取。
    #[serde(default)]
    pub last_actual_mkt: Option<String>,
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
            language: "zh-CN".to_string(),
            resolved_language: "zh-CN".to_string(),
            mkt: "zh-CN".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.auto_update, settings.auto_update);
        assert_eq!(deserialized.save_directory, settings.save_directory);
        assert_eq!(deserialized.launch_at_startup, settings.launch_at_startup);
        assert_eq!(deserialized.theme, settings.theme);
        assert_eq!(deserialized.language, "zh-CN");
        assert_eq!(deserialized.resolved_language, "zh-CN");
        assert_eq!(deserialized.mkt, "zh-CN");
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
        };

        let wallpaper = LocalWallpaper::from(entry.clone());

        assert_eq!(wallpaper.title, entry.title);
        assert_eq!(wallpaper.copyright, entry.copyright);
        assert_eq!(wallpaper.copyright_link, entry.copyrightlink);
        assert_eq!(wallpaper.end_date, entry.enddate);
    }

    #[test]
    fn test_local_wallpaper_serialization() {
        let wallpaper = LocalWallpaper {
            title: "Test Title".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
            urlbase: "/th?id=OHR.Test_EN-US1234567890".to_string(),
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
            "language": "zh-CN"
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert!(settings.auto_update);
        assert_eq!(settings.theme, "system");
        assert_eq!(settings.language, "zh-CN");
        // 旧 JSON 不含 resolved_language 和 mkt，应默认为空字符串
        assert_eq!(settings.resolved_language, "");
        assert_eq!(settings.mkt, "");
    }

    #[test]
    fn test_app_settings_normalize_language() {
        let base = AppSettings {
            auto_update: true,
            save_directory: None,
            launch_at_startup: false,
            theme: "system".to_string(),
            language: "auto".to_string(),
            resolved_language: String::new(),
            mkt: String::new(),
        };

        // "auto" 是有效值，normalize 不应改变
        let mut settings_auto = base.clone();
        settings_auto.normalize_language();
        assert_eq!(settings_auto.language, "auto");

        // "zh-CN" 是有效值，不应改变
        let mut settings_zh = AppSettings {
            language: "zh-CN".to_string(),
            ..base.clone()
        };
        settings_zh.normalize_language();
        assert_eq!(settings_zh.language, "zh-CN");

        // "en-US" 是有效值，不应改变
        let mut settings_en = AppSettings {
            language: "en-US".to_string(),
            ..base.clone()
        };
        settings_en.normalize_language();
        assert_eq!(settings_en.language, "en-US");

        // 其他无效值应被归一化为系统检测的语言
        let mut settings_invalid = AppSettings {
            language: "fr-FR".to_string(),
            ..base.clone()
        };
        settings_invalid.normalize_language();
        assert!(settings_invalid.language == "zh-CN" || settings_invalid.language == "en-US");
    }

    #[test]
    fn test_app_settings_default_language_is_auto() {
        let settings = AppSettings::default();
        // 默认语言偏好应为 "auto"
        assert_eq!(
            settings.language, "auto",
            "Default language should be 'auto'"
        );
        // resolved_language 应为系统检测的具体语言
        assert!(
            settings.resolved_language == "zh-CN" || settings.resolved_language == "en-US",
            "Default resolved_language should be zh-CN or en-US, got: {}",
            settings.resolved_language
        );
    }

    #[test]
    fn test_app_settings_compute_resolved_language() {
        let mut settings = AppSettings {
            auto_update: true,
            save_directory: None,
            launch_at_startup: false,
            theme: "system".to_string(),
            language: "auto".to_string(),
            resolved_language: String::new(),
            mkt: String::new(),
        };

        // "auto" 应解析为系统语言
        settings.compute_resolved_language();
        assert!(
            settings.resolved_language == "zh-CN" || settings.resolved_language == "en-US",
            "auto should resolve to zh-CN or en-US, got: {}",
            settings.resolved_language
        );

        // "zh-CN" 应解析为 "zh-CN"
        settings.language = "zh-CN".to_string();
        settings.compute_resolved_language();
        assert_eq!(settings.resolved_language, "zh-CN");

        // "en-US" 应解析为 "en-US"
        settings.language = "en-US".to_string();
        settings.compute_resolved_language();
        assert_eq!(settings.resolved_language, "en-US");
    }

    #[test]
    fn test_app_settings_normalize_mkt() {
        let mut settings = AppSettings {
            auto_update: true,
            save_directory: None,
            launch_at_startup: false,
            theme: "system".to_string(),
            language: "auto".to_string(),
            resolved_language: "zh-CN".to_string(),
            mkt: String::new(),
        };

        // 空 mkt 应回退到 resolved_language
        settings.normalize_mkt();
        assert_eq!(settings.mkt, "zh-CN");

        // 有效 mkt 不应改变
        settings.mkt = "ja-JP".to_string();
        settings.normalize_mkt();
        assert_eq!(settings.mkt, "ja-JP");

        // 无效 mkt 应回退到 resolved_language
        settings.mkt = "xx-YY".to_string();
        settings.normalize_mkt();
        assert_eq!(settings.mkt, "zh-CN");

        // resolved_language 为 en-US 时的回退
        settings.resolved_language = "en-US".to_string();
        settings.mkt = "".to_string();
        settings.normalize_mkt();
        assert_eq!(settings.mkt, "en-US");
    }

    #[test]
    fn test_app_settings_default_mkt() {
        let settings = AppSettings::default();
        // 默认 mkt 应跟随 resolved_language
        assert!(
            crate::utils::is_valid_mkt(&settings.mkt),
            "Default mkt should be a valid market code, got: {}",
            settings.mkt
        );
    }

    #[test]
    fn test_app_settings_mkt_serde_missing() {
        // 旧版本 JSON 不含 mkt 字段，反序列化后 mkt 应为空字符串
        let json = r#"{
            "auto_update": true,
            "save_directory": null,
            "launch_at_startup": false,
            "theme": "system",
            "language": "zh-CN"
        }"#;

        let settings: AppSettings = serde_json::from_str(json).unwrap();
        assert_eq!(
            settings.mkt, "",
            "Missing mkt should default to empty string"
        );
    }
}
