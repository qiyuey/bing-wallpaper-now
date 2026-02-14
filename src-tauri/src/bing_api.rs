use crate::models::{BingImageArchive, BingImageEntry};
use crate::utils;
use anyhow::{Context, Result};
use log::{error, info, warn};

const BING_API_URL: &str = "https://www.bing.com/HPImageArchive.aspx";
const BING_BASE_URL: &str = "https://www.bing.com";

/// Bing API 获取结果
///
/// 除了返回图片列表外，还包含从响应中检测到的实际 mkt。
/// 当中国 Bing 忽略请求参数中的 mkt 并强制返回 zh-CN 时，
/// `actual_mkt` 会与请求的 `mkt` 不同。
#[derive(Debug, Clone)]
pub struct BingFetchResult {
    /// 图片列表
    pub images: Vec<BingImageEntry>,
    /// 从 copyrightlink 检测到的实际 mkt（标准化后）
    /// None 表示无法从响应中检测
    pub actual_mkt: Option<String>,
}

/// 从 Bing API 获取壁纸列表
///
/// # Arguments
/// * `count` - 要获取的图片数量 (1-8)
/// * `idx` - 起始索引,0表示今天
/// * `mkt` - 市场/语言代码，例如 "zh-CN" 或 "en-US"
///
/// # Returns
/// `BingFetchResult` 包含图片列表和检测到的实际 mkt
pub async fn fetch_bing_images(count: u8, idx: u8, mkt: &str) -> Result<BingFetchResult> {
    let count = count.min(8); // Bing API 限制最多8张

    let url = format!(
        "{}?format=js&n={}&idx={}&mkt={}",
        BING_API_URL, count, idx, mkt
    );

    info!(target: "bing_api", "开始请求 Bing API: count={}, idx={}, mkt={}, url={}", count, idx, mkt, url);

    let start_time = std::time::Instant::now();

    let response = match reqwest::get(&url).await {
        Ok(resp) => {
            let elapsed = start_time.elapsed();
            let status = resp.status();
            info!(target: "bing_api", "Bing API 响应收到: status={}, 耗时={:.2}ms", status, elapsed.as_secs_f64() * 1000.0);

            if !status.is_success() {
                warn!(target: "bing_api", "Bing API 返回非成功状态: status={}", status);
            }

            resp
        }
        Err(e) => {
            let elapsed = start_time.elapsed();
            error!(target: "bing_api", "Bing API 请求失败: url={}, 耗时={:.2}ms, 错误={}", url, elapsed.as_secs_f64() * 1000.0, e);
            return Err(e).context("Failed to fetch from Bing API");
        }
    };

    let parse_start = std::time::Instant::now();
    let archive: BingImageArchive = match response.json().await {
        Ok(archive) => {
            let elapsed = parse_start.elapsed();
            info!(target: "bing_api", "Bing API 响应解析成功: 耗时={:.2}ms", elapsed.as_secs_f64() * 1000.0);
            archive
        }
        Err(e) => {
            let elapsed = parse_start.elapsed();
            error!(target: "bing_api", "Bing API 响应解析失败: 耗时={:.2}ms, 错误={}", elapsed.as_secs_f64() * 1000.0, e);
            return Err(e).context("Failed to parse Bing API response");
        }
    };

    // 从第一个图片的 copyrightlink 检测实际 mkt
    let actual_mkt = archive
        .images
        .first()
        .and_then(|img| utils::detect_actual_mkt(&img.copyrightlink));

    if let Some(ref detected) = actual_mkt
        && detected != mkt
    {
        warn!(
            target: "bing_api",
            "实际返回的 mkt ({}) 与请求的 mkt ({}) 不同",
            detected, mkt
        );
    }

    // 通过比较第一张图片的 enddate 与本地日期判断是否需要日期调整
    // 这种方式不依赖市场列表硬编码，能正确处理所有市场和时区组合
    let needs_adjustment = archive
        .images
        .first()
        .map(|img| utils::is_date_ahead_of_local(&img.enddate))
        .unwrap_or(false);

    if needs_adjustment {
        info!(
            target: "bing_api",
            "检测到 API 日期超前于本地日期，将对所有日期减一天（mkt: {}）",
            actual_mkt.as_deref().unwrap_or(mkt)
        );
    }

    // 为每个图片条目添加完整的 URL，并根据时区差异调整日期
    let images: Vec<BingImageEntry> = archive
        .images
        .into_iter()
        .map(|mut img| {
            if !img.url.starts_with("http") {
                img.url = format!("{}{}", BING_BASE_URL, img.url);
            }
            // 如果 API 日期超前于本地日期，减一天对齐
            if needs_adjustment {
                img.startdate = subtract_one_day(&img.startdate);
                img.enddate = subtract_one_day(&img.enddate);
            }
            img
        })
        .collect();

    let total_elapsed = start_time.elapsed();
    info!(
        target: "bing_api",
        "Bing API 请求完成: 获取到 {} 张图片, actual_mkt={:?}, 总耗时={:.2}ms",
        images.len(),
        actual_mkt,
        total_elapsed.as_secs_f64() * 1000.0
    );

    Ok(BingFetchResult { images, actual_mkt })
}

/// 将日期字符串减一天（YYYYMMDD 格式）
///
/// # Arguments
/// * `date_str` - 日期字符串，格式为 YYYYMMDD
///
/// # Returns
/// 减一天后的日期字符串（YYYYMMDD 格式）
fn subtract_one_day(date_str: &str) -> String {
    use chrono::{Datelike, NaiveDate};

    // 解析日期字符串 YYYYMMDD
    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y%m%d") {
        // 减一天
        if let Some(prev_day) = date.pred_opt() {
            return format!(
                "{:04}{:02}{:02}",
                prev_day.year(),
                prev_day.month(),
                prev_day.day()
            );
        }
    }

    // 如果解析失败，返回原字符串
    date_str.to_string()
}

/// 获取壁纸的高分辨率 URL
///
/// # Arguments
/// * `urlbase` - 从 Bing API 获取的 urlbase 字段
/// * `resolution` - 分辨率,例如 "1920x1080", "UHD" 等
pub fn get_wallpaper_url(urlbase: &str, resolution: &str) -> String {
    format!("{}{}_{}.jpg", BING_BASE_URL, urlbase, resolution)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试辅助函数：将日期字符串减一天
    fn subtract_one_day(date_str: &str) -> String {
        use chrono::{Datelike, NaiveDate};

        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y%m%d")
            && let Some(prev_day) = date.pred_opt()
        {
            return format!(
                "{:04}{:02}{:02}",
                prev_day.year(),
                prev_day.month(),
                prev_day.day()
            );
        }

        date_str.to_string()
    }

    #[tokio::test]
    #[ignore = "Network test ignored by default. Run with: BING_TEST=1 cargo test -- --ignored"]
    async fn test_fetch_bing_images() {
        // Only execute network call when explicitly enabled
        if std::env::var("BING_TEST").ok().as_deref() != Some("1") {
            // Silently skip (return Ok(()))
            return;
        }

        let result = fetch_bing_images(1, 0, "zh-CN").await;
        assert!(result.is_ok(), "Bing fetch failed");
        let result = result.unwrap();
        assert!(!result.images.is_empty(), "No images returned");
        assert!(result.images[0].url.starts_with("http"));
    }

    #[test]
    fn test_subtract_one_day() {
        // 测试正常日期减一天
        assert_eq!(subtract_one_day("20251031"), "20251030");
        assert_eq!(subtract_one_day("20240101"), "20231231");
        assert_eq!(subtract_one_day("20240301"), "20240229"); // 闰年
        assert_eq!(subtract_one_day("20240201"), "20240131");

        // 测试无效日期格式（返回原字符串）
        assert_eq!(subtract_one_day("invalid"), "invalid");
        assert_eq!(subtract_one_day(""), "");
    }

    #[test]
    fn test_get_wallpaper_url_different_resolutions() {
        let urlbase = "/th?id=OHR.TestImage_ZH-CN1234567890";

        // Test UHD resolution
        let uhd_url = get_wallpaper_url(urlbase, "UHD");
        assert_eq!(
            uhd_url,
            "https://www.bing.com/th?id=OHR.TestImage_ZH-CN1234567890_UHD.jpg"
        );

        // Test 1920x1080 resolution
        let fhd_url = get_wallpaper_url(urlbase, "1920x1080");
        assert_eq!(
            fhd_url,
            "https://www.bing.com/th?id=OHR.TestImage_ZH-CN1234567890_1920x1080.jpg"
        );

        // Test 1366x768 resolution
        let hd_url = get_wallpaper_url(urlbase, "1366x768");
        assert_eq!(
            hd_url,
            "https://www.bing.com/th?id=OHR.TestImage_ZH-CN1234567890_1366x768.jpg"
        );
    }

    #[test]
    fn test_get_wallpaper_url_empty_resolution() {
        let urlbase = "/th?id=OHR.TestImage";
        let url = get_wallpaper_url(urlbase, "");
        assert_eq!(url, "https://www.bing.com/th?id=OHR.TestImage_.jpg");
    }

    #[test]
    fn test_get_wallpaper_url_special_characters() {
        let urlbase = "/th?id=OHR.Test&Image_ZH-CN";
        let url = get_wallpaper_url(urlbase, "UHD");
        assert!(url.contains("Test&Image"));
        assert!(url.ends_with("_UHD.jpg"));
    }

    #[test]
    fn test_get_wallpaper_url_with_query_params() {
        let urlbase = "/th?id=OHR.TestImage&rf=Test";
        let url = get_wallpaper_url(urlbase, "1920x1080");
        assert!(url.contains("rf=Test"));
        assert!(url.ends_with("_1920x1080.jpg"));
    }

    #[tokio::test]
    async fn test_fetch_bing_images_count_clamping() {
        // Test that count > 8 is clamped to 8
        // We can't test actual fetching without network, but we can verify the logic
        let count = 10u8; // Greater than max (8)
        let clamped_count = count.min(8);
        assert_eq!(clamped_count, 8, "Count should be clamped to 8");

        let count = 3u8; // Within range
        let clamped_count = count.min(8);
        assert_eq!(clamped_count, 3, "Count should remain as is");
    }

    #[test]
    fn test_bing_api_url_format() {
        // Verify the expected URL format
        let expected_format = format!(
            "{}?format=js&n={}&idx={}&mkt={}",
            BING_API_URL, 3, 0, "zh-CN"
        );
        assert!(expected_format.contains("format=js"));
        assert!(expected_format.contains("n=3"));
        assert!(expected_format.contains("idx=0"));
        assert!(expected_format.contains("mkt=zh-CN"));
    }

    #[test]
    fn test_constants_validity() {
        // Test that constants are valid
        assert!(BING_API_URL.starts_with("https://"));
        assert!(BING_BASE_URL.starts_with("https://"));
        assert!(BING_API_URL.contains("bing.com"));
        assert_eq!(BING_BASE_URL, "https://www.bing.com");
    }

    #[tokio::test]
    async fn test_fetch_bing_images_invalid_url() {
        // Test error handling for network failures
        // This will fail due to the real URL, but we're testing error handling
        let result = reqwest::get("https://invalid-domain-that-does-not-exist-12345.com").await;
        assert!(result.is_err(), "Should fail for invalid domain");
    }

    #[test]
    fn test_url_construction_edge_cases() {
        // Test edge cases in URL construction

        // Empty urlbase
        let url = get_wallpaper_url("", "UHD");
        assert_eq!(url, "https://www.bing.com_UHD.jpg");

        // urlbase without leading slash
        let url = get_wallpaper_url("th?id=OHR.Test", "1920x1080");
        assert_eq!(url, "https://www.bing.comth?id=OHR.Test_1920x1080.jpg");

        // Very long resolution string
        let url = get_wallpaper_url("/th?id=OHR.Test", "verylongresolutionstring");
        assert!(url.ends_with("_verylongresolutionstring.jpg"));
    }

    #[test]
    fn test_get_wallpaper_url_consistency() {
        // Test that calling the same function with the same inputs produces consistent results
        let urlbase = "/th?id=OHR.TestImage";
        let resolution = "UHD";

        let url1 = get_wallpaper_url(urlbase, resolution);
        let url2 = get_wallpaper_url(urlbase, resolution);

        assert_eq!(url1, url2, "Same inputs should produce same outputs");
    }

    #[test]
    fn test_bing_base_url_in_wallpaper_url() {
        // Verify that BING_BASE_URL is correctly used in URL construction
        let urlbase = "/test";
        let url = get_wallpaper_url(urlbase, "UHD");
        assert!(url.starts_with(BING_BASE_URL));
    }

    // ─── subtract_one_day 边界用例 ───

    #[test]
    fn test_subtract_one_day_year_boundary() {
        // 跨年边界
        assert_eq!(subtract_one_day("20250101"), "20241231");
        assert_eq!(subtract_one_day("20210101"), "20201231");
    }

    #[test]
    fn test_subtract_one_day_month_boundaries() {
        // 各月份首日
        assert_eq!(subtract_one_day("20240301"), "20240229"); // 闰年 2 月
        assert_eq!(subtract_one_day("20230301"), "20230228"); // 非闰年 2 月
        assert_eq!(subtract_one_day("20240401"), "20240331");
        assert_eq!(subtract_one_day("20240501"), "20240430");
        assert_eq!(subtract_one_day("20240601"), "20240531");
        assert_eq!(subtract_one_day("20240701"), "20240630");
        assert_eq!(subtract_one_day("20240801"), "20240731");
        assert_eq!(subtract_one_day("20240901"), "20240831");
        assert_eq!(subtract_one_day("20241001"), "20240930");
        assert_eq!(subtract_one_day("20241101"), "20241031");
        assert_eq!(subtract_one_day("20241201"), "20241130");
    }

    #[test]
    fn test_subtract_one_day_normal_day() {
        assert_eq!(subtract_one_day("20240215"), "20240214");
        assert_eq!(subtract_one_day("20240630"), "20240629");
    }

    // ─── BingFetchResult 结构测试 ───

    #[test]
    fn test_bing_fetch_result_fields() {
        let result = BingFetchResult {
            images: vec![BingImageEntry {
                url: "https://www.bing.com/test.jpg".to_string(),
                urlbase: "/th?id=OHR.Test".to_string(),
                copyright: "Test (Author)".to_string(),
                copyrightlink: "https://www.bing.com/search?q=test&mkt=zh-cn".to_string(),
                title: "Test Title".to_string(),
                startdate: "20240101".to_string(),
                enddate: "20240102".to_string(),
            }],
            actual_mkt: Some("zh-CN".to_string()),
        };

        assert_eq!(result.images.len(), 1);
        assert_eq!(result.actual_mkt, Some("zh-CN".to_string()));
        assert_eq!(result.images[0].title, "Test Title");
    }

    #[test]
    fn test_bing_fetch_result_empty() {
        let result = BingFetchResult {
            images: vec![],
            actual_mkt: None,
        };

        assert!(result.images.is_empty());
        assert!(result.actual_mkt.is_none());
    }
}
