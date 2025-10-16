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
}
