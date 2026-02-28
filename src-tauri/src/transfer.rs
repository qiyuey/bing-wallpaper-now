use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::Emitter;

use crate::{AppState, index_manager, models, storage};

/// 导入/导出结果统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct TransferResult {
    /// 实际新增的元数据条目数（目标索引中原本不存在的）
    metadata_new: usize,
    /// 覆盖更新的元数据条目数（目标索引中已存在相同 key 的）
    metadata_updated: usize,
    /// 因 mkt 验证失败而跳过的条目数
    metadata_skipped: usize,
    images_copied: usize,
    images_skipped: usize,
    images_failed: usize,
    mkt_count: usize,
}

/// 图片复制结果
struct ImageCopyResult {
    copied: usize,
    skipped: usize,
    failed: usize,
}

/// 复制壁纸图片文件（仅复制目标目录中不存在的文件）
///
/// 识别 YYYYMMDD.jpg 和 YYYYMMDDr.jpg 格式的壁纸文件，
/// 使用 atomic copy（先写临时文件再 rename）确保数据完整性。
async fn copy_wallpaper_images(
    source_dir: &Path,
    target_dir: &Path,
    log_target: &str,
) -> Result<ImageCopyResult, String> {
    let mut copied: usize = 0;
    let mut skipped: usize = 0;
    let mut failed: usize = 0;

    let mut read_dir = tokio::fs::read_dir(source_dir)
        .await
        .map_err(|e| format!("Failed to read source directory: {}", e))?;

    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| format!("Failed to read directory entry: {}", e))?
    {
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if !name.ends_with(".jpg") {
            continue;
        }
        let stem = name
            .strip_suffix("r.jpg")
            .or_else(|| name.strip_suffix(".jpg"));
        match stem {
            Some(s) if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) => {}
            _ => continue,
        }

        let target_file = target_dir.join(&*name);
        if tokio::fs::try_exists(&target_file).await.unwrap_or(false) {
            skipped += 1;
            continue;
        }

        let source_file = entry.path();
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();
        let temp_file = target_dir.join(format!("{}.{}{:x}.tmp", name, std::process::id(), nonce));
        if let Err(e) = tokio::fs::copy(&source_file, &temp_file).await {
            warn!(target: log_target, "Failed to copy {}: {}", name, e);
            let _ = tokio::fs::remove_file(&temp_file).await;
            failed += 1;
            continue;
        }
        if let Err(e) = tokio::fs::rename(&temp_file, &target_file).await {
            warn!(target: log_target, "Failed to rename temp file {}: {}", name, e);
            let _ = tokio::fs::remove_file(&temp_file).await;
            failed += 1;
            continue;
        }
        copied += 1;
    }

    Ok(ImageCopyResult {
        copied,
        skipped,
        failed,
    })
}

/// 合并元数据到目标目录（best-effort：单个 mkt 失败不中断整体）
///
/// 使用 `storage::save_wallpapers_metadata`，走全局 IndexManager 缓存，
/// 确保写入后同一目录的后续读取能看到最新数据。
async fn merge_metadata_to_directory(
    source_mkt: &indexmap::IndexMap<String, indexmap::IndexMap<String, models::LocalWallpaper>>,
    directory: &Path,
    log_target: &str,
) -> (usize, usize, usize) {
    let mut metadata_new: usize = 0;
    let mut metadata_updated: usize = 0;
    let mut metadata_skipped: usize = 0;

    for (mkt, wallpapers_map) in source_mkt {
        let wallpapers: Vec<_> = wallpapers_map.values().cloned().collect();
        let total = wallpapers.len();
        match storage::save_wallpapers_metadata(wallpapers, directory, mkt).await {
            Ok(result) => {
                metadata_new += result.new_count;
                metadata_updated += result.validated - result.new_count;
                metadata_skipped += total - result.validated;
            }
            Err(e) => {
                warn!(target: log_target, "Failed to merge metadata for mkt {}: {}", mkt, e);
                metadata_skipped += total;
            }
        }
    }

    (metadata_new, metadata_updated, metadata_skipped)
}

/// 检查两个路径是否指向同一目录
fn is_same_directory(a: &Path, b: &Path) -> bool {
    let a_canonical = a.canonicalize().unwrap_or_else(|_| a.to_path_buf());
    let b_canonical = b.canonicalize().unwrap_or_else(|_| b.to_path_buf());
    a_canonical == b_canonical
}

/// 从外部壁纸目录导入数据（index.json + 壁纸图片）
///
/// 读取源目录的 index.json，将元数据合并到当前索引，
/// 并将源目录中的壁纸图片复制到当前壁纸目录。
#[tauri::command]
pub(crate) async fn import_wallpapers(
    source_dir: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<TransferResult, String> {
    let source_path = PathBuf::from(&source_dir);

    if !source_path.is_dir() {
        return Err("NOT_DIRECTORY".to_string());
    }

    let wallpaper_dir = state.wallpaper_directory.lock().await.clone();

    if is_same_directory(&source_path, &wallpaper_dir) {
        return Err("SAME_DIRECTORY".to_string());
    }

    let external_index = index_manager::IndexManager::load_external_index(&source_path)
        .await
        .map_err(|e| format!("Failed to load external index: {}", e))?;

    if external_index.mkt.is_empty() {
        return Err("NO_DATA".to_string());
    }

    storage::ensure_wallpaper_directory(&wallpaper_dir)
        .await
        .map_err(|e| format!("Failed to ensure wallpaper directory: {}", e))?;

    let mkt_count = external_index.mkt.len();
    let (metadata_new, metadata_updated, metadata_skipped) =
        merge_metadata_to_directory(&external_index.mkt, &wallpaper_dir, "import").await;

    let images = copy_wallpaper_images(&source_path, &wallpaper_dir, "import").await?;

    info!(
        target: "import",
        "导入完成: 新增 {} 条, 更新 {} 条, 跳过 {} 条, 图片复制 {} 张, 跳过 {} 张, 失败 {} 张, {} 个 mkt",
        metadata_new, metadata_updated, metadata_skipped,
        images.copied, images.skipped, images.failed, mkt_count
    );

    let _ = app.emit("wallpaper-updated", ());

    Ok(TransferResult {
        metadata_new,
        metadata_updated,
        metadata_skipped,
        images_copied: images.copied,
        images_skipped: images.skipped,
        images_failed: images.failed,
        mkt_count,
    })
}

/// 将当前壁纸数据导出到指定目录（index.json + 壁纸图片）
///
/// 读取当前壁纸目录的 index.json，将元数据合并到目标目录的索引，
/// 并将壁纸图片复制到目标目录。如果目标目录已有数据，执行合并。
#[tauri::command]
pub(crate) async fn export_wallpapers(
    target_dir: String,
    state: tauri::State<'_, AppState>,
) -> Result<TransferResult, String> {
    let target_path = PathBuf::from(&target_dir);

    if !target_path.is_dir() {
        return Err("NOT_DIRECTORY".to_string());
    }

    let wallpaper_dir = state.wallpaper_directory.lock().await.clone();

    if is_same_directory(&wallpaper_dir, &target_path) {
        return Err("SAME_DIRECTORY".to_string());
    }

    let source_index = storage::get_index_snapshot(&wallpaper_dir)
        .await
        .map_err(|e| format!("Failed to load current index: {}", e))?;

    if source_index.mkt.is_empty() {
        return Err("NO_DATA".to_string());
    }

    storage::ensure_wallpaper_directory(&target_path)
        .await
        .map_err(|e| format!("Failed to ensure target directory: {}", e))?;

    let mkt_count = source_index.mkt.len();
    let (metadata_new, metadata_updated, metadata_skipped) =
        merge_metadata_to_directory(&source_index.mkt, &target_path, "export").await;

    let images = copy_wallpaper_images(&wallpaper_dir, &target_path, "export").await?;

    storage::remove_index_manager(&target_path);

    info!(
        target: "export",
        "导出完成: 新增 {} 条, 更新 {} 条, 跳过 {} 条, 图片复制 {} 张, 跳过 {} 张, 失败 {} 张, {} 个 mkt",
        metadata_new, metadata_updated, metadata_skipped,
        images.copied, images.skipped, images.failed, mkt_count
    );

    Ok(TransferResult {
        metadata_new,
        metadata_updated,
        metadata_skipped,
        images_copied: images.copied,
        images_skipped: images.skipped,
        images_failed: images.failed,
        mkt_count,
    })
}
