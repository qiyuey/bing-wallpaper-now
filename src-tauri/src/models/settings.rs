use serde::{Deserialize, Serialize};

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
