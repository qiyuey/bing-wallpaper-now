//! 工具函数模块
//!
//! 提供通用的工具函数，避免代码重复

// ─── 语言相关 ───

/// 检测系统语言
///
/// 通过检查环境变量 LANG、LC_ALL、LC_MESSAGES 来检测系统语言
/// 返回 "zh-CN" 或 "en-US"
pub fn detect_system_language() -> &'static str {
    let system_lang = std::env::var("LANG")
        .or_else(|_| std::env::var("LC_ALL"))
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .unwrap_or_else(|_| String::new());

    if system_lang.contains("zh") || system_lang.contains("CN") {
        "zh-CN"
    } else {
        "en-US"
    }
}

/// 解析语言设置，确保返回有效的语言代码
///
/// 只有 "zh-CN" 和 "en-US" 是有效值，其他任何值（包括旧版本的 "auto"）
/// 都视为无配置，通过系统语言检测来决定使用中文还是英文。
///
/// # Arguments
/// * `language` - 存储的语言设置值
///
/// # Returns
/// 有效的语言代码："zh-CN" 或 "en-US"
pub fn resolve_language(language: &str) -> &'static str {
    match language {
        "zh-CN" => "zh-CN",
        "en-US" => "en-US",
        _ => {
            // 非中文/英文的配置一律走无配置流程
            detect_system_language()
        }
    }
}

// ─── mkt（市场代码）相关 ───

/// Bing API 支持的市场代码列表
///
/// 基于 Bing Image Search API 官方文档：
/// https://learn.microsoft.com/en-us/bing/search-apis/bing-image-search/reference/market-codes
pub const SUPPORTED_MKTS: &[&str] = &[
    // 亚太
    "zh-CN", "zh-TW", "zh-HK", "ja-JP", "ko-KR", "en-AU", "en-NZ", "en-IN", "en-ID", "en-MY",
    "en-PH", // 欧洲
    "en-GB", "de-DE", "de-AT", "de-CH", "fr-FR", "fr-BE", "fr-CH", "it-IT", "es-ES", "nl-NL",
    "nl-BE", "pl-PL", "ru-RU", "sv-SE", "da-DK", "fi-FI", "no-NO", "tr-TR", // 美洲
    "en-US", "en-CA", "fr-CA", "pt-BR", "es-MX", "es-AR", "es-CL", "es-US", // 非洲
    "en-ZA",
];

/// 检查 mkt 是否是 Bing API 支持的有效市场代码
pub fn is_valid_mkt(mkt: &str) -> bool {
    SUPPORTED_MKTS.contains(&mkt)
}

/// 市场分组（用于前端下拉列表渲染）
#[derive(Debug, Clone, serde::Serialize)]
pub struct MarketGroup {
    /// 区域 ID（用于 i18n，如 "asia_pacific"）
    pub region: &'static str,
    /// 该区域下的市场列表
    pub markets: Vec<MarketOption>,
}

/// 单个市场选项
#[derive(Debug, Clone, serde::Serialize)]
pub struct MarketOption {
    /// 市场代码（如 "zh-CN"）
    pub code: &'static str,
    /// 显示名称（使用本地语言，如 "中国大陆"）
    pub label: &'static str,
}

/// 获取按区域分组的市场列表（单一数据源，前后端共享）
pub fn get_market_groups() -> Vec<MarketGroup> {
    vec![
        MarketGroup {
            region: "asia_pacific",
            markets: vec![
                MarketOption {
                    code: "zh-CN",
                    label: "中国大陆",
                },
                MarketOption {
                    code: "zh-TW",
                    label: "台灣",
                },
                MarketOption {
                    code: "zh-HK",
                    label: "香港",
                },
                MarketOption {
                    code: "ja-JP",
                    label: "日本",
                },
                MarketOption {
                    code: "ko-KR",
                    label: "한국",
                },
                MarketOption {
                    code: "en-AU",
                    label: "Australia",
                },
                MarketOption {
                    code: "en-NZ",
                    label: "New Zealand",
                },
                MarketOption {
                    code: "en-IN",
                    label: "India",
                },
                MarketOption {
                    code: "en-ID",
                    label: "Indonesia",
                },
                MarketOption {
                    code: "en-MY",
                    label: "Malaysia",
                },
                MarketOption {
                    code: "en-PH",
                    label: "Philippines",
                },
            ],
        },
        MarketGroup {
            region: "europe",
            markets: vec![
                MarketOption {
                    code: "en-GB",
                    label: "United Kingdom",
                },
                MarketOption {
                    code: "de-DE",
                    label: "Deutschland",
                },
                MarketOption {
                    code: "de-AT",
                    label: "Österreich",
                },
                MarketOption {
                    code: "de-CH",
                    label: "Schweiz",
                },
                MarketOption {
                    code: "fr-FR",
                    label: "France",
                },
                MarketOption {
                    code: "fr-BE",
                    label: "Belgique - FR",
                },
                MarketOption {
                    code: "fr-CH",
                    label: "Suisse - FR",
                },
                MarketOption {
                    code: "it-IT",
                    label: "Italia",
                },
                MarketOption {
                    code: "es-ES",
                    label: "España",
                },
                MarketOption {
                    code: "nl-NL",
                    label: "Nederland",
                },
                MarketOption {
                    code: "nl-BE",
                    label: "België - NL",
                },
                MarketOption {
                    code: "pl-PL",
                    label: "Polska",
                },
                MarketOption {
                    code: "ru-RU",
                    label: "Россия",
                },
                MarketOption {
                    code: "sv-SE",
                    label: "Sverige",
                },
                MarketOption {
                    code: "da-DK",
                    label: "Danmark",
                },
                MarketOption {
                    code: "fi-FI",
                    label: "Suomi",
                },
                MarketOption {
                    code: "no-NO",
                    label: "Norge",
                },
                MarketOption {
                    code: "tr-TR",
                    label: "Türkiye",
                },
            ],
        },
        MarketGroup {
            region: "americas",
            markets: vec![
                MarketOption {
                    code: "en-US",
                    label: "United States",
                },
                MarketOption {
                    code: "en-CA",
                    label: "Canada - EN",
                },
                MarketOption {
                    code: "fr-CA",
                    label: "Canada - FR",
                },
                MarketOption {
                    code: "pt-BR",
                    label: "Brasil",
                },
                MarketOption {
                    code: "es-MX",
                    label: "México",
                },
                MarketOption {
                    code: "es-AR",
                    label: "Argentina",
                },
                MarketOption {
                    code: "es-CL",
                    label: "Chile",
                },
                MarketOption {
                    code: "es-US",
                    label: "Estados Unidos - ES",
                },
            ],
        },
        MarketGroup {
            region: "africa",
            markets: vec![MarketOption {
                code: "en-ZA",
                label: "South Africa",
            }],
        },
    ]
}

/// 标准化 mkt 大小写
///
/// Bing API 返回的 mkt 可能是小写（如 copyrightlink 中的 "zh-cn"），
/// 需要标准化为 "xx-YY" 格式（语言小写-国家大写）。
///
/// 例如：`"zh-cn"` -> `"zh-CN"`，`"EN-US"` -> `"en-US"`
pub fn normalize_mkt_case(mkt: &str) -> String {
    if let Some((lang, country)) = mkt.split_once('-') {
        format!("{}-{}", lang.to_lowercase(), country.to_uppercase())
    } else {
        mkt.to_string()
    }
}

/// 从 Bing API 响应的 copyrightlink 中检测实际返回的市场代码
///
/// copyrightlink 示例：`https://www.bing.com/search?q=...&form=hpcapt&mkt=zh-cn`
/// 其中 mkt 参数的值就是 Bing 实际使用的市场代码（通常是小写）。
///
/// # Arguments
/// * `copyrightlink` - Bing API 响应中的 copyrightlink URL
///
/// # Returns
/// 标准化后的 mkt（如 "zh-CN"），如果无法解析则返回 None
pub fn detect_actual_mkt(copyrightlink: &str) -> Option<String> {
    // 从 URL 中提取 mkt 参数值
    // copyrightlink 格式: https://www.bing.com/search?q=...&mkt=zh-cn
    let mkt_value = copyrightlink
        .split('?')
        .nth(1)? // 取 query string
        .split('&')
        .find(|param| {
            let lower = param.to_lowercase();
            lower.starts_with("mkt=")
        })?
        .split('=')
        .nth(1)?; // 取 mkt=xxx 的值

    if mkt_value.is_empty() {
        return None;
    }

    // 标准化大小写：zh-cn -> zh-CN
    let normalized = normalize_mkt_case(mkt_value);

    // 验证是否在支持的市场列表中
    if is_valid_mkt(&normalized) {
        Some(normalized)
    } else {
        // 即使不在列表中，也返回标准化后的值（可能是新增的市场）
        log::warn!("检测到未知的 mkt: {} (原始: {})", normalized, mkt_value);
        Some(normalized)
    }
}

/// 判断 API 返回的日期是否超前于本地日期（需要减一天）
///
/// 通过比较 Bing API 返回的第一张图片的 enddate 与本地日期来判断：
/// - 如果 enddate > 本地日期，说明存在时区差异（如美洲市场），需要减一天
/// - 如果 enddate <= 本地日期，日期已对齐（如东亚、欧洲市场），无需调整
///
/// 这种基于实际数据的判断比基于市场列表的硬编码更准确，
/// 能正确处理所有市场和时区组合，包括夏令时边界情况。
///
/// # Arguments
/// * `enddate` - Bing API 返回的 enddate 字符串（YYYYMMDD 格式）
///
/// # Returns
/// `true` 表示需要减一天
pub fn is_date_ahead_of_local(enddate: &str) -> bool {
    let today = chrono::Local::now().format("%Y%m%d").to_string();
    enddate > today.as_str()
}

/// 解析 mkt 设置，确保返回有效的市场代码
///
/// 验证 mkt 是否在 SUPPORTED_MKTS 中，无效时使用 fallback_language 回退。
///
/// # Arguments
/// * `mkt` - 用户设置的市场代码
/// * `fallback_language` - 回退语言（通常是 resolved_language）
///
/// # Returns
/// 有效的市场代码
pub fn resolve_mkt<'a>(mkt: &'a str, fallback_language: &str) -> &'a str {
    if is_valid_mkt(mkt) {
        return mkt;
    }
    // mkt 无效时，从 SUPPORTED_MKTS 中查找 fallback_language 对应的 &'static str 返回
    if is_valid_mkt(fallback_language) {
        for &supported in SUPPORTED_MKTS {
            if supported == fallback_language {
                return supported;
            }
        }
    }
    "en-US"
}

// ─── mkt 运行时辅助 ───

/// 获取有效的 mkt（用于读取壁纸索引）
///
/// 优先使用 `last_actual_mkt`（最近一次 Bing API 返回的实际 mkt），
/// 否则回退到用户设置的 `settings_mkt`。
///
/// 这确保了写入和读取使用同一个 mkt key。
pub fn effective_mkt(last_actual_mkt: Option<&str>, settings_mkt: &str) -> String {
    last_actual_mkt
        .map(|s| s.to_string())
        .unwrap_or_else(|| settings_mkt.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── 语言测试 ───

    #[test]
    fn test_resolve_language_valid() {
        assert_eq!(resolve_language("zh-CN"), "zh-CN");
        assert_eq!(resolve_language("en-US"), "en-US");
    }

    #[test]
    fn test_resolve_language_invalid_uses_system_detection() {
        // 无效语言值（包括旧版本的 "auto"）应通过系统语言检测
        let result = resolve_language("auto");
        assert!(result == "zh-CN" || result == "en-US");

        let result = resolve_language("fr-FR");
        assert!(result == "zh-CN" || result == "en-US");

        let result = resolve_language("");
        assert!(result == "zh-CN" || result == "en-US");
    }

    // ─── mkt 测试 ───

    #[test]
    fn test_is_valid_mkt() {
        assert!(is_valid_mkt("zh-CN"));
        assert!(is_valid_mkt("en-US"));
        assert!(is_valid_mkt("ja-JP"));
        assert!(is_valid_mkt("de-DE"));
        assert!(!is_valid_mkt(""));
        assert!(!is_valid_mkt("xx-YY"));
        assert!(!is_valid_mkt("zh-cn")); // 大小写敏感
    }

    #[test]
    fn test_normalize_mkt_case() {
        assert_eq!(normalize_mkt_case("zh-cn"), "zh-CN");
        assert_eq!(normalize_mkt_case("EN-US"), "en-US");
        assert_eq!(normalize_mkt_case("ja-JP"), "ja-JP");
        assert_eq!(normalize_mkt_case("de-de"), "de-DE");
        assert_eq!(normalize_mkt_case("invalid"), "invalid"); // 无分隔符
    }

    #[test]
    fn test_resolve_mkt_valid() {
        assert_eq!(resolve_mkt("zh-CN", "en-US"), "zh-CN");
        assert_eq!(resolve_mkt("ja-JP", "en-US"), "ja-JP");
        assert_eq!(resolve_mkt("en-GB", "zh-CN"), "en-GB");
    }

    #[test]
    fn test_resolve_mkt_invalid_uses_fallback() {
        assert_eq!(resolve_mkt("", "zh-CN"), "zh-CN");
        assert_eq!(resolve_mkt("xx-YY", "en-US"), "en-US");
        assert_eq!(resolve_mkt("invalid", "ja-JP"), "ja-JP");
    }

    #[test]
    fn test_resolve_mkt_invalid_fallback_uses_en_us() {
        assert_eq!(resolve_mkt("", ""), "en-US");
        assert_eq!(resolve_mkt("invalid", "also-invalid"), "en-US");
    }

    // ─── detect_actual_mkt 测试 ───

    #[test]
    fn test_detect_actual_mkt_from_copyrightlink() {
        // 典型的中文 copyrightlink
        let link =
            "https://www.bing.com/search?q=%E6%83%85%E4%BA%BA%E8%8A%82&form=hpcapt&mkt=zh-cn";
        assert_eq!(detect_actual_mkt(link), Some("zh-CN".to_string()));

        // 英文 copyrightlink
        let link = "https://www.bing.com/search?q=test&form=hpcapt&mkt=en-us";
        assert_eq!(detect_actual_mkt(link), Some("en-US".to_string()));

        // 日文 copyrightlink
        let link = "https://www.bing.com/search?q=test&form=hpcapt&mkt=ja-jp";
        assert_eq!(detect_actual_mkt(link), Some("ja-JP".to_string()));
    }

    #[test]
    fn test_detect_actual_mkt_missing() {
        // 没有 mkt 参数
        let link = "https://www.bing.com/search?q=test&form=hpcapt";
        assert_eq!(detect_actual_mkt(link), None);

        // 空字符串
        assert_eq!(detect_actual_mkt(""), None);

        // 无 query string
        let link = "https://www.bing.com/search";
        assert_eq!(detect_actual_mkt(link), None);
    }

    #[test]
    fn test_detect_actual_mkt_empty_value() {
        let link = "https://www.bing.com/search?mkt=&form=hpcapt";
        assert_eq!(detect_actual_mkt(link), None);
    }

    // ─── is_date_ahead_of_local 测试 ───

    #[test]
    fn test_is_date_ahead_of_local() {
        // 明显过去的日期不需要调整
        assert!(!is_date_ahead_of_local("20200101"));

        // 明显未来的日期需要调整
        assert!(is_date_ahead_of_local("29991231"));

        // 当天日期不需要调整（enddate == today 时不超前）
        let today = chrono::Local::now().format("%Y%m%d").to_string();
        assert!(!is_date_ahead_of_local(&today));
    }

    #[test]
    fn test_supported_mkts_completeness() {
        // 确保列表不为空且包含核心市场
        assert!(!SUPPORTED_MKTS.is_empty());
        assert!(SUPPORTED_MKTS.contains(&"zh-CN"));
        assert!(SUPPORTED_MKTS.contains(&"en-US"));
        assert!(SUPPORTED_MKTS.contains(&"ja-JP"));
        // 确保没有重复
        let mut sorted = SUPPORTED_MKTS.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            SUPPORTED_MKTS.len(),
            "SUPPORTED_MKTS contains duplicates"
        );
    }

    // ─── detect_system_language 测试 ───

    #[test]
    fn test_detect_system_language_returns_valid() {
        // 不管系统语言如何，结果应是 zh-CN 或 en-US
        let result = detect_system_language();
        assert!(
            result == "zh-CN" || result == "en-US",
            "detect_system_language should return zh-CN or en-US, got: {}",
            result
        );
    }

    // ─── get_market_groups 测试 ───

    #[test]
    fn test_get_market_groups_structure() {
        let groups = get_market_groups();

        // 应有 4 个区域分组
        assert_eq!(groups.len(), 4);

        // 验证区域 ID
        let regions: Vec<&str> = groups.iter().map(|g| g.region).collect();
        assert!(regions.contains(&"asia_pacific"));
        assert!(regions.contains(&"europe"));
        assert!(regions.contains(&"americas"));
        assert!(regions.contains(&"africa"));
    }

    #[test]
    fn test_get_market_groups_contains_all_supported_mkts() {
        let groups = get_market_groups();

        // 收集所有 market group 中的 code
        let group_codes: Vec<&str> = groups
            .iter()
            .flat_map(|g| g.markets.iter().map(|m| m.code))
            .collect();

        // 每个 SUPPORTED_MKTS 中的 mkt 都应在 market groups 中
        for &mkt in SUPPORTED_MKTS {
            assert!(
                group_codes.contains(&mkt),
                "SUPPORTED_MKTS contains {} but it's not in any market group",
                mkt
            );
        }

        // market groups 中的每个 code 都应在 SUPPORTED_MKTS 中
        for code in &group_codes {
            assert!(
                SUPPORTED_MKTS.contains(code),
                "Market group contains {} but it's not in SUPPORTED_MKTS",
                code
            );
        }
    }

    #[test]
    fn test_get_market_groups_no_duplicate_codes() {
        let groups = get_market_groups();

        let codes: Vec<&str> = groups
            .iter()
            .flat_map(|g| g.markets.iter().map(|m| m.code))
            .collect();

        let mut sorted = codes.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            codes.len(),
            "Market groups contain duplicate codes"
        );
    }

    #[test]
    fn test_get_market_groups_non_empty_labels() {
        let groups = get_market_groups();

        for group in &groups {
            assert!(!group.region.is_empty(), "Group region should not be empty");
            assert!(
                !group.markets.is_empty(),
                "Group {} should have at least one market",
                group.region
            );
            for market in &group.markets {
                assert!(
                    !market.label.is_empty(),
                    "Market {} should have a non-empty label",
                    market.code
                );
                assert!(
                    !market.code.is_empty(),
                    "Market should have a non-empty code"
                );
            }
        }
    }

    #[test]
    fn test_get_market_groups_code_format() {
        let groups = get_market_groups();

        for group in &groups {
            for market in &group.markets {
                // 每个 code 应符合 xx-YY 格式
                assert!(
                    market.code.contains('-'),
                    "Market code {} should contain '-'",
                    market.code
                );
                let parts: Vec<&str> = market.code.split('-').collect();
                assert_eq!(
                    parts.len(),
                    2,
                    "Market code {} should have exactly 2 parts",
                    market.code
                );
                // 语言部分应为小写
                assert_eq!(
                    parts[0],
                    parts[0].to_lowercase(),
                    "Language part of {} should be lowercase",
                    market.code
                );
                // 国家部分应为大写
                assert_eq!(
                    parts[1],
                    parts[1].to_uppercase(),
                    "Country part of {} should be uppercase",
                    market.code
                );
            }
        }
    }

    // ─── effective_mkt 测试 ───

    #[test]
    fn test_effective_mkt_prefers_actual() {
        assert_eq!(effective_mkt(Some("zh-CN"), "en-US"), "zh-CN");
    }

    #[test]
    fn test_effective_mkt_falls_back_to_settings() {
        assert_eq!(effective_mkt(None, "ja-JP"), "ja-JP");
    }
}
