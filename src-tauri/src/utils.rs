//! 工具函数模块
//! 
//! 提供通用的工具函数，避免代码重复

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

/// 根据语言设置获取 Bing API 市场代码
/// 
/// # Arguments
/// * `language` - 语言设置，可以是 "zh-CN"、"en-US" 或 "auto"
/// 
/// # Returns
/// Bing API 使用的市场代码，"zh-CN" 或 "en-US"
pub fn get_bing_market_code(language: &str) -> &'static str {
    match language {
        "zh-CN" => "zh-CN",
        "en-US" => "en-US",
        _ => {
            // 自动模式：使用系统语言检测
            detect_system_language()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_bing_market_code() {
        assert_eq!(get_bing_market_code("zh-CN"), "zh-CN");
        assert_eq!(get_bing_market_code("en-US"), "en-US");
        // 自动模式会调用 detect_system_language，结果取决于系统环境
        let auto_result = get_bing_market_code("auto");
        assert!(auto_result == "zh-CN" || auto_result == "en-US");
    }
}

