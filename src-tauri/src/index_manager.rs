use crate::models::{LocalWallpaper, WallpaperIndex};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

/// 索引文件名
const INDEX_FILE: &str = "index.json";

/// 内存缓存的索引管理器
///
/// 提供高效的壁纸元数据管理，使用单一 JSON 文件存储所有元数据，
/// 并在内存中缓存以减少磁盘 I/O。
pub struct IndexManager {
    directory: PathBuf,
    cache: Arc<Mutex<Option<WallpaperIndex>>>,
}

impl IndexManager {
    /// 创建新的索引管理器
    ///
    /// # Arguments
    /// * `directory` - 壁纸存储目录
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取索引文件路径
    fn index_path(&self) -> PathBuf {
        self.directory.join(INDEX_FILE)
    }

    /// 加载索引（优先使用缓存）
    ///
    /// 如果缓存中有数据，直接返回缓存；否则从磁盘加载。
    /// 如果磁盘上没有索引文件，返回空索引。
    pub async fn load_index(&self) -> Result<WallpaperIndex> {
        let index_path = self.index_path();

        // 检查缓存
        {
            let cache = self.cache.lock().await;
            if let Some(index) = cache.as_ref() {
                log::debug!(
                    "使用缓存的索引，包含 {} 种语言，路径: {}",
                    index.wallpapers_by_language.len(),
                    index_path.display()
                );
                return Ok(index.clone());
            }
        }

        // 从磁盘加载
        log::debug!("从磁盘加载索引，路径: {}", index_path.display());
        let index = match self.load_from_disk().await {
            Ok(index) => {
                let lang_count = index.wallpapers_by_language.len();
                let total_wallpapers: usize =
                    index.wallpapers_by_language.values().map(|m| m.len()).sum();
                log::info!(
                    "成功加载索引文件，包含 {} 种语言，共 {} 张壁纸，路径: {}",
                    lang_count,
                    total_wallpapers,
                    index_path.display()
                );
                index
            }
            Err(e) => {
                log::warn!(
                    "索引文件加载失败 ({}), 将使用空索引，路径: {}",
                    e,
                    index_path.display()
                );
                WallpaperIndex::default()
            }
        };

        // 更新缓存
        {
            let mut cache = self.cache.lock().await;
            *cache = Some(index.clone());
        }

        Ok(index)
    }

    /// 从磁盘加载索引
    async fn load_from_disk(&self) -> Result<WallpaperIndex> {
        let path = self.index_path();
        if !path.exists() {
            log::debug!("索引文件不存在，返回空索引，路径: {}", path.display());
            return Ok(WallpaperIndex::default());
        }

        log::debug!("读取索引文件，路径: {}", path.display());
        let contents = fs::read_to_string(&path)
            .await
            .with_context(|| format!("Failed to read index file: {}", path.display()))?;

        log::debug!("解析索引文件内容，大小: {} bytes", contents.len());
        let index: WallpaperIndex = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to deserialize index file: {}", path.display()))?;

        // 版本检查
        if index.version != WallpaperIndex::VERSION {
            log::error!(
                "索引版本不匹配 (期望: {}, 实际: {}), 数据将被重置，路径: {}",
                WallpaperIndex::VERSION,
                index.version,
                path.display()
            );
            // 考虑保存旧索引备份（可选）
            let backup_path = self.index_path().with_extension("backup");
            if let Err(e) = fs::copy(&self.index_path(), &backup_path).await {
                log::warn!("保存索引备份失败: {}", e);
            } else {
                log::info!("已保存旧索引备份到: {}", backup_path.display());
            }
            return Ok(WallpaperIndex::default());
        }

        log::debug!("索引文件版本检查通过，版本: {}", index.version);
        Ok(index)
    }

    /// 保存索引到磁盘
    ///
    /// 使用原子写入（临时文件 + 重命名）确保数据完整性。
    /// 直接序列化 WallpaperIndex，支持多语言。
    pub async fn save_index(&self, index: &WallpaperIndex) -> Result<()> {
        // 序列化为 JSON（人类可读格式，便于调试）
        let json = serde_json::to_string_pretty(index).context("Failed to serialize index")?;

        // 确保目录存在
        fs::create_dir_all(&self.directory)
            .await
            .context("Failed to create directory")?;

        // 原子写入
        let temp_path = self.index_path().with_extension("tmp");
        fs::write(&temp_path, json)
            .await
            .context("Failed to write temporary index file")?;

        fs::rename(&temp_path, self.index_path())
            .await
            .context("Failed to rename index file")?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().await;
            *cache = Some(index.clone());
        }

        Ok(())
    }

    /// 批量添加或更新壁纸（性能优化）
    ///
    /// 一次性写入多个壁纸，比多次调用 `upsert_wallpaper` 效率高。
    ///
    /// # Arguments
    /// * `wallpapers` - 要添加或更新的壁纸列表
    /// * `language` - 语言代码（如 "zh-CN", "en-US"）
    pub async fn upsert_wallpapers(
        &self,
        wallpapers: Vec<LocalWallpaper>,
        language: &str,
    ) -> Result<()> {
        if wallpapers.is_empty() {
            return Ok(());
        }

        let mut index = self.load_index().await?;
        index.upsert_wallpapers_for_language(language, wallpapers);
        self.save_index(&index).await
    }

    /// 批量删除壁纸（性能优化）
    ///
    /// 从所有语言中删除指定 end_date 的壁纸。
    ///
    /// # Arguments
    /// * `end_dates` - 要删除的壁纸的结束日期列表
    pub async fn remove_wallpapers(&self, end_dates: &[String]) -> Result<()> {
        if end_dates.is_empty() {
            return Ok(());
        }

        let mut index = self.load_index().await?;
        // 从所有语言中删除这些 end_date
        for lang_wallpapers in index.wallpapers_by_language.values_mut() {
            for end_date in end_dates {
                lang_wallpapers.remove(end_date);
            }
        }
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 获取所有壁纸（排序）
    ///
    /// 返回按日期降序排列的壁纸列表（最新的在前）。
    ///
    /// # Arguments
    /// * `language` - 语言代码（如 "zh-CN", "en-US"）
    pub async fn get_all_wallpapers(&self, language: &str) -> Result<Vec<LocalWallpaper>> {
        let index = self.load_index().await?;
        let available_languages: Vec<String> =
            index.wallpapers_by_language.keys().cloned().collect();
        let wallpapers = index.get_wallpapers_for_language(language);

        log::debug!(
            "获取壁纸列表，语言: {}, 找到 {} 张壁纸，可用语言: {:?}",
            language,
            wallpapers.len(),
            available_languages
        );

        Ok(wallpapers)
    }

    /// 获取所有语言的唯一壁纸（用于清理操作）
    pub async fn get_all_wallpapers_unique(&self) -> Result<Vec<LocalWallpaper>> {
        let index = self.load_index().await?;
        Ok(index.get_all_wallpapers_unique())
    }

    /// 清理缓存
    ///
    /// 清除内存中的缓存，下次访问时会重新从磁盘加载。
    #[allow(dead_code)]
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.lock().await;
        *cache = None;
    }

    /// 强制从磁盘重新加载
    ///
    /// 清除缓存并重新从磁盘加载索引。
    #[allow(dead_code)]
    pub async fn reload(&self) -> Result<WallpaperIndex> {
        self.clear_cache().await;
        self.load_index().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_index_manager_new_index() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_new_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());
        let index = manager.load_index().await.unwrap();

        assert_eq!(index.version, WallpaperIndex::VERSION);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_upsert_and_get() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_upsert_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        let wallpaper = LocalWallpaper {
            id: "test123".to_string(),
            title: "Test Wallpaper".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/test.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.TestWallpaper".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper.clone()], "zh-CN")
            .await
            .unwrap();

        let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
        let retrieved = all.into_iter().find(|w| w.end_date == "20240102");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Wallpaper");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_batch_operations() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_batch_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        let wallpapers = vec![
            LocalWallpaper {
                id: "test1".to_string(),
                title: "Wallpaper 1".to_string(),
                copyright: "Copyright 1".to_string(),
                copyright_link: "https://example.com/1".to_string(),
                start_date: "20240101".to_string(),
                end_date: "20240102".to_string(),
                file_path: "/tmp/test1.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper1".to_string(),
            },
            LocalWallpaper {
                id: "test2".to_string(),
                title: "Wallpaper 2".to_string(),
                copyright: "Copyright 2".to_string(),
                copyright_link: "https://example.com/2".to_string(),
                start_date: "20240102".to_string(),
                end_date: "20240103".to_string(),
                file_path: "/tmp/test2.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper2".to_string(),
            },
        ];

        manager
            .upsert_wallpapers(wallpapers, "zh-CN")
            .await
            .unwrap();

        let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert_eq!(all.len(), 2);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_persistence() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_persist_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let wallpaper = LocalWallpaper {
            id: "persist_test".to_string(),
            title: "Persist Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/persist.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.PersistTest".to_string(),
        };

        // 第一个管理器实例
        {
            let manager = IndexManager::new(temp_dir.clone());
            manager
                .upsert_wallpapers(vec![wallpaper.clone()], "zh-CN")
                .await
                .unwrap();
        }

        // 第二个管理器实例（模拟程序重启）
        {
            let manager = IndexManager::new(temp_dir.clone());
            let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
            let retrieved = all.into_iter().find(|w| w.end_date == "20240102");
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().title, "Persist Test");
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_version_mismatch() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_version_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let index_path = temp_dir.join("index.json");

        // 创建一个旧版本的索引文件（v2）
        let old_index = r#"{
  "version": 2,
  "last_updated": "2024-01-01T00:00:00Z",
  "wallpapers_by_language": {
    "zh-CN": {
      "20240101": {
        "id": "test",
        "title": "Old Version",
        "copyright": "Test",
        "copyright_link": "https://example.com",
        "start_date": "20240101",
        "end_date": "20240102",
        "file_path": "/tmp/test.jpg",
        "download_time": "2024-01-01T00:00:00Z",
        "urlbase": ""
      }
    }
  }
}"#;
        fs::write(&index_path, old_index).await.unwrap();

        // 尝试加载旧版本索引
        let manager = IndexManager::new(temp_dir.clone());
        let index = manager.load_index().await.unwrap();

        // 应该返回空索引（版本不匹配）
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.wallpapers_by_language.is_empty());

        // 检查备份文件是否创建
        let backup_path = index_path.with_extension("backup");
        assert!(backup_path.exists(), "备份文件应该被创建");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_end_date_as_key() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_key_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 创建多个壁纸，使用不同的 end_date
        let wallpapers = vec![
            LocalWallpaper {
                id: "test1".to_string(),
                title: "Wallpaper 1".to_string(),
                copyright: "Copyright 1".to_string(),
                copyright_link: "https://example.com/1".to_string(),
                start_date: "20240101".to_string(),
                end_date: "20240102".to_string(),
                file_path: "/tmp/20240102.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper1".to_string(),
            },
            LocalWallpaper {
                id: "test2".to_string(),
                title: "Wallpaper 2".to_string(),
                copyright: "Copyright 2".to_string(),
                copyright_link: "https://example.com/2".to_string(),
                start_date: "20240102".to_string(),
                end_date: "20240103".to_string(),
                file_path: "/tmp/20240103.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper2".to_string(),
            },
        ];

        manager
            .upsert_wallpapers(wallpapers, "zh-CN")
            .await
            .unwrap();

        // 验证索引文件使用 end_date 作为 key
        let index = manager.load_index().await.unwrap();
        let zh_cn_wallpapers = index.wallpapers_by_language.get("zh-CN").unwrap();

        // 应该能用 end_date 作为 key 找到壁纸
        assert!(zh_cn_wallpapers.contains_key("20240102"));
        assert!(zh_cn_wallpapers.contains_key("20240103"));

        // 不应该能用 start_date 作为 key 找到壁纸
        // start_date "20240101" 不是 key（第一个壁纸的 start_date）
        assert!(!zh_cn_wallpapers.contains_key("20240101"));

        // 验证所有 key 都是 end_date，而不是 start_date
        // 遍历所有壁纸，确保 key 等于 end_date
        for (key, wallpaper) in zh_cn_wallpapers.iter() {
            assert_eq!(key, &wallpaper.end_date, "索引 key 必须等于 end_date");
            assert_ne!(key, &wallpaper.start_date, "索引 key 不应该等于 start_date");
        }

        // 验证文件路径与 end_date 一致
        let wp1 = zh_cn_wallpapers.get("20240102").unwrap();
        assert!(wp1.file_path.contains("20240102"));
        assert_eq!(wp1.end_date, "20240102");
        assert_eq!(wp1.start_date, "20240101"); // start_date 是数据的一部分，但不会作为 key

        let wp2 = zh_cn_wallpapers.get("20240103").unwrap();
        assert!(wp2.file_path.contains("20240103"));
        assert_eq!(wp2.end_date, "20240103");
        assert_eq!(wp2.start_date, "20240102"); // start_date 是数据的一部分，但不会作为 key

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_multilanguage() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_multilang_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 添加中文壁纸
        let wallpaper_zh = LocalWallpaper {
            id: "test_zh".to_string(),
            title: "中文壁纸".to_string(),
            copyright: "版权信息".to_string(),
            copyright_link: "https://example.com/zh".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.Wallpaper_ZH-CN".to_string(),
        };

        // 添加英文壁纸
        let wallpaper_en = LocalWallpaper {
            id: "test_en".to_string(),
            title: "English Wallpaper".to_string(),
            copyright: "Copyright Info".to_string(),
            copyright_link: "https://example.com/en".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.Wallpaper_EN-US".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper_zh], "zh-CN")
            .await
            .unwrap();

        manager
            .upsert_wallpapers(vec![wallpaper_en], "en-US")
            .await
            .unwrap();

        // 验证多语言存储
        let index = manager.load_index().await.unwrap();
        assert_eq!(index.wallpapers_by_language.len(), 2);
        assert!(index.wallpapers_by_language.contains_key("zh-CN"));
        assert!(index.wallpapers_by_language.contains_key("en-US"));

        // 验证每个语言都有正确的壁纸
        let zh_wallpapers = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert_eq!(zh_wallpapers.len(), 1);
        assert_eq!(zh_wallpapers[0].title, "中文壁纸");

        let en_wallpapers = manager.get_all_wallpapers("en-US").await.unwrap();
        assert_eq!(en_wallpapers.len(), 1);
        assert_eq!(en_wallpapers[0].title, "English Wallpaper");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_remove_wallpapers() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_remove_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 添加多个壁纸
        let wallpapers = vec![
            LocalWallpaper {
                id: "test1".to_string(),
                title: "Wallpaper 1".to_string(),
                copyright: "Copyright 1".to_string(),
                copyright_link: "https://example.com/1".to_string(),
                start_date: "20240101".to_string(),
                end_date: "20240102".to_string(),
                file_path: "/tmp/20240102.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper1".to_string(),
            },
            LocalWallpaper {
                id: "test2".to_string(),
                title: "Wallpaper 2".to_string(),
                copyright: "Copyright 2".to_string(),
                copyright_link: "https://example.com/2".to_string(),
                start_date: "20240102".to_string(),
                end_date: "20240103".to_string(),
                file_path: "/tmp/20240103.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper2".to_string(),
            },
            LocalWallpaper {
                id: "test3".to_string(),
                title: "Wallpaper 3".to_string(),
                copyright: "Copyright 3".to_string(),
                copyright_link: "https://example.com/3".to_string(),
                start_date: "20240103".to_string(),
                end_date: "20240104".to_string(),
                file_path: "/tmp/20240104.jpg".to_string(),
                download_time: Utc::now(),
                urlbase: "/th?id=OHR.Wallpaper3".to_string(),
            },
        ];

        manager
            .upsert_wallpapers(wallpapers, "zh-CN")
            .await
            .unwrap();

        // 验证添加成功
        let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert_eq!(all.len(), 3);

        // 删除一个壁纸（使用 end_date）
        manager
            .remove_wallpapers(&["20240103".to_string()])
            .await
            .unwrap();

        // 验证删除成功
        let all_after = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert_eq!(all_after.len(), 2);

        // 验证正确的壁纸被删除
        let end_dates: Vec<String> = all_after.iter().map(|w| w.end_date.clone()).collect();
        assert!(end_dates.contains(&"20240102".to_string()));
        assert!(end_dates.contains(&"20240104".to_string()));
        assert!(!end_dates.contains(&"20240103".to_string()));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_cache() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_cache_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        let wallpaper = LocalWallpaper {
            id: "cache_test".to_string(),
            title: "Cache Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.CacheTest".to_string(),
        };

        // 第一次加载（应该从磁盘）
        manager
            .upsert_wallpapers(vec![wallpaper.clone()], "zh-CN")
            .await
            .unwrap();

        // 第二次加载（应该使用缓存）
        let index1 = manager.load_index().await.unwrap();
        let index2 = manager.load_index().await.unwrap();

        // 两次加载应该返回相同的数据
        assert_eq!(index1.wallpapers_by_language.len(), index2.wallpapers_by_language.len());

        // 清理缓存并重新加载
        manager.clear_cache().await;
        let index3 = manager.load_index().await.unwrap();

        // 应该从磁盘重新加载，数据应该一致
        assert_eq!(index1.wallpapers_by_language.len(), index3.wallpapers_by_language.len());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_update_existing() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_update_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 添加初始壁纸
        let wallpaper1 = LocalWallpaper {
            id: "test".to_string(),
            title: "Original Title".to_string(),
            copyright: "Original Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.Test".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper1], "zh-CN")
            .await
            .unwrap();

        // 更新同一 end_date 的壁纸（应该覆盖）
        let wallpaper2 = LocalWallpaper {
            id: "test".to_string(),
            title: "Updated Title".to_string(),
            copyright: "Updated Copyright".to_string(),
            copyright_link: "https://example.com/updated".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(), // 相同的 end_date
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.TestUpdated".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper2], "zh-CN")
            .await
            .unwrap();

        // 验证更新成功
        let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].title, "Updated Title");
        assert_eq!(all[0].copyright, "Updated Copyright");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_get_all_wallpapers_unique() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_unique_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 添加相同 end_date 但不同语言的壁纸
        let wallpaper_zh = LocalWallpaper {
            id: "test_zh".to_string(),
            title: "中文".to_string(),
            copyright: "版权".to_string(),
            copyright_link: "https://example.com/zh".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.Wallpaper_ZH-CN".to_string(),
        };

        let wallpaper_en = LocalWallpaper {
            id: "test_en".to_string(),
            title: "English".to_string(),
            copyright: "Copyright".to_string(),
            copyright_link: "https://example.com/en".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(), // 相同的 end_date
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.Wallpaper_EN-US".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper_zh], "zh-CN")
            .await
            .unwrap();

        manager
            .upsert_wallpapers(vec![wallpaper_en], "en-US")
            .await
            .unwrap();

        // 获取唯一壁纸列表
        let unique_wallpapers = manager.get_all_wallpapers_unique().await.unwrap();

        // 应该只有一张壁纸（因为 end_date 相同）
        assert_eq!(unique_wallpapers.len(), 1);
        assert_eq!(unique_wallpapers[0].end_date, "20240102");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_empty_operations() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_empty_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 空列表操作应该成功
        manager.upsert_wallpapers(vec![], "zh-CN").await.unwrap();
        manager.remove_wallpapers(&[]).await.unwrap();

        // 获取空列表应该返回空
        let all = manager.get_all_wallpapers("zh-CN").await.unwrap();
        assert!(all.is_empty());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_atomic_write() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_atomic_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());
        let index_path = manager.index_path();

        let wallpaper = LocalWallpaper {
            id: "atomic_test".to_string(),
            title: "Atomic Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.AtomicTest".to_string(),
        };

        // 保存索引
        manager
            .upsert_wallpapers(vec![wallpaper], "zh-CN")
            .await
            .unwrap();

        // 验证临时文件不存在（应该已经被重命名）
        let temp_path = index_path.with_extension("tmp");
        assert!(!temp_path.exists(), "临时文件应该已被删除");

        // 验证索引文件存在
        assert!(index_path.exists(), "索引文件应该存在");

        // 验证可以正确加载
        let index = manager.load_index().await.unwrap();
        assert_eq!(index.wallpapers_by_language.len(), 1);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_json_serialization() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_json_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());
        let index_path = manager.index_path();

        // 创建壁纸并保存
        let wallpaper = LocalWallpaper {
            id: "json_test".to_string(),
            title: "JSON Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            start_date: "20240101".to_string(),
            end_date: "20240102".to_string(),
            file_path: "/tmp/20240102.jpg".to_string(),
            download_time: Utc::now(),
            urlbase: "/th?id=OHR.JsonTest".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper], "zh-CN")
            .await
            .unwrap();

        // 读取 JSON 文件内容
        let json_content = fs::read_to_string(&index_path).await.unwrap();

        // 验证 JSON 内容包含 end_date 作为 key
        assert!(
            json_content.contains("\"20240102\""),
            "JSON 应该包含 end_date 作为 key"
        );
        assert!(
            json_content.contains("\"end_date\": \"20240102\""),
            "JSON 应该包含 end_date 字段"
        );

        // 验证 JSON 内容不包含 start_date 作为 key（在 wallpapers_by_language 中）
        // 注意：这里要检查的是内层 key，不是字段名
        // JSON 格式应该是：{"zh-CN": {"20240102": {...}}}
        // 所以 "20240102" 应该是 key，而不是 "20240101"
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let zh_cn_map = parsed["wallpapers_by_language"]["zh-CN"].as_object().unwrap();

        // 验证 key 是 end_date
        assert!(zh_cn_map.contains_key("20240102"), "JSON key 应该是 end_date");
        assert!(!zh_cn_map.contains_key("20240101"), "JSON key 不应该是 start_date");

        // 验证版本号
        assert_eq!(parsed["version"], 3, "版本号应该是 3");

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_invalid_json_handling() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_invalid_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let index_path = temp_dir.join("index.json");

        // 创建一个无效的 JSON 文件
        fs::write(&index_path, "invalid json content").await.unwrap();

        // 尝试加载（应该返回空索引，因为解析失败）
        let manager = IndexManager::new(temp_dir.clone());
        let index = manager.load_index().await.unwrap();

        // 应该返回空索引（默认值）
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.wallpapers_by_language.is_empty());

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_concurrent_access() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_concurrent_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        // 创建多个壁纸
        let wallpapers = (1..=5)
            .map(|i| LocalWallpaper {
                id: format!("test{}", i),
                title: format!("Wallpaper {}", i),
                copyright: format!("Copyright {}", i),
                copyright_link: format!("https://example.com/{}", i),
                start_date: format!("202401{:02}", i),
                end_date: format!("202401{:02}", i + 1),
                file_path: format!("/tmp/202401{:02}.jpg", i + 1),
                download_time: Utc::now(),
                urlbase: format!("/th?id=OHR.Wallpaper{}", i),
            })
            .collect();

        manager
            .upsert_wallpapers(wallpapers, "zh-CN")
            .await
            .unwrap();

        // 并发读取
        let manager1 = IndexManager::new(temp_dir.clone());
        let manager2 = IndexManager::new(temp_dir.clone());
        let manager3 = IndexManager::new(temp_dir.clone());

        let (r1, r2, r3) = tokio::join!(
            manager1.get_all_wallpapers("zh-CN"),
            manager2.get_all_wallpapers("zh-CN"),
            manager3.get_all_wallpapers("zh-CN")
        );

        let all1 = r1.unwrap();
        let all2 = r2.unwrap();
        let all3 = r3.unwrap();

        // 所有读取应该返回相同的结果
        assert_eq!(all1.len(), 5);
        assert_eq!(all2.len(), 5);
        assert_eq!(all3.len(), 5);

        // 验证排序（应该按 end_date 降序）
        for i in 0..4 {
            assert!(
                all1[i].end_date >= all1[i + 1].end_date,
                "壁纸应该按 end_date 降序排列"
            );
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
