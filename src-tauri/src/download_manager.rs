use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 下载图片到指定路径
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
pub async fn download_image(url: &str, save_path: &Path) -> Result<()> {
    // 创建父目录(如果不存在)
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent)
            .await
            .context("Failed to create parent directory")?;
    }

    // 检查文件是否已存在
    if save_path.exists() {
        return Ok(());
    }

    // 下载图片
    let response = reqwest::get(url)
        .await
        .context("Failed to download image")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", response.status());
    }

    let bytes = response
        .bytes()
        .await
        .context("Failed to read image bytes")?;

    // 保存到临时文件,然后重命名(原子操作)
    let temp_path = save_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path)
        .await
        .context("Failed to create temporary file")?;

    file.write_all(&bytes)
        .await
        .context("Failed to write image data")?;

    file.sync_all().await.context("Failed to sync file")?;

    // 重命名为最终文件名
    fs::rename(&temp_path, save_path)
        .await
        .context("Failed to rename temporary file")?;

    Ok(())
}
