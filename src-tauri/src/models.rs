use chrono::{DateTime, Utc};
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
        }
    }
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_update: bool,
    pub update_interval_hours: u64,
    pub save_directory: Option<String>,
    pub keep_image_count: u32,
    pub launch_at_startup: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_update: true,
            update_interval_hours: 24,
            save_directory: None,
            keep_image_count: 50,
            launch_at_startup: false,
        }
    }
}
