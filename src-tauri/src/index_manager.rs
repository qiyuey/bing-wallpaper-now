use crate::models::{LocalWallpaper, WallpaperIndex};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;

/// 索引文件名
const INDEX_FILE: &str = "index.msgpack";

/// 内存缓存的索引管理器
///
/// 提供高效的壁纸元数据管理，使用单一 MessagePack 文件存储所有元数据，
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
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some(index) = cache.as_ref() {
                return Ok(index.clone());
            }
        }

        // 从磁盘加载
        let index = self.load_from_disk().await.unwrap_or_else(|e| {
            log::info!(
                "Index file not found or corrupted ({}), will rebuild if needed",
                e
            );
            WallpaperIndex::default()
        });

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(index.clone());
        }

        Ok(index)
    }

    /// 从磁盘加载索引
    async fn load_from_disk(&self) -> Result<WallpaperIndex> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(WallpaperIndex::default());
        }

        let bytes = fs::read(&path).await.context("Failed to read index file")?;

        let index: WallpaperIndex =
            rmp_serde::from_slice(&bytes).context("Failed to deserialize index")?;

        // 版本检查
        if index.version != WallpaperIndex::VERSION {
            log::warn!(
                "Index version mismatch (expected {}, got {}), creating new index",
                WallpaperIndex::VERSION,
                index.version
            );
            return Ok(WallpaperIndex::default());
        }

        Ok(index)
    }

    /// 保存索引到磁盘
    ///
    /// 使用原子写入（临时文件 + 重命名）确保数据完整性。
    pub async fn save_index(&self, index: &WallpaperIndex) -> Result<()> {
        // 序列化为 MessagePack（比 JSON 更紧凑、更快）
        let bytes = rmp_serde::to_vec(index).context("Failed to serialize index")?;

        // 确保目录存在
        fs::create_dir_all(&self.directory)
            .await
            .context("Failed to create directory")?;

        // 原子写入
        let temp_path = self.index_path().with_extension("tmp");
        fs::write(&temp_path, &bytes)
            .await
            .context("Failed to write temporary index file")?;

        fs::rename(&temp_path, self.index_path())
            .await
            .context("Failed to rename index file")?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(index.clone());
        }

        Ok(())
    }

    /// 添加或更新壁纸
    ///
    /// # Arguments
    /// * `wallpaper` - 要添加或更新的壁纸
    #[allow(dead_code)]
    pub async fn upsert_wallpaper(&self, wallpaper: LocalWallpaper) -> Result<()> {
        let mut index = self.load_index().await?;
        index
            .wallpapers
            .insert(wallpaper.start_date.clone(), wallpaper);
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 批量添加或更新壁纸（性能优化）
    ///
    /// 一次性写入多个壁纸，比多次调用 `upsert_wallpaper` 效率高。
    ///
    /// # Arguments
    /// * `wallpapers` - 要添加或更新的壁纸列表
    pub async fn upsert_wallpapers(&self, wallpapers: Vec<LocalWallpaper>) -> Result<()> {
        if wallpapers.is_empty() {
            return Ok(());
        }

        let mut index = self.load_index().await?;
        for wallpaper in wallpapers {
            index
                .wallpapers
                .insert(wallpaper.start_date.clone(), wallpaper);
        }
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 删除壁纸
    ///
    /// # Arguments
    /// * `start_date` - 要删除的壁纸的开始日期
    #[allow(dead_code)]
    pub async fn remove_wallpaper(&self, start_date: &str) -> Result<()> {
        let mut index = self.load_index().await?;
        index.wallpapers.remove(start_date);
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 批量删除壁纸（性能优化）
    ///
    /// # Arguments
    /// * `start_dates` - 要删除的壁纸的开始日期列表
    pub async fn remove_wallpapers(&self, start_dates: &[String]) -> Result<()> {
        if start_dates.is_empty() {
            return Ok(());
        }

        let mut index = self.load_index().await?;
        for start_date in start_dates {
            index.wallpapers.remove(start_date);
        }
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 获取所有壁纸（排序）
    ///
    /// 返回按日期降序排列的壁纸列表（最新的在前）。
    pub async fn get_all_wallpapers(&self) -> Result<Vec<LocalWallpaper>> {
        let index = self.load_index().await?;
        let mut wallpapers: Vec<_> = index.wallpapers.into_values().collect();
        // 按 end_date 降序排序（最新的在前）
        wallpapers.sort_by(|a, b| b.end_date.cmp(&a.end_date));
        Ok(wallpapers)
    }

    /// 获取单个壁纸
    ///
    /// # Arguments
    /// * `start_date` - 壁纸的开始日期
    #[allow(dead_code)]
    pub async fn get_wallpaper(&self, start_date: &str) -> Result<Option<LocalWallpaper>> {
        let index = self.load_index().await?;
        Ok(index.wallpapers.get(start_date).cloned())
    }

    /// 清理缓存
    ///
    /// 清除内存中的缓存，下次访问时会重新从磁盘加载。
    #[allow(dead_code)]
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
    }

    /// 强制从磁盘重新加载
    ///
    /// 清除缓存并重新从磁盘加载索引。
    #[allow(dead_code)]
    pub async fn reload(&self) -> Result<WallpaperIndex> {
        self.clear_cache();
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

        manager.upsert_wallpaper(wallpaper.clone()).await.unwrap();

        let retrieved = manager.get_wallpaper("20240101").await.unwrap();
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

        manager.upsert_wallpapers(wallpapers).await.unwrap();

        let all = manager.get_all_wallpapers().await.unwrap();
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
            manager.upsert_wallpaper(wallpaper.clone()).await.unwrap();
        }

        // 第二个管理器实例（模拟程序重启）
        {
            let manager = IndexManager::new(temp_dir.clone());
            let retrieved = manager.get_wallpaper("20240101").await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().title, "Persist Test");
        }

        // 清理
        let _ = fs::remove_dir_all(&temp_dir).await;
    }
}
