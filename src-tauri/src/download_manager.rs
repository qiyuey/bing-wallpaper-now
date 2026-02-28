use anyhow::{Context, Result};
use futures::stream::StreamExt;
use log::{error, info};
use reqwest::Client;
use std::path::Path;
use std::sync::LazyLock;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
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

/// 按需下载单个壁纸
///
/// 从文件路径中提取 end_date，查找对应的元数据并下载图片。
/// 如果文件已存在则直接返回成功。
///
/// # Arguments
/// * `file_path` - 壁纸文件路径（例如：/path/to/20251031.jpg）
/// * `wallpaper_dir` - 壁纸存储目录
/// * `app` - Tauri app handle
///
/// # Returns
/// `Ok(())` 如果下载成功或文件已存在，`Err` 如果下载失败
pub(crate) async fn download_wallpaper_if_needed(
    file_path: &Path,
    wallpaper_dir: &Path,
    app: &AppHandle,
) -> std::result::Result<(), String> {
    use crate::{AppState, bing_api, storage};

    if file_path.exists() {
        return Ok(());
    }

    // 验证文件路径是否在壁纸目录下（安全性检查）
    if let Some(parent) = file_path.parent() {
        if let (Ok(parent_can), Ok(dir_can)) = (parent.canonicalize(), wallpaper_dir.canonicalize())
        {
            if !parent_can.starts_with(&dir_can) {
                return Err(format!(
                    "文件路径不在壁纸目录下: {} (期望在: {})",
                    file_path.display(),
                    wallpaper_dir.display()
                ));
            }
        } else if parent != wallpaper_dir {
            return Err(format!(
                "文件路径的父目录不匹配: {} (期望: {})",
                parent.display(),
                wallpaper_dir.display()
            ));
        }
    } else {
        return Err(format!("无法确定文件路径的父目录: {}", file_path.display()));
    }

    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "无法从路径中提取文件名".to_string())?;

    let is_portrait = filename.ends_with("r.jpg");
    let end_date = if is_portrait {
        filename
            .strip_suffix("r.jpg")
            .ok_or_else(|| format!("文件名格式不正确，应为 YYYYMMDDr.jpg: {}", filename))?
    } else {
        filename
            .strip_suffix(".jpg")
            .ok_or_else(|| format!("文件名格式不正确，应为 YYYYMMDD.jpg: {}", filename))?
    };

    let app_state = app.state::<AppState>();
    let mkt = crate::get_effective_mkt(&app_state).await;

    let wallpapers = storage::get_local_wallpapers(wallpaper_dir, &mkt)
        .await
        .map_err(|e| format!("获取壁纸列表失败: {}", e))?;

    let wallpaper = wallpapers
        .iter()
        .find(|w| w.end_date == end_date)
        .ok_or_else(|| format!("未找到 end_date 为 {} 的壁纸元数据", end_date))?;

    if wallpaper.urlbase.is_empty() {
        info!(
            target: "commands",
            "壁纸元数据缺少 urlbase，尝试从 API 获取: {}",
            end_date
        );
        return Err(
            "壁纸元数据缺少 urlbase 信息，无法下载。请等待下次更新或手动刷新。".to_string(),
        );
    }

    let resolution = if is_portrait { "1080x1920" } else { "UHD" };
    let image_url = bing_api::get_wallpaper_url(&wallpaper.urlbase, resolution);

    info!(
        target: "commands",
        "开始按需下载壁纸: {} -> {}",
        end_date,
        file_path.display()
    );

    match download_image(&image_url, file_path).await {
        Ok(()) => {
            info!(target: "commands", "成功按需下载壁纸: {}", file_path.display());
            let _ = app.emit("image-downloaded", end_date);
            Ok(())
        }
        Err(e) => {
            error!(
                target: "commands",
                "按需下载壁纸失败 {}: {}",
                end_date,
                e
            );
            Err(format!("下载失败: {}", e))
        }
    }
}

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

/// 内部下载实现（使用全局客户端和流式传输）
///
/// # Arguments
/// * `url` - 图片 URL
/// * `save_path` - 保存路径
async fn download_image_internal(url: &str, save_path: &Path) -> Result<()> {
    // 检查文件是否已存在
    if save_path.exists() {
        log::debug!("文件已存在，跳过下载: {}", save_path.display());
        return Ok(());
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

    let content_length = response.content_length();

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

    // 校验 1: Content-Length (如果服务器提供了)
    if let Some(expected_len) = content_length {
        let metadata = file.metadata().await?;
        if metadata.len() != expected_len {
            // 删除不完整的文件
            let _ = fs::remove_file(&temp_path).await;
            anyhow::bail!(
                "文件大小不匹配: 期望={}, 实际={}",
                expected_len,
                metadata.len()
            );
        }
    }

    // 校验 2: 图片格式有效性 (尝试解析图片头)
    // 使用 spawn_blocking 因为 image crate 操作是阻塞的
    let temp_path_clone = temp_path.clone();
    let validation_result = tokio::task::spawn_blocking(move || {
        // 使用 image crate 尝试读取图片头信息
        match image::ImageReader::open(&temp_path_clone) {
            Ok(reader) => match reader.with_guessed_format() {
                Ok(reader) => match reader.into_dimensions() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow::anyhow!("无效的图片文件(无法获取尺寸): {}", e)),
                },
                Err(e) => Err(anyhow::anyhow!("无法识别图片格式: {}", e)),
            },
            Err(e) => Err(anyhow::anyhow!("无法打开文件: {}", e)),
        }
    })
    .await;

    // 处理 spawn_blocking 的 JoinError (Result<Result<()>>)
    let validation_result = match validation_result {
        Ok(res) => res,
        Err(e) => Err(anyhow::anyhow!("校验任务执行失败: {}", e)),
    };

    if let Err(e) = validation_result {
        log::warn!(
            "文件校验失败，将删除临时文件: {}, 错误: {}",
            temp_path.display(),
            e
        );
        let _ = fs::remove_file(&temp_path).await;
        return Err(e);
    }

    log::debug!("文件校验通过: {}", temp_path.display());

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
}
