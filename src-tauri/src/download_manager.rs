use anyhow::{Context, Result};
use futures::stream::{self, StreamExt};
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 全局 HTTP 客户端，复用连接池
///
/// 配置说明：
/// - pool_max_idle_per_host: 每个主机最多保持 8 个空闲连接
/// - pool_idle_timeout: 连接空闲 90 秒后自动关闭
/// - timeout: 请求总超时时间 60 秒
/// - connect_timeout: 连接建立超时 10 秒
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(8)
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .timeout(Duration::from_secs(60))
        .connect_timeout(Duration::from_secs(10))
        .user_agent("BingWallpaperNow/0.2.0")
        .build()
        .expect("Failed to create HTTP client")
});

/// 下载图片到指定路径（使用全局客户端）
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
pub async fn download_image(url: &str, save_path: &Path) -> Result<()> {
    download_image_with_retry(url, save_path, 3).await
}

/// 带重试机制的图片下载
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
/// * `max_retries` - 最大重试次数
async fn download_image_with_retry(url: &str, save_path: &Path, max_retries: usize) -> Result<()> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        match download_image_internal(url, save_path).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                if attempts < max_retries {
                    // 指数退避: 1s, 2s, 4s
                    let delay = Duration::from_secs(1 << (attempts - 1));
                    log::debug!(
                        "Download failed (attempt {}/{}), retrying after {:?}...",
                        attempts,
                        max_retries,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    Err(last_error
        .unwrap()
        .context(format!("Failed to download after {} attempts", max_retries)))
}

/// 内部下载实现（使用全局客户端和流式传输）
async fn download_image_internal(url: &str, save_path: &Path) -> Result<()> {
    // 检查文件是否已存在
    if save_path.exists() {
        log::debug!("File already exists, skipping download: {:?}", save_path);
        return Ok(());
    }

    // 创建父目录(如果不存在)
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent)
            .await
            .context("Failed to create parent directory")?;
    }

    // 使用全局客户端发起请求
    let response = HTTP_CLIENT
        .get(url)
        .send()
        .await
        .context("Failed to send request")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", response.status());
    }

    // 流式下载：边下载边写入磁盘，减少内存占用
    let mut stream = response.bytes_stream();
    let temp_path = save_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path)
        .await
        .context("Failed to create temporary file")?;

    let mut downloaded = 0u64;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk")?;
        downloaded += chunk.len() as u64;
    }

    log::debug!("Downloaded {} bytes to {:?}", downloaded, temp_path);

    // 确保数据写入磁盘
    file.sync_all().await.context("Failed to sync file")?;

    // 原子重命名为最终文件名
    fs::rename(&temp_path, save_path)
        .await
        .context("Failed to rename temporary file")?;

    Ok(())
}

/// 并发下载多张壁纸
///
/// # Arguments
/// * `download_tasks` - 下载任务列表 [(url, save_path)]
/// * `max_concurrent` - 最大并发数（默认 4）
///
/// # Returns
/// 返回所有下载结果，成功返回路径，失败返回错误
pub async fn download_images_concurrent(
    download_tasks: Vec<(String, PathBuf)>,
    max_concurrent: usize,
) -> Vec<Result<PathBuf>> {
    log::info!(
        "Starting concurrent download of {} images (max_concurrent={})",
        download_tasks.len(),
        max_concurrent
    );

    let start = std::time::Instant::now();

    let results = stream::iter(download_tasks)
        .map(|(url, save_path)| async move {
            let result = download_image(&url, &save_path).await;
            match &result {
                Ok(_) => log::debug!("Successfully downloaded: {:?}", save_path),
                Err(e) => log::error!("Failed to download {:?}: {}", save_path, e),
            }
            result.map(|_| save_path)
        })
        .buffer_unordered(max_concurrent) // 并发执行
        .collect::<Vec<_>>()
        .await;

    let elapsed = start.elapsed();
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    log::info!(
        "Concurrent download completed: {}/{} successful in {:?}",
        success_count,
        results.len(),
        elapsed
    );

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_download_image_creates_file() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_download_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path = temp_dir.join("test.jpg");

        // 使用一个小的测试图片 URL（Bing API）
        let url = "https://www.bing.com/th?id=OHR.BingWallpaper_ZH-CN0000000000_UHD.jpg";

        // 实际下载测试（仅在显式启用时运行）
        if std::env::var("BING_TEST").ok().as_deref() == Some("1") {
            let result = download_image(url, &save_path).await;
            assert!(result.is_ok());
            assert!(save_path.exists());

            // 验证可以跳过已存在的文件
            let result2 = download_image(url, &save_path).await;
            assert!(result2.is_ok());
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_concurrent_download() {
        // 测试并发下载逻辑（不实际下载）
        let tasks = vec![
            (
                "https://example.com/1.jpg".to_string(),
                PathBuf::from("/tmp/1.jpg"),
            ),
            (
                "https://example.com/2.jpg".to_string(),
                PathBuf::from("/tmp/2.jpg"),
            ),
        ];

        // 不实际执行网络请求，仅验证 API 可用性
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_download_invalid_url() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_invalid_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path = temp_dir.join("invalid.jpg");
        let invalid_url = "https://invalid-domain-that-does-not-exist-12345.com/image.jpg";

        // 测试无效 URL 的错误处理
        let result = download_image(invalid_url, &save_path).await;
        assert!(result.is_err());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_download_to_invalid_directory() {
        // 测试无效目录的错误处理
        let invalid_path = PathBuf::from("/nonexistent/directory/that/does/not/exist/test.jpg");
        let url = "https://example.com/test.jpg";

        let result = download_image(url, &invalid_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_download_empty_list() {
        // 测试空任务列表
        let tasks: Vec<(String, PathBuf)> = vec![];
        let results = download_images_concurrent(tasks, 4).await;

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_concurrent_download_single_task() {
        // 测试单个任务
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_single_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path = temp_dir.join("single.jpg");
        let tasks = vec![(
            "https://invalid-url.com/test.jpg".to_string(),
            save_path.clone(),
        )];

        let results = download_images_concurrent(tasks, 1).await;

        assert_eq!(results.len(), 1);
        // 应该失败（无效 URL）
        assert!(results[0].is_err());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_concurrent_download_mixed_results() {
        // 测试混合成功/失败的场景
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_mixed_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let tasks = vec![
            (
                "https://invalid-url-1.com/test.jpg".to_string(),
                temp_dir.join("1.jpg"),
            ),
            (
                "https://invalid-url-2.com/test.jpg".to_string(),
                temp_dir.join("2.jpg"),
            ),
            (
                "https://invalid-url-3.com/test.jpg".to_string(),
                temp_dir.join("3.jpg"),
            ),
        ];

        let results = download_images_concurrent(tasks, 2).await;

        assert_eq!(results.len(), 3);
        // 所有应该失败（无效 URL）
        assert!(results.iter().all(|r| r.is_err()));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_download_skips_existing_file() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_existing_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path = temp_dir.join("existing.jpg");

        // 创建一个已存在的文件
        fs::write(&save_path, b"existing content").await.unwrap();
        let original_content = fs::read(&save_path).await.unwrap();

        // 尝试下载到已存在的文件
        let url = "https://example.com/test.jpg";
        let result = download_image(url, &save_path).await;

        // 应该成功（跳过下载）
        assert!(result.is_ok());

        // 文件内容应该保持不变
        let current_content = fs::read(&save_path).await.unwrap();
        assert_eq!(original_content, current_content);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_concurrent_download_max_concurrent_parameter() {
        // 测试不同的并发参数
        let tasks: Vec<(String, PathBuf)> = (0..10)
            .map(|i| {
                (
                    format!("https://example.com/{}.jpg", i),
                    PathBuf::from(format!("/tmp/test_{}.jpg", i)),
                )
            })
            .collect();

        // 测试 max_concurrent = 1 (顺序执行)
        let results_1 = download_images_concurrent(tasks.clone(), 1).await;
        assert_eq!(results_1.len(), 10);

        // 测试 max_concurrent = 5
        let results_5 = download_images_concurrent(tasks.clone(), 5).await;
        assert_eq!(results_5.len(), 10);

        // 测试 max_concurrent = 20 (超过任务数)
        let results_20 = download_images_concurrent(tasks, 20).await;
        assert_eq!(results_20.len(), 10);
    }

    #[tokio::test]
    async fn test_http_client_reuse() {
        // 测试全局 HTTP 客户端可以被多次调用
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_reuse_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        // 进行多次下载以测试连接池复用
        for i in 0..3 {
            let save_path = temp_dir.join(format!("test_{}.jpg", i));
            let url = "https://invalid-url.com/test.jpg";

            let result = download_image(url, &save_path).await;
            // 所有请求应该都失败但不会 panic
            assert!(result.is_err());
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
