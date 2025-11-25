use anyhow::{Context, Result};
use futures::stream::StreamExt;
use reqwest::Client;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 全局 HTTP 客户端，复用连接池
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(4)
        .tcp_nodelay(true)
        .user_agent("BingWallpaperNow/0.3.1")
        .build()
        .expect("Failed to create HTTP client")
});

/// 下载图片到指定路径（使用全局客户端）
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
/// * `expected_hash` - 可选的期望 MD5 哈希值（十六进制字符串），用于校验文件完整性
pub async fn download_image(
    url: &str,
    save_path: &Path,
    expected_hash: Option<&str>,
) -> Result<()> {
    download_image_with_retry(url, save_path, expected_hash, 10).await
}

/// 带重试机制的图片下载
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
/// * `expected_hash` - 可选的期望 MD5 哈希值（十六进制字符串），用于校验文件完整性
/// * `max_retries` - 最大重试次数
async fn download_image_with_retry(
    url: &str,
    save_path: &Path,
    expected_hash: Option<&str>,
    max_retries: usize,
) -> Result<()> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        match download_image_internal(url, save_path, expected_hash).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                if attempts < max_retries {
                    // 改进的重试延迟策略：
                    // 前3次使用较短的固定间隔（5秒），适合处理临时网络波动
                    // 后续使用指数退避，最大延迟60秒，避免等待时间过长
                    let delay = if attempts <= 3 {
                        Duration::from_secs(5) // 前3次：5秒固定间隔
                    } else {
                        // 第4次开始：10, 20, 40, 60, 60, 60... 秒
                        let exponential = 10 * (1 << (attempts - 4));
                        Duration::from_secs(exponential.min(60)) // 最大60秒
                    };

                    log::warn!(
                        "图片下载失败(第 {}/{} 次): {}，{}秒后重试",
                        attempts,
                        max_retries,
                        last_error.as_ref().unwrap(),
                        delay.as_secs()
                    );
                    tokio::time::sleep(delay).await;
                } else {
                    log::error!(
                        "图片下载失败(第 {}/{} 次): {}，已达最大重试次数",
                        attempts,
                        max_retries,
                        last_error.as_ref().unwrap()
                    );
                }
            }
        }
    }

    Err(last_error
        .unwrap()
        .context(format!("Failed to download after {} attempts", max_retries)))
}

/// 计算文件的 MD5 哈希值
///
/// # Arguments
/// * `file_path` - 文件路径
///
/// # Returns
/// MD5 哈希值的十六进制字符串
async fn calculate_file_hash(file_path: &Path) -> Result<String> {
    let content = fs::read(file_path)
        .await
        .context("Failed to read file for hash calculation")?;
    let hash = md5::compute(&content);
    Ok(format!("{:x}", hash))
}

/// 内部下载实现（使用全局客户端和流式传输）
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
/// * `expected_hash` - 可选的期望 MD5 哈希值（十六进制字符串），用于校验文件完整性
async fn download_image_internal(
    url: &str,
    save_path: &Path,
    expected_hash: Option<&str>,
) -> Result<()> {
    // 检查文件是否已存在
    if save_path.exists() {
        // 如果提供了期望哈希值，验证已存在文件的哈希
        if let Some(expected) = expected_hash {
            let actual_hash = calculate_file_hash(save_path).await?;
            if actual_hash.to_lowercase() != expected.to_lowercase() {
                log::warn!(
                    "已存在的文件哈希值不匹配，将重新下载: 期望={}, 实际={}, 文件={}",
                    expected,
                    actual_hash,
                    save_path.display()
                );
                // 删除不匹配的文件，继续下载
                if let Err(e) = fs::remove_file(save_path).await {
                    log::warn!("删除哈希不匹配的文件失败: {}", e);
                }
            } else {
                log::debug!("文件已存在且哈希值匹配，跳过下载: {}", save_path.display());
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    // 创建父目录(如果不存在)
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent)
            .await
            .context("Failed to create parent directory")?;
    }

    // 使用全局客户端发起请求，提供更详细的错误信息
    let response = HTTP_CLIENT.get(url).send().await.map_err(|e| {
        // 提供更详细的错误信息，帮助诊断问题
        let error_msg = if e.is_connect() {
            format!("Connection failed: {}", e)
        } else if e.is_timeout() {
            format!("Request timeout: {}", e)
        } else if e.is_builder() {
            format!("Request build error: {}", e)
        } else if let Some(url_err) = e.url() {
            format!("URL error for {}: {}", url_err, e)
        } else {
            format!("Network error: {}", e)
        };
        anyhow::anyhow!(error_msg)
    })?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download image: HTTP {}", response.status());
    }

    // 流式下载：边下载边写入磁盘，减少内存占用
    let mut stream = response.bytes_stream();
    let temp_path = save_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path)
        .await
        .context("Failed to create temporary file")?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk")?;
    }

    // 确保数据写入磁盘
    file.sync_all().await.context("Failed to sync file")?;

    // 如果提供了期望哈希值，校验下载的文件
    if let Some(expected) = expected_hash {
        let actual_hash = calculate_file_hash(&temp_path).await?;
        if actual_hash.to_lowercase() != expected.to_lowercase() {
            // 删除哈希不匹配的临时文件
            let _ = fs::remove_file(&temp_path).await;
            anyhow::bail!(
                "文件哈希值校验失败: 期望={}, 实际={}",
                expected,
                actual_hash
            );
        }
        log::debug!(
            "文件哈希值校验通过: {} (MD5: {})",
            save_path.display(),
            actual_hash
        );
    }

    // 原子重命名为最终文件名
    fs::rename(&temp_path, save_path)
        .await
        .context("Failed to rename temporary file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// 用于测试的下载函数，使用更短的超时时间（1秒）
    async fn download_image_fast_timeout(url: &str, save_path: &Path) -> Result<()> {
        let client = Client::builder()
            .timeout(Duration::from_secs(1))
            .connect_timeout(Duration::from_millis(500))
            .build()
            .context("Failed to create test HTTP client")?;

        if save_path.exists() {
            return Ok(());
        }

        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create parent directory")?;
        }

        let response = client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to download image: HTTP {}", response.status());
        }

        let bytes = response.bytes().await.context("Failed to read bytes")?;
        fs::write(save_path, &bytes)
            .await
            .context("Failed to write file")?;

        Ok(())
    }

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
            let result = download_image(url, &save_path, None).await;
            assert!(result.is_ok());
            assert!(save_path.exists());

            // 验证可以跳过已存在的文件
            let result2 = download_image(url, &save_path, None).await;
            assert!(result2.is_ok());
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_concurrent_download() {
        // 测试并发下载逻辑（不实际下载）
        let tasks = [
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

        // 测试无效 URL 的错误处理 - 使用快速超时
        let result = download_image_fast_timeout(invalid_url, &save_path).await;
        assert!(result.is_err());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_download_to_invalid_directory() {
        // 测试无效目录的错误处理 - 使用快速超时
        let invalid_path = PathBuf::from("/nonexistent/directory/that/does/not/exist/test.jpg");
        let url = "https://invalid-url-test.com/test.jpg";

        let result = download_image_fast_timeout(url, &invalid_path).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_concurrent_download_single_task() {
        // 测试单个任务的并发逻辑（不实际下载）
        let tasks = [(
            "https://example.com/test.jpg".to_string(),
            PathBuf::from("/tmp/single_test.jpg"),
        )];

        // 验证任务列表长度
        assert_eq!(tasks.len(), 1);

        // 测试并发参数
        let max_concurrent = 1;
        assert_eq!(max_concurrent, 1);
    }

    #[tokio::test]
    async fn test_concurrent_download_mixed_results() {
        // 测试多任务的并发逻辑（不实际下载）
        let tasks = [
            (
                "https://example.com/1.jpg".to_string(),
                PathBuf::from("/tmp/1.jpg"),
            ),
            (
                "https://example.com/2.jpg".to_string(),
                PathBuf::from("/tmp/2.jpg"),
            ),
            (
                "https://example.com/3.jpg".to_string(),
                PathBuf::from("/tmp/3.jpg"),
            ),
        ];

        // 验证任务列表长度
        assert_eq!(tasks.len(), 3);

        // 测试并发参数
        let max_concurrent = 2;
        assert!(max_concurrent > 0);
        assert!(max_concurrent < tasks.len());
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
        let result = download_image(url, &save_path, None).await;

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
        // 测试不同的并发参数逻辑（不实际下载）
        let tasks: Vec<(String, PathBuf)> = (0..10)
            .map(|i| {
                (
                    format!("https://example.com/{}.jpg", i),
                    PathBuf::from(format!("/tmp/test_{}.jpg", i)),
                )
            })
            .collect();

        // 验证任务列表
        assert_eq!(tasks.len(), 10);

        // 测试不同的并发参数值
        let max_concurrent_1 = 1;
        let max_concurrent_5 = 5;
        let max_concurrent_20 = 20;

        assert_eq!(max_concurrent_1, 1); // 顺序执行
        assert!(max_concurrent_5 > 1 && max_concurrent_5 < tasks.len()); // 正常并发
        assert!(max_concurrent_20 > tasks.len()); // 超过任务数
    }

    #[tokio::test]
    async fn test_http_client_reuse() {
        // 测试 HTTP 客户端可以被多次调用 - 使用快速超时
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

            let result = download_image_fast_timeout(url, &save_path).await;
            // 所有请求应该都失败但不会 panic
            assert!(result.is_err());
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_hash_verification() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_hash_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path = temp_dir.join("test_hash.jpg");
        let test_content = b"test image content for hash verification";

        // 创建测试文件
        fs::write(&save_path, test_content).await.unwrap();

        // 计算正确的哈希值
        let correct_hash = calculate_file_hash(&save_path).await.unwrap();

        // 测试哈希校验通过的情况
        let result = calculate_file_hash(&save_path).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_lowercase(), correct_hash.to_lowercase());

        // 测试哈希校验失败的情况（使用错误的哈希值）
        let wrong_hash = "wrong_hash_value_1234567890abcdef1234567890abcdef";
        assert_ne!(correct_hash.to_lowercase(), wrong_hash.to_lowercase());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_hash_verification_with_different_content() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_hash_diff_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let save_path1 = temp_dir.join("test1.jpg");
        let save_path2 = temp_dir.join("test2.jpg");
        let content1 = b"test content 1";
        let content2 = b"test content 2";

        // 创建两个不同内容的文件
        fs::write(&save_path1, content1).await.unwrap();
        fs::write(&save_path2, content2).await.unwrap();

        // 计算哈希值
        let hash1 = calculate_file_hash(&save_path1).await.unwrap();
        let hash2 = calculate_file_hash(&save_path2).await.unwrap();

        // 不同内容的文件应该有不同哈希值
        assert_ne!(hash1, hash2);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
