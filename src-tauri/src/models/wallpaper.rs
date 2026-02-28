use serde::{Deserialize, Serialize};

use super::bing::BingImageEntry;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
