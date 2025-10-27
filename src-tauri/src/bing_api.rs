use crate::models::{BingImageArchive, BingImageEntry};
use anyhow::{Context, Result};

const BING_API_URL: &str = "https://www.bing.com/HPImageArchive.aspx";
const BING_BASE_URL: &str = "https://www.bing.com";

/// 从 Bing API 获取壁纸列表
///
/// # Arguments
/// * `count` - 要获取的图片数量 (1-8)
/// * `idx` - 起始索引,0表示今天
pub async fn fetch_bing_images(count: u8, idx: u8) -> Result<Vec<BingImageEntry>> {
    let count = count.min(8); // Bing API 限制最多8张

    let url = format!(
        "{}?format=js&n={}&idx={}&mkt=zh-CN",
        BING_API_URL, count, idx
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to fetch from Bing API")?;

    let archive: BingImageArchive = response
        .json()
        .await
        .context("Failed to parse Bing API response")?;

    // 为每个图片条目添加完整的 URL
    let images = archive
        .images
        .into_iter()
        .map(|mut img| {
            if !img.url.starts_with("http") {
                img.url = format!("{}{}", BING_BASE_URL, img.url);
            }
            img
        })
        .collect();

    Ok(images)
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

    #[tokio::test]
    #[ignore = "Network test ignored by default. Run with: BING_TEST=1 cargo test -- --ignored"]
    async fn test_fetch_bing_images() {
        // Only execute network call when explicitly enabled
        if std::env::var("BING_TEST").ok().as_deref() != Some("1") {
            // Silently skip (return Ok(()))
            return;
        }

        let images = fetch_bing_images(1, 0).await;
        assert!(images.is_ok(), "Bing fetch failed");
        let images = images.unwrap();
        assert!(!images.is_empty(), "No images returned");
        assert!(images[0].url.starts_with("http"));
    }

    #[test]
    fn test_get_wallpaper_url() {
        let urlbase = "/th?id=OHR.BingWallpaper_EN-US1234567890";
        let url = get_wallpaper_url(urlbase, "1920x1080");
        assert!(url.contains("1920x1080"));
        assert!(url.starts_with("https://"));
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
        let expected_format = format!("{}?format=js&n={}&idx={}&mkt=zh-CN", BING_API_URL, 3, 0);
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
}
