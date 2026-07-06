use crate::{AppState, index_manager, models::WallpaperIndex, storage};
use chrono::Local;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct WallpaperDataStats {
    count: usize,
    earliest_end_date: Option<String>,
    latest_end_date: Option<String>,
}

fn build_wallpaper_data_stats(index: &WallpaperIndex) -> WallpaperDataStats {
    let wallpapers = index.get_all_wallpapers_unique();
    let latest_end_date = wallpapers
        .first()
        .map(|wallpaper| wallpaper.end_date.clone());
    let earliest_end_date = wallpapers
        .last()
        .map(|wallpaper| wallpaper.end_date.clone());

    WallpaperDataStats {
        count: wallpapers.len(),
        earliest_end_date,
        latest_end_date,
    }
}

/// 获取默认壁纸目录
#[tauri::command]
pub(crate) async fn get_default_wallpaper_directory() -> Result<String, String> {
    storage::get_default_wallpaper_directory()
        .map_err(|e| e.to_string())
        .map(|p| p.to_string_lossy().to_string())
}

/// 获取最后一次成功更新时间（本地时区）
/// 优先从内存状态读取，如果为空则从索引文件读取
#[tauri::command]
pub(crate) async fn get_last_update_time(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    {
        let guard = state.last_update_time.lock().await;
        if let Some(dt) = *guard {
            return Ok(Some(dt.format("%Y-%m-%d %H:%M:%S").to_string()));
        }
    }

    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    let index_manager = index_manager::IndexManager::new(wallpaper_dir.clone());
    match index_manager.load_index().await {
        Ok(index) => {
            let local_time = index.last_updated.with_timezone(&Local);
            Ok(Some(local_time.format("%Y-%m-%d %H:%M:%S").to_string()))
        }
        Err(_) => Ok(None),
    }
}

#[tauri::command]
pub(crate) async fn get_update_in_progress(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let flag = state.update_in_progress.lock().await;
    Ok(*flag)
}

/// 确保壁纸目录存在
#[tauri::command]
pub(crate) async fn ensure_wallpaper_directory_exists(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    storage::ensure_wallpaper_directory(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())
}

/// 获取当前壁纸目录（用户自定义或默认）
#[tauri::command]
pub(crate) async fn get_wallpaper_directory(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let wallpaper_dir = state.wallpaper_directory.lock().await;
    Ok(wallpaper_dir.to_string_lossy().to_string())
}

/// 获取当前壁纸目录中的全量唯一壁纸数据统计
#[tauri::command]
pub(crate) async fn get_wallpaper_data_stats(
    state: tauri::State<'_, AppState>,
) -> Result<WallpaperDataStats, String> {
    let wallpaper_dir = {
        let dir = state.wallpaper_directory.lock().await;
        dir.clone()
    };

    let index = storage::get_index_snapshot(&wallpaper_dir)
        .await
        .map_err(|e| e.to_string())?;

    Ok(build_wallpaper_data_stats(&index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::LocalWallpaper;

    fn make_wallpaper(end_date: &str, title: &str) -> LocalWallpaper {
        LocalWallpaper {
            title: title.to_string(),
            copyright: format!("Copyright for {}", title),
            copyright_link: "https://example.com".to_string(),
            end_date: end_date.to_string(),
            urlbase: format!("/th?id=OHR.{}", title),
        }
    }

    #[test]
    fn test_build_wallpaper_data_stats_dedupes_by_end_date() {
        let mut index = WallpaperIndex::new();
        index.upsert_wallpapers_for_mkt(
            "zh-CN",
            vec![
                make_wallpaper("20240103", "Third"),
                make_wallpaper("20240101", "First"),
            ],
        );
        index.upsert_wallpapers_for_mkt(
            "en-US",
            vec![
                make_wallpaper("20240103", "Third English"),
                make_wallpaper("20240102", "Second"),
            ],
        );

        let stats = build_wallpaper_data_stats(&index);

        assert_eq!(stats.count, 3);
        assert_eq!(stats.earliest_end_date.as_deref(), Some("20240101"));
        assert_eq!(stats.latest_end_date.as_deref(), Some("20240103"));
    }

    #[test]
    fn test_build_wallpaper_data_stats_empty_index() {
        let index = WallpaperIndex::new();
        let stats = build_wallpaper_data_stats(&index);

        assert_eq!(stats.count, 0);
        assert_eq!(stats.earliest_end_date, None);
        assert_eq!(stats.latest_end_date, None);
    }
}
