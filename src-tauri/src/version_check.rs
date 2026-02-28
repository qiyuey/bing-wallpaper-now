use crate::runtime_state;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tauri::AppHandle;

/// GitHub Releases API 响应结构
#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

/// GitHub Release Asset 结构
#[derive(Debug, Deserialize)]
pub(crate) struct GitHubAsset {
    pub name: String,
    #[serde(rename = "browser_download_url", skip_deserializing)]
    pub _browser_download_url: String,
}

/// 版本检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct VersionCheckResult {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub has_update: bool,
    pub release_url: Option<String>,
    pub platform_available: bool,
}

/// 添加版本到"不再提醒"列表（保存最大版本）
#[tauri::command]
pub(crate) async fn add_ignored_update_version(
    app: AppHandle,
    version: String,
) -> Result<(), String> {
    let mut runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    let should_update = runtime_state
        .ignored_update_version
        .as_ref()
        .map(|ignored| compare_versions(ignored, &version) < 0)
        .unwrap_or(true);

    if should_update {
        runtime_state.ignored_update_version = Some(version.clone());
        runtime_state::save_runtime_state(&app, &runtime_state)
            .map_err(|e| format!("Failed to save runtime state: {}", e))?;
        info!(
            target: "version_check",
            "Updated ignored update version to: {}",
            version
        );
    }

    Ok(())
}

/// 检查版本是否应该被忽略（版本小于等于忽略的版本）
#[tauri::command]
pub(crate) async fn is_version_ignored(app: AppHandle, version: String) -> Result<bool, String> {
    let runtime_state = runtime_state::load_runtime_state(&app)
        .map_err(|e| format!("Failed to load runtime state: {}", e))?;

    match runtime_state.ignored_update_version {
        Some(ref ignored_version) => Ok(compare_versions(&version, ignored_version) <= 0),
        None => Ok(false),
    }
}

/// 检查 GitHub Releases 是否有新版本
///
/// # Returns
/// 返回版本检查结果，包含当前版本、最新版本和是否有更新
#[tauri::command]
pub(crate) async fn check_for_updates() -> Result<VersionCheckResult, String> {
    const GITHUB_API_URL: &str =
        "https://api.github.com/repos/qiyuey/bing-wallpaper-now/releases/latest";
    const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

    let is_dev_version = CURRENT_VERSION.contains('-');
    let current_version = CURRENT_VERSION
        .split('-')
        .next()
        .unwrap_or(CURRENT_VERSION)
        .to_string();

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("Bing-Wallpaper-Now/1.0")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    match client.get(GITHUB_API_URL).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<GitHubRelease>().await {
                    Ok(release) => {
                        let latest_version = release.tag_name.trim_start_matches('v').to_string();

                        let platform_available = has_platform_asset(&release.assets);

                        // 开发版本（如 1.1.5-0）视为比同号正式版（1.1.5）更旧
                        let cmp = compare_versions(&current_version, &latest_version);
                        let has_update =
                            platform_available && (cmp < 0 || (cmp == 0 && is_dev_version));

                        info!(
                            target: "version_check",
                            "Version check completed: current={}, latest={}, has_update={}, platform_available={}",
                            current_version,
                            latest_version,
                            has_update,
                            platform_available
                        );

                        Ok(VersionCheckResult {
                            current_version,
                            latest_version: Some(latest_version),
                            has_update,
                            release_url: Some(release.html_url),
                            platform_available,
                        })
                    }
                    Err(e) => {
                        warn!(target: "version_check", "Failed to parse GitHub release response: {}", e);
                        Ok(VersionCheckResult {
                            current_version,
                            latest_version: None,
                            has_update: false,
                            release_url: None,
                            platform_available: false,
                        })
                    }
                }
            } else {
                warn!(
                    target: "version_check",
                    "GitHub API returned status: {}",
                    response.status()
                );
                Ok(VersionCheckResult {
                    current_version,
                    latest_version: None,
                    has_update: false,
                    release_url: None,
                    platform_available: false,
                })
            }
        }
        Err(e) => {
            warn!(target: "version_check", "Failed to check for updates: {}", e);
            Ok(VersionCheckResult {
                current_version,
                latest_version: None,
                has_update: false,
                release_url: None,
                platform_available: false,
            })
        }
    }
}

/// 获取当前平台应该使用的安装包文件扩展名
fn get_platform_extensions() -> Vec<&'static str> {
    #[cfg(target_os = "windows")]
    {
        vec![".msi", ".exe"]
    }
    #[cfg(target_os = "macos")]
    {
        vec![".dmg"]
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        vec![".deb", ".rpm", ".AppImage"]
    }
}

/// 检查 assets 中是否有当前平台的安装包
fn has_platform_asset(assets: &[GitHubAsset]) -> bool {
    let extensions = get_platform_extensions();
    assets
        .iter()
        .any(|asset| extensions.iter().any(|ext| asset.name.ends_with(ext)))
}

/// 比较两个版本号字符串
///
/// # Returns
/// - 负数：如果 version1 < version2
/// - 0：如果 version1 == version2
/// - 正数：如果 version1 > version2
pub(crate) fn compare_versions(version1: &str, version2: &str) -> i32 {
    let v1_parts: Vec<u32> = version1
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();
    let v2_parts: Vec<u32> = version2
        .split('.')
        .map(|s| s.parse().unwrap_or(0))
        .collect();

    let max_len = v1_parts.len().max(v2_parts.len());

    for i in 0..max_len {
        let v1_part = v1_parts.get(i).copied().unwrap_or(0);
        let v2_part = v2_parts.get(i).copied().unwrap_or(0);

        match v1_part.cmp(&v2_part) {
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Equal => continue,
        }
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(compare_versions("1.0.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert!(compare_versions("1.0.1", "1.0.0") > 0);

        assert_eq!(compare_versions("1.0", "1.0.0"), 0);
        assert!(compare_versions("1.0.0", "1.0.1") < 0);
        assert!(compare_versions("1.0.1", "1.0") > 0);

        assert!(compare_versions("0.9.9", "1.0.0") < 0);
        assert!(compare_versions("1.0.0", "2.0.0") < 0);

        assert!(compare_versions("1.0.0", "1.1.0") < 0);
        assert!(compare_versions("1.1.0", "1.0.0") > 0);

        assert_eq!(compare_versions("invalid", "0.0.0"), 0);
        assert_eq!(compare_versions("1.0.invalid", "1.0.0"), 0);
    }

    #[test]
    fn test_dev_version_update_detection() {
        fn has_update(current: &str, latest: &str, platform_available: bool) -> bool {
            let is_dev = current.contains('-');
            let current_clean = current.split('-').next().unwrap_or(current);
            let cmp = compare_versions(current_clean, latest);
            platform_available && (cmp < 0 || (cmp == 0 && is_dev))
        }

        assert!(has_update("1.1.5-0", "1.1.5", true));
        assert!(!has_update("1.1.5", "1.1.5", true));
        assert!(has_update("1.1.5-0", "1.2.0", true));
        assert!(!has_update("1.1.5-0", "1.1.4", true));
        assert!(!has_update("1.1.5-0", "1.1.5", false));
        assert!(!has_update("1.0.0", "1.1.0", false));
    }

    #[test]
    fn test_has_platform_asset() {
        #[cfg(target_os = "windows")]
        {
            let assets = vec![
                GitHubAsset {
                    name: "Bing.Wallpaper.Now_0.4.6_x64_zh-CN.msi".to_string(),
                    _browser_download_url: "https://example.com/test.msi".to_string(),
                },
                GitHubAsset {
                    name: "Bing.Wallpaper.Now_0.4.6_x64-setup.exe".to_string(),
                    _browser_download_url: "https://example.com/test.exe".to_string(),
                },
                GitHubAsset {
                    name: "test.dmg".to_string(),
                    _browser_download_url: "https://example.com/test.dmg".to_string(),
                },
            ];
            assert!(has_platform_asset(&assets));
            assert!(!has_platform_asset(&[]));
        }

        #[cfg(target_os = "macos")]
        {
            let assets = vec![GitHubAsset {
                name: "Bing.Wallpaper.Now_0.4.6_aarch64.dmg".to_string(),
                _browser_download_url: "https://example.com/test.dmg".to_string(),
            }];
            assert!(has_platform_asset(&assets));

            let assets_false = vec![GitHubAsset {
                name: "test.msi".to_string(),
                _browser_download_url: "https://example.com/test.msi".to_string(),
            }];
            assert!(!has_platform_asset(&assets_false));
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let assets = vec![GitHubAsset {
                name: "bing-wallpaper-now_0.4.6_amd64.deb".to_string(),
                _browser_download_url: "https://example.com/test.deb".to_string(),
            }];
            assert!(has_platform_asset(&assets));

            let assets_false = vec![GitHubAsset {
                name: "test.msi".to_string(),
                _browser_download_url: "https://example.com/test.msi".to_string(),
            }];
            assert!(!has_platform_asset(&assets_false));
        }
    }
}
