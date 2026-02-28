use crate::models::{LocalWallpaper, WallpaperIndex};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

/// 索引文件名
const INDEX_FILE: &str = "index.json";

/// 索引最大条目数限制（基于唯一日期数）
///
/// 限制为 2000 个唯一日期，相当于约 5.5 年的历史记录。
///
/// 文件大小估算：
/// - 单语言：约 400KB（格式化后）
/// - 双语言：约 800KB（格式化后）
/// - 三语言：约 1.2MB（格式化后）
///
/// 性能考虑：
/// - serde_json 解析 1-2MB JSON 文件通常 < 50ms
/// - 使用内存缓存机制，大部分情况下不需要从磁盘加载
/// - IndexMap 在内存中的占用略大于 JSON，但在可接受范围内
const MAX_INDEX_COUNT: usize = 2000;

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

    /// 从任意路径加载 index.json（只读，不走缓存，不回写迁移）
    ///
    /// 用于导入场景：读取外部壁纸目录的 index.json 并解析为 WallpaperIndex。
    /// 支持 v4 和 v5 格式（通过 serde alias 自动兼容）。
    /// 不支持的版本返回错误（与 `load_from_disk` 的静默降级不同，导入需要明确失败）。
    pub async fn load_external_index(path: &Path) -> Result<WallpaperIndex> {
        let index_path = path.join(INDEX_FILE);

        let contents = match fs::read_to_string(&index_path).await {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                anyhow::bail!("Index file not found: {}", index_path.display());
            }
            Err(e) => {
                return Err(anyhow::Error::new(e).context(format!(
                    "Failed to read index file: {}",
                    index_path.display()
                )));
            }
        };

        let json_value: serde_json::Value = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse JSON: {}", index_path.display()))?;

        let file_version = json_value
            .get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if file_version != WallpaperIndex::VERSION
            && file_version != WallpaperIndex::MIGRATE_FROM_VERSION
        {
            anyhow::bail!(
                "Unsupported index version: v{} (supported: v{}, v{})",
                file_version,
                WallpaperIndex::MIGRATE_FROM_VERSION,
                WallpaperIndex::VERSION
            );
        }

        let mut index: WallpaperIndex = serde_json::from_value(json_value).with_context(|| {
            format!("Failed to deserialize index file: {}", index_path.display())
        })?;

        index.version = WallpaperIndex::VERSION;
        index.sort_all();

        log::info!(
            "成功加载外部索引文件，包含 {} 个 mkt，共 {} 张壁纸，路径: {}",
            index.mkt.len(),
            index.mkt.values().map(|m| m.len()).sum::<usize>(),
            index_path.display()
        );

        Ok(index)
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
                    "使用缓存的索引，包含 {} 个 mkt，路径: {}",
                    index.mkt.len(),
                    index_path.display()
                );
                return Ok(index.clone());
            }
        }

        // 从磁盘加载
        log::debug!("从磁盘加载索引，路径: {}", index_path.display());
        let index = match self.load_from_disk().await {
            Ok(index) => {
                let mkt_count = index.mkt.len();
                let total_wallpapers: usize = index.mkt.values().map(|m| m.len()).sum();
                log::info!(
                    "成功加载索引文件，包含 {} 个 mkt，共 {} 张壁纸，路径: {}",
                    mkt_count,
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

        // 先解析为 JSON Value，检查版本号
        let json_value: serde_json::Value = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse JSON: {}", path.display()))?;

        // 检查版本号
        let file_version = json_value
            .get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        if file_version == WallpaperIndex::VERSION {
            // 当前版本，直接反序列化
            let mut index: WallpaperIndex = serde_json::from_value(json_value)
                .with_context(|| format!("Failed to deserialize index file: {}", path.display()))?;
            index.sort_all();
            log::debug!("索引文件加载成功，版本: v{}", index.version);
            return Ok(index);
        }

        if file_version == WallpaperIndex::MIGRATE_FROM_VERSION {
            // v4 → v5 迁移：wallpapers_by_language → mkt
            log::info!(
                "检测到 v{} 索引，开始迁移到 v{}，路径: {}",
                file_version,
                WallpaperIndex::VERSION,
                path.display()
            );

            // 1. 备份旧文件
            let backup_path = path.with_extension(format!("json.v{file_version}.bak"));
            fs::copy(&path, &backup_path).await.with_context(|| {
                format!(
                    "Failed to backup index file: {} → {}",
                    path.display(),
                    backup_path.display()
                )
            })?;
            log::info!("已备份旧索引文件: {}", backup_path.display());

            // 2. 反序列化（serde alias 自动兼容 wallpapers_by_language → mkt）
            let mut index: WallpaperIndex =
                serde_json::from_value(json_value).with_context(|| {
                    format!(
                        "Failed to deserialize v{file_version} index: {}",
                        path.display()
                    )
                })?;

            // 3. 升级版本号
            index.version = WallpaperIndex::VERSION;
            index.sort_all();

            // 4. 回写新格式
            self.save_index(&index).await?;
            log::info!(
                "索引迁移完成 v{} → v{}，路径: {}",
                file_version,
                WallpaperIndex::VERSION,
                path.display()
            );
            return Ok(index);
        }

        // 不支持的旧版本，返回空索引
        log::warn!(
            "索引版本不支持 (当前: v{}, 文件: v{}), 返回空索引，路径: {}",
            WallpaperIndex::VERSION,
            file_version,
            path.display()
        );
        Ok(WallpaperIndex::default())
    }

    /// 保存索引到磁盘
    ///
    /// 使用原子写入（临时文件 + 重命名）确保数据完整性。
    /// 直接序列化 WallpaperIndex，支持多语言。
    /// 使用紧凑格式以节省存储空间。
    pub async fn save_index(&self, index: &WallpaperIndex) -> Result<()> {
        // 序列化为 JSON（紧凑格式，节省存储空间）
        let json = serde_json::to_string(index).context("Failed to serialize index")?;

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
    /// 如果索引数据超过最大限制（默认 2000 个唯一日期），会自动清理最旧的条目。
    /// 返回实际新增的条目数（不含覆盖已存在的条目）。
    ///
    /// # Arguments
    /// * `wallpapers` - 要添加或更新的壁纸列表
    /// * `language` - 语言代码（如 "zh-CN", "en-US")
    pub async fn upsert_wallpapers(
        &self,
        wallpapers: Vec<LocalWallpaper>,
        language: &str,
    ) -> Result<usize> {
        if wallpapers.is_empty() {
            return Ok(0);
        }

        let mut index = self.load_index().await?;
        let new_count = index.upsert_wallpapers_for_mkt(language, wallpapers);

        // 限制索引数量，防止 JSON 文件过大
        index.limit_index_size(MAX_INDEX_COUNT);

        self.save_index(&index).await?;
        Ok(new_count)
    }

    /// 获取所有壁纸（排序）
    ///
    /// 返回按日期降序排列的壁纸列表（最新的在前）。
    ///
    /// # Arguments
    /// * `language` - 语言代码（如 "zh-CN", "en-US"）
    pub async fn get_all_wallpapers(&self, language: &str) -> Result<Vec<LocalWallpaper>> {
        let index = self.load_index().await?;
        let available_mkts: Vec<String> = index.mkt.keys().cloned().collect();
        let wallpapers = index.get_wallpapers_for_mkt(language);

        log::debug!(
            "获取壁纸列表，mkt: {}, 找到 {} 张壁纸，可用 mkt: {:?}",
            language,
            wallpapers.len(),
            available_mkts
        );

        Ok(wallpapers)
    }

    /// 获取 index.json 中所有可用的 mkt key
    ///
    /// 用于 fallback 场景：当 effective_mkt 对应的壁纸列表为空时，
    /// 可从可用 key 中选择最匹配的 mkt。
    pub async fn get_available_mkt_keys(&self) -> Result<Vec<String>> {
        let index = self.load_index().await?;
        let mut keys: Vec<String> = index.mkt.keys().cloned().collect();
        // 排序确保返回顺序稳定，避免上层 fallback 使用时出现随机行为。
        keys.sort();
        log::debug!("index.json 可用 mkt keys: {:?}", keys);
        Ok(keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            title: "Test Wallpaper".to_string(),
            copyright: "Test Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
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
                title: "Wallpaper 1".to_string(),
                copyright: "Copyright 1".to_string(),
                copyright_link: "https://example.com/1".to_string(),
                end_date: "20240102".to_string(),
                urlbase: "/th?id=OHR.Wallpaper1".to_string(),
            },
            LocalWallpaper {
                title: "Wallpaper 2".to_string(),
                copyright: "Copyright 2".to_string(),
                copyright_link: "https://example.com/2".to_string(),
                end_date: "20240103".to_string(),
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
            title: "Persist Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
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
                title: "Wallpaper 1".to_string(),
                copyright: "Copyright 1".to_string(),
                copyright_link: "https://example.com/1".to_string(),
                end_date: "20240102".to_string(),
                urlbase: "/th?id=OHR.Wallpaper1".to_string(),
            },
            LocalWallpaper {
                title: "Wallpaper 2".to_string(),
                copyright: "Copyright 2".to_string(),
                copyright_link: "https://example.com/2".to_string(),
                end_date: "20240103".to_string(),
                urlbase: "/th?id=OHR.Wallpaper2".to_string(),
            },
        ];

        manager
            .upsert_wallpapers(wallpapers, "zh-CN")
            .await
            .unwrap();

        // 验证索引文件使用 end_date 作为 key
        let index = manager.load_index().await.unwrap();
        let zh_cn_wallpapers = index.mkt.get("zh-CN").unwrap();

        // 应该能用 end_date 作为 key 找到壁纸
        assert!(zh_cn_wallpapers.contains_key("20240102"));
        assert!(zh_cn_wallpapers.contains_key("20240103"));

        // 验证所有 key 都是 end_date
        // 遍历所有壁纸，确保 key 等于 end_date
        for (key, wallpaper) in zh_cn_wallpapers.iter() {
            assert_eq!(key, &wallpaper.end_date, "索引 key 必须等于 end_date");
        }

        // 验证 end_date
        let wp1 = zh_cn_wallpapers.get("20240102").unwrap();
        assert_eq!(wp1.end_date, "20240102");

        let wp2 = zh_cn_wallpapers.get("20240103").unwrap();
        assert_eq!(wp2.end_date, "20240103");

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
            title: "中文壁纸".to_string(),
            copyright: "版权信息".to_string(),
            copyright_link: "https://example.com/zh".to_string(),
            end_date: "20240102".to_string(),
            urlbase: "/th?id=OHR.Wallpaper_ZH-CN".to_string(),
        };

        // 添加英文壁纸
        let wallpaper_en = LocalWallpaper {
            title: "English Wallpaper".to_string(),
            copyright: "Copyright Info".to_string(),
            copyright_link: "https://example.com/en".to_string(),
            end_date: "20240102".to_string(),
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
        assert_eq!(index.mkt.len(), 2);
        assert!(index.mkt.contains_key("zh-CN"));
        assert!(index.mkt.contains_key("en-US"));

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
    async fn test_index_manager_cache() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_cache_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        let wallpaper = LocalWallpaper {
            title: "Cache Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
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
        assert_eq!(index1.mkt.len(), index2.mkt.len());

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
            title: "Original Title".to_string(),
            copyright: "Original Copyright".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
            urlbase: "/th?id=OHR.Test".to_string(),
        };

        manager
            .upsert_wallpapers(vec![wallpaper1], "zh-CN")
            .await
            .unwrap();

        // 更新同一 end_date 的壁纸（应该覆盖）
        let wallpaper2 = LocalWallpaper {
            title: "Updated Title".to_string(),
            copyright: "Updated Copyright".to_string(),
            copyright_link: "https://example.com/updated".to_string(),
            end_date: "20240102".to_string(), // 相同的 end_date
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
            title: "Atomic Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
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
        assert_eq!(index.mkt.len(), 1);

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
            title: "JSON Test".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
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
        // 验证使用短字段名格式
        assert!(
            json_content.contains("\"d\":\"20240102\""),
            "JSON 应该使用短字段名 d 表示 end_date"
        );
        assert!(
            json_content.contains("\"t\""),
            "JSON 应该使用短字段名 t 表示 title"
        );
        assert!(
            json_content.contains("\"c\""),
            "JSON 应该使用短字段名 c 表示 copyright"
        );

        // 验证 JSON 内容是紧凑格式（不是格式化）
        assert!(
            !json_content.contains("\n  "),
            "JSON 应该是紧凑格式，不应该包含缩进"
        );

        // 验证 JSON 内容使用 end_date 作为 key（在 mkt 中）
        // 注意：这里要检查的是内层 key，不是字段名
        // JSON 格式应该是：{"zh-CN": {"20240102": {...}}}
        // 所以 "20240102" 应该是 key
        let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let zh_cn_map = parsed["mkt"]["zh-CN"].as_object().unwrap();

        // 验证 key 是 end_date
        assert!(
            zh_cn_map.contains_key("20240102"),
            "JSON key 应该是 end_date"
        );

        // 验证版本号
        assert_eq!(
            parsed["version"],
            WallpaperIndex::VERSION,
            "版本号应该是 v{}",
            WallpaperIndex::VERSION
        );

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_index_manager_migrate_v4_to_v5() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_migrate_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let index_path = temp_dir.join("index.json");
        let backup_path = temp_dir.join("index.json.v4.bak");

        // 写入 v4 格式的 index.json（使用 wallpapers_by_language 字段名）
        // 注意：LocalWallpaper 的 serde 短字段名是 t/c/l/d/u
        let v4_json = r#"{"version":4,"last_updated":"2025-02-14T00:00:00Z","wallpapers_by_language":{"zh-CN":{"20250214":{"t":"Test","c":"Copyright","l":"https://example.com","d":"20250214","u":"/th?id=OHR.Test"}}}}"#;
        fs::write(&index_path, v4_json).await.unwrap();

        // 加载索引 —— 应触发 v4 → v5 迁移
        let manager = IndexManager::new(temp_dir.clone());
        let index = manager.load_index().await.unwrap();

        // 验证数据被正确加载
        assert_eq!(index.mkt.len(), 1);
        assert!(index.mkt.contains_key("zh-CN"));
        let zh_wallpapers = index.mkt.get("zh-CN").unwrap();
        assert!(zh_wallpapers.contains_key("20250214"));

        // 验证版本号已升级到 v5
        assert_eq!(
            index.version,
            WallpaperIndex::VERSION,
            "迁移后版本号应为 v{}",
            WallpaperIndex::VERSION
        );

        // 验证备份文件已创建
        assert!(
            backup_path.exists(),
            "应创建备份文件: {}",
            backup_path.display()
        );
        let backup_content = fs::read_to_string(&backup_path).await.unwrap();
        assert!(
            backup_content.contains("wallpapers_by_language"),
            "备份文件应保留原始 v4 内容"
        );
        assert!(
            backup_content.contains("\"version\":4"),
            "备份文件应保留 v4 版本号"
        );

        // 验证磁盘上的 JSON 已迁移为新字段名 "mkt"
        let migrated_json = fs::read_to_string(&index_path).await.unwrap();
        assert!(
            migrated_json.contains("\"mkt\""),
            "迁移后应使用 mkt 字段名，实际内容: {}",
            migrated_json
        );
        assert!(
            !migrated_json.contains("wallpapers_by_language"),
            "迁移后不应再包含 wallpapers_by_language，实际内容: {}",
            migrated_json
        );

        // 验证再次加载不会重复迁移（直接走 v5 分支）
        let manager2 = IndexManager::new(temp_dir.clone());
        let index2 = manager2.load_index().await.unwrap();
        assert_eq!(index2.version, WallpaperIndex::VERSION);
        assert_eq!(index2.mkt.len(), 1);

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
        fs::write(&index_path, "invalid json content")
            .await
            .unwrap();

        // 尝试加载（应该返回空索引，因为解析失败）
        let manager = IndexManager::new(temp_dir.clone());
        let index = manager.load_index().await.unwrap();

        // 应该返回空索引（默认值）
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.mkt.is_empty());

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
                title: format!("Wallpaper {}", i),
                copyright: format!("Copyright {}", i),
                copyright_link: format!("https://example.com/{}", i),
                end_date: format!("202401{:02}", i + 1),
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

    #[tokio::test]
    async fn test_get_available_mkt_keys_returns_sorted_keys() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_index_keys_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let manager = IndexManager::new(temp_dir.clone());

        let wallpaper = LocalWallpaper {
            title: "Key Order".to_string(),
            copyright: "Test".to_string(),
            copyright_link: "https://example.com".to_string(),
            end_date: "20240102".to_string(),
            urlbase: "/th?id=OHR.KeyOrder".to_string(),
        };

        // 有意按非字典序写入语言 key，验证返回顺序稳定。
        manager
            .upsert_wallpapers(vec![wallpaper.clone()], "zh-CN")
            .await
            .unwrap();
        manager
            .upsert_wallpapers(vec![wallpaper.clone()], "en-US")
            .await
            .unwrap();
        manager
            .upsert_wallpapers(vec![wallpaper], "ja-JP")
            .await
            .unwrap();

        let keys = manager.get_available_mkt_keys().await.unwrap();
        assert_eq!(keys, vec!["en-US", "ja-JP", "zh-CN"]);

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_v5() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_v5_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let v5_json = r#"{"version":5,"last_updated":"2025-02-14T00:00:00Z","mkt":{"zh-CN":{"20250214":{"t":"Test","c":"Copyright","l":"https://example.com","d":"20250214","u":"/th?id=OHR.Test"}}}}"#;
        fs::write(temp_dir.join("index.json"), v5_json)
            .await
            .unwrap();

        let index = IndexManager::load_external_index(&temp_dir).await.unwrap();
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert_eq!(index.mkt.len(), 1);
        assert!(index.mkt.contains_key("zh-CN"));
        let zh = index.mkt.get("zh-CN").unwrap();
        assert!(zh.contains_key("20250214"));
        assert_eq!(zh.get("20250214").unwrap().title, "Test");

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_v4() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_v4_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let v4_json = r#"{"version":4,"last_updated":"2025-02-14T00:00:00Z","wallpapers_by_language":{"en-US":{"20250213":{"t":"Hello","c":"(c)","l":"https://example.com","d":"20250213","u":"/th?id=OHR.Hello"}}}}"#;
        fs::write(temp_dir.join("index.json"), v4_json)
            .await
            .unwrap();

        let index = IndexManager::load_external_index(&temp_dir).await.unwrap();
        assert_eq!(index.version, WallpaperIndex::VERSION);
        assert!(index.mkt.contains_key("en-US"));

        // Source file should NOT be modified (read-only)
        let contents = fs::read_to_string(temp_dir.join("index.json"))
            .await
            .unwrap();
        assert!(
            contents.contains("wallpapers_by_language"),
            "Source file should remain unchanged"
        );

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_missing_file() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_missing_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let result = IndexManager::load_external_index(&temp_dir).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Index file not found")
        );

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_unsupported_version() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_badver_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let json = r#"{"version":1,"last_updated":"2025-01-01T00:00:00Z","mkt":{}}"#;
        fs::write(temp_dir.join("index.json"), json).await.unwrap();

        let result = IndexManager::load_external_index(&temp_dir).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported index version")
        );

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_invalid_json() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_invalid_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        fs::write(temp_dir.join("index.json"), "not json at all")
            .await
            .unwrap();

        let result = IndexManager::load_external_index(&temp_dir).await;
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_load_external_index_multilang() {
        let unique = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("bw_ext_multi_{unique}"));
        fs::create_dir_all(&temp_dir).await.unwrap();

        let json = r#"{"version":5,"last_updated":"2025-02-14T00:00:00Z","mkt":{"zh-CN":{"20250214":{"t":"中文","c":"c","l":"l","d":"20250214","u":"u"},"20250213":{"t":"中文2","c":"c","l":"l","d":"20250213","u":"u"}},"en-US":{"20250214":{"t":"English","c":"c","l":"l","d":"20250214","u":"u"}}}}"#;
        fs::write(temp_dir.join("index.json"), json).await.unwrap();

        let index = IndexManager::load_external_index(&temp_dir).await.unwrap();
        assert_eq!(index.mkt.len(), 2);

        let zh = index.get_wallpapers_for_mkt("zh-CN");
        assert_eq!(zh.len(), 2);
        assert_eq!(zh[0].end_date, "20250214");

        let en = index.get_wallpapers_for_mkt("en-US");
        assert_eq!(en.len(), 1);

        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
