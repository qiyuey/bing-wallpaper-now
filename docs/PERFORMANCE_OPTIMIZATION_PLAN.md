# 性能优化实施规划

> Bing Wallpaper Now - Performance Optimization Roadmap  
> 版本: 1.0  
> 日期: 2025-10-27

## 概述

本文档详细规划了 Bing Wallpaper Now 项目的性能优化路线图，涵盖四个主要优化方向：

1. **并发图片下载与连接池复用** - 后端优化
2. **React 组件渲染优化** - 前端优化  
3. **非阻塞文件 I/O 与高效元数据存储** - 后端优化
4. **内存高效的图片流式传输与压缩** - 后端优化

---

## 一、并发图片下载与连接池复用

### 当前状况分析

**文件**: `src-tauri/src/download_manager.rs`

**现状**:

- 使用简单的 `reqwest::get()` 进行单次下载
- 每次请求创建新的 HTTP 客户端
- 顺序下载，没有并发控制
- 无连接池复用，每个请求建立新的 TCP 连接

**问题**:

- 下载 8 张壁纸需要顺序执行 8 次网络请求
- 每次请求都有 TCP 握手开销（~50-100ms）
- 无法充分利用网络带宽
- 总下载时间 = 单次下载时间 × 8

### 优化目标

- ✅ 实现并发下载，最多 8 张壁纸同时下载
- ✅ 连接池复用，减少 TCP 握手开销
- ✅ 优雅处理部分失败（某张图片失败不影响其他）
- ✅ 添加进度跟踪和错误重试机制

### 实施方案

#### 1.1 创建全局 HTTP 客户端（连接池复用）

```rust
// src-tauri/src/download_manager.rs
use std::sync::LazyLock;
use reqwest::Client;
use std::time::Duration;

/// 全局 HTTP 客户端，复用连接池
static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .pool_max_idle_per_host(8)  // 每个主机最多 8 个空闲连接
        .pool_idle_timeout(Some(Duration::from_secs(90)))  // 连接空闲 90 秒后关闭
        .timeout(Duration::from_secs(60))  // 请求超时 60 秒
        .connect_timeout(Duration::from_secs(10))  // 连接超时 10 秒
        .user_agent("BingWallpaperNow/0.2.0")
        .build()
        .expect("Failed to create HTTP client")
});
```

#### 1.2 实现并发下载逻辑

```rust
use futures::stream::{self, StreamExt};
use anyhow::{Context, Result};

/// 并发下载多张壁纸
/// 
/// # Arguments
/// * `download_tasks` - 下载任务列表 (url, save_path)
/// * `max_concurrent` - 最大并发数 (默认 4)
pub async fn download_images_concurrent(
    download_tasks: Vec<(String, PathBuf)>,
    max_concurrent: usize,
) -> Vec<Result<PathBuf>> {
    stream::iter(download_tasks)
        .map(|(url, save_path)| async move {
            download_image_with_retry(&url, &save_path, 3).await
        })
        .buffer_unordered(max_concurrent)  // 并发执行
        .collect::<Vec<_>>()
        .await
}

/// 带重试机制的下载
async fn download_image_with_retry(
    url: &str,
    save_path: &Path,
    max_retries: usize,
) -> Result<PathBuf> {
    let mut attempts = 0;
    let mut last_error = None;

    while attempts < max_retries {
        match download_image_internal(url, save_path).await {
            Ok(_) => return Ok(save_path.to_path_buf()),
            Err(e) => {
                attempts += 1;
                last_error = Some(e);
                if attempts < max_retries {
                    // 指数退避: 1s, 2s, 4s
                    tokio::time::sleep(Duration::from_secs(1 << (attempts - 1))).await;
                }
            }
        }
    }

    Err(last_error.unwrap().context(format!("Failed after {} attempts", max_retries)))
}

/// 内部下载实现（使用全局客户端）
async fn download_image_internal(url: &str, save_path: &Path) -> Result<()> {
    // 检查文件是否已存在
    if save_path.exists() {
        return Ok(());
    }

    // 创建父目录
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 使用全局客户端下载（连接池复用）
    let response = HTTP_CLIENT.get(url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    let bytes = response.bytes().await?;

    // 原子写入（临时文件 + 重命名）
    let temp_path = save_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path).await?;
    file.write_all(&bytes).await?;
    file.sync_all().await?;
    fs::rename(&temp_path, save_path).await?;

    Ok(())
}
```

#### 1.3 集成到 wallpaper_manager.rs

```rust
// src-tauri/src/wallpaper_manager.rs
use crate::download_manager::download_images_concurrent;
use crate::bing_api::get_wallpaper_url;

pub async fn download_all_wallpapers(
    images: &[BingImageEntry],
    directory: &Path,
) -> Vec<Result<LocalWallpaper>> {
    // 准备下载任务
    let tasks: Vec<(String, PathBuf)> = images
        .iter()
        .map(|img| {
            let url = get_wallpaper_url(&img.urlbase, "UHD");
            let path = directory.join(format!("{}.jpg", img.startdate));
            (url, path)
        })
        .collect();

    // 并发下载（最多 4 个并发）
    let results = download_images_concurrent(tasks, 4).await;

    // 处理结果...
    results
        .into_iter()
        .zip(images.iter())
        .map(|(result, img)| {
            result.and_then(|path| {
                // 保存元数据
                save_wallpaper_metadata(img, &path).await
            })
        })
        .collect()
}
```

### 性能收益预估

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 下载 8 张壁纸耗时 | ~16-24s | ~4-6s | **70-75%** |
| TCP 连接数 | 8 次 | 1-4 次（复用） | **50-87%** |
| 网络利用率 | 12.5% | 50-100% | **4-8x** |

---

## 二、React 组件渲染优化

### 当前状况分析

**文件**:

- `src/components/WallpaperGrid.tsx`
- `src/components/WallpaperCard.tsx`
- `src/App.tsx`

**现状**:

- `WallpaperCard` 无 memo 优化，每次父组件更新都重新渲染
- `WallpaperGrid` 一次性渲染所有壁纸（8 张）
- 图片使用 `loading="lazy"` 但无虚拟化
- 无防抖/节流处理用户交互

**问题**:

- 父组件状态变化导致所有卡片不必要的重渲染
- 大量 DOM 节点影响初始渲染性能
- 图片加载阻塞用户交互

### 优化目标

- ✅ 使用 React.memo 避免不必要的重渲染
- ✅ 优化组件 props 传递（避免内联函数）
- ✅ 实现骨架屏提升感知性能
- ✅ 图片懒加载和渐进式加载
- ⚠️ 虚拟列表（可选，当壁纸数量 > 20 时考虑）

### 实施方案

#### 2.1 优化 WallpaperCard 组件

```tsx
// src/components/WallpaperCard.tsx
import { memo, useCallback } from "react";
import { LocalWallpaper } from "../types";
import { openUrl } from "@tauri-apps/plugin-opener";
import { convertFileSrc } from "@tauri-apps/api/core";

interface WallpaperCardProps {
  wallpaper: LocalWallpaper;
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
}

// 使用 memo 优化，只在 props 变化时重新渲染
export const WallpaperCard = memo(function WallpaperCard({
  wallpaper,
  onSetWallpaper,
}: WallpaperCardProps) {
  // 使用 useCallback 避免函数重新创建
  const handleImageClick = useCallback(async () => {
    if (wallpaper.copyright_link) {
      try {
        await openUrl(wallpaper.copyright_link);
      } catch (err) {
        console.error("Failed to open link:", err);
      }
    }
  }, [wallpaper.copyright_link]);

  const handleSetWallpaper = useCallback(() => {
    onSetWallpaper(wallpaper);
  }, [wallpaper, onSetWallpaper]);

  const parseTitleAndSubtitle = useCallback(() => {
    const title = wallpaper.title;
    const copyright = wallpaper.copyright;
    const match = copyright.match(/^([^(]+?)(?:\s*\(([^)]+)\))?$/);
    const subtitle = match ? match[1].trim() : copyright;
    return { title, subtitle };
  }, [wallpaper.title, wallpaper.copyright]);

  const { title, subtitle } = parseTitleAndSubtitle();
  const imageUrl = convertFileSrc(wallpaper.file_path);

  return (
    <div className="wallpaper-card">
      <div
        className="wallpaper-image-container"
        onClick={handleImageClick}
        style={{ cursor: "pointer" }}
        title="点击查看详情"
      >
        <img
          src={imageUrl}
          alt={title}
          className="wallpaper-image"
          loading="lazy"
          decoding="async"  // 异步解码，不阻塞主线程
        />
      </div>
      <div className="wallpaper-info">
        <h3 className="wallpaper-title">{title}</h3>
        {subtitle && <p className="wallpaper-subtitle">{subtitle}</p>}
      </div>
      <div className="wallpaper-actions">
        <button onClick={handleSetWallpaper} className="btn btn-primary">
          设置壁纸
        </button>
      </div>
    </div>
  );
});
```

#### 2.2 优化 WallpaperGrid 组件

```tsx
// src/components/WallpaperGrid.tsx
import { memo, useCallback } from "react";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
}

export const WallpaperGrid = memo(function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
}: WallpaperGridProps) {
  // 使用 useCallback 避免传递不稳定的 props
  const handleSetWallpaper = useCallback(
    (wallpaper: LocalWallpaper) => {
      onSetWallpaper(wallpaper);
    },
    [onSetWallpaper]
  );

  if (loading) {
    return (
      <div className="wallpaper-grid-loading">
        {/* 骨架屏 */}
        {[...Array(8)].map((_, i) => (
          <div key={i} className="wallpaper-card-skeleton">
            <div className="skeleton-image" />
            <div className="skeleton-text" />
          </div>
        ))}
      </div>
    );
  }

  if (wallpapers.length === 0) {
    return (
      <div className="wallpaper-grid-empty">
        <p>暂无壁纸</p>
      </div>
    );
  }

  return (
    <div className="wallpaper-grid">
      {wallpapers.map((wallpaper) => (
        <WallpaperCard
          key={wallpaper.id}
          wallpaper={wallpaper}
          onSetWallpaper={handleSetWallpaper}
        />
      ))}
    </div>
  );
});
```

#### 2.3 添加骨架屏样式

```css
/* src/App.css */
.wallpaper-card-skeleton {
  border-radius: 8px;
  overflow: hidden;
  background: #f5f5f5;
  animation: pulse 1.5s ease-in-out infinite;
}

.skeleton-image {
  width: 100%;
  height: 200px;
  background: linear-gradient(
    90deg,
    #f0f0f0 25%,
    #e0e0e0 50%,
    #f0f0f0 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-text {
  height: 20px;
  margin: 12px;
  background: linear-gradient(
    90deg,
    #f0f0f0 25%,
    #e0e0e0 50%,
    #f0f0f0 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
}

@keyframes shimmer {
  0% {
    background-position: -200% 0;
  }
  100% {
    background-position: 200% 0;
  }
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.8;
  }
}
```

#### 2.4 优化 App.tsx 交互处理

```tsx
// src/App.tsx
import { useCallback, useMemo } from "react";

function App() {
  // ... 其他代码

  // 使用 useCallback 避免函数重新创建
  const handleSetWallpaper = useCallback(async (wallpaper: LocalWallpaper) => {
    try {
      await setDesktopWallpaper(wallpaper.file_path);
    } catch (err) {
      console.error("Failed to set wallpaper:", err);
      alert("设置壁纸失败: " + String(err));
    }
  }, [setDesktopWallpaper]);

  // 防抖刷新函数（避免重复点击）
  const handleRefresh = useCallback(
    debounce(async () => {
      await fetchLocalWallpapers();
      try {
        await forceUpdate();
      } catch (err) {
        console.warn("Force update failed:", err);
      }
    }, 1000),  // 1 秒防抖
    [fetchLocalWallpapers, forceUpdate]
  );

  // ... 渲染逻辑
}

// 防抖工具函数
function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: NodeJS.Timeout | null = null;
  return (...args: Parameters<T>) => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}
```

### 性能收益预估

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 首次渲染时间 | ~200ms | ~100ms | **50%** |
| 重渲染次数 | 8-16 次 | 1-2 次 | **87-94%** |
| 交互响应时间 | ~100ms | ~50ms | **50%** |
| 内存占用 | 基准 | -10% | **10%** |

---

## 三、非阻塞文件 I/O 与高效元数据存储

### 当前状况分析

**文件**: `src-tauri/src/storage.rs`

**现状**:

- 使用 `tokio::fs` 异步 I/O（✅ 已优化）
- 元数据独立存储为 JSON 文件（每张壁纸 1 个 JSON）
- 读取所有壁纸需要遍历目录 + 读取 N 个 JSON 文件
- 无索引，查询效率 O(n)

**问题**:

- 8 张壁纸需要读取 16 个文件（8 张 JPG + 8 个 JSON）
- 每次启动都要扫描目录并读取所有元数据
- 元数据分散，无法批量操作

### 优化目标

- ✅ 合并元数据到单一索引文件
- ✅ 实现增量更新（只写入变化的数据）
- ✅ 添加内存缓存减少磁盘 I/O
- ✅ 使用 MessagePack 替代 JSON（更紧凑）

### 实施方案

#### 3.1 设计统一的元数据索引

```rust
// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 壁纸元数据索引（单一文件存储）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WallpaperIndex {
    /// 版本号（用于兼容性检查）
    pub version: u32,
    /// 最后更新时间
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// 壁纸列表（key = start_date）
    pub wallpapers: HashMap<String, LocalWallpaper>,
}
```

#### 3.2 实现索引管理模块

```rust
// src-tauri/src/index_manager.rs
use crate::models::{LocalWallpaper, WallpaperIndex};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;

/// 索引文件名
const INDEX_FILE: &str = "index.msgpack";
const INDEX_VERSION: u32 = 1;

/// 内存缓存的索引管理器
pub struct IndexManager {
    directory: PathBuf,
    cache: Arc<Mutex<Option<WallpaperIndex>>>,
}

impl IndexManager {
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
    pub async fn load_index(&self) -> Result<WallpaperIndex> {
        // 检查缓存
        {
            let cache = self.cache.lock().unwrap();
            if let Some(index) = cache.as_ref() {
                return Ok(index.clone());
            }
        }

        // 从磁盘加载
        let index = self.load_from_disk().await.unwrap_or_default();

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

        let bytes = fs::read(&path).await?;
        let index: WallpaperIndex = rmp_serde::from_slice(&bytes)
            .context("Failed to deserialize index")?;

        // 版本检查
        if index.version != INDEX_VERSION {
            log::warn!("Index version mismatch, rebuilding...");
            return self.rebuild_index().await;
        }

        Ok(index)
    }

    /// 保存索引到磁盘
    pub async fn save_index(&self, index: &WallpaperIndex) -> Result<()> {
        // 序列化为 MessagePack（比 JSON 更紧凑）
        let bytes = rmp_serde::to_vec(index)
            .context("Failed to serialize index")?;

        // 原子写入
        let temp_path = self.index_path().with_extension("tmp");
        fs::write(&temp_path, &bytes).await?;
        fs::rename(&temp_path, self.index_path()).await?;

        // 更新缓存
        {
            let mut cache = self.cache.lock().unwrap();
            *cache = Some(index.clone());
        }

        Ok(())
    }

    /// 添加或更新壁纸
    pub async fn upsert_wallpaper(&self, wallpaper: LocalWallpaper) -> Result<()> {
        let mut index = self.load_index().await?;
        index.wallpapers.insert(wallpaper.start_date.clone(), wallpaper);
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 删除壁纸
    pub async fn remove_wallpaper(&self, start_date: &str) -> Result<()> {
        let mut index = self.load_index().await?;
        index.wallpapers.remove(start_date);
        index.last_updated = chrono::Utc::now();
        self.save_index(&index).await
    }

    /// 获取所有壁纸（排序）
    pub async fn get_all_wallpapers(&self) -> Result<Vec<LocalWallpaper>> {
        let index = self.load_index().await?;
        let mut wallpapers: Vec<_> = index.wallpapers.into_values().collect();
        wallpapers.sort_by(|a, b| b.start_date.cmp(&a.start_date));
        Ok(wallpapers)
    }

    /// 重建索引（从现有 JSON 文件迁移）
    async fn rebuild_index(&self) -> Result<WallpaperIndex> {
        log::info!("Rebuilding index from JSON files...");
        
        let mut index = WallpaperIndex {
            version: INDEX_VERSION,
            last_updated: chrono::Utc::now(),
            wallpapers: HashMap::new(),
        };

        // 扫描目录中的 JSON 文件
        let mut entries = fs::read_dir(&self.directory).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path).await {
                    if let Ok(wallpaper) = serde_json::from_str::<LocalWallpaper>(&content) {
                        index.wallpapers.insert(wallpaper.start_date.clone(), wallpaper);
                    }
                }
            }
        }

        // 保存新索引
        self.save_index(&index).await?;
        Ok(index)
    }

    /// 清理缓存
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        *cache = None;
    }
}
```

#### 3.3 集成到 storage.rs

```rust
// src-tauri/src/storage.rs
use crate::index_manager::IndexManager;
use std::sync::Arc;

/// 全局索引管理器（使用 Arc 共享）
static INDEX_MANAGER: once_cell::sync::OnceCell<Arc<IndexManager>> = 
    once_cell::sync::OnceCell::new();

/// 初始化索引管理器
pub fn initialize_index_manager(directory: PathBuf) {
    INDEX_MANAGER.get_or_init(|| {
        Arc::new(IndexManager::new(directory))
    });
}

/// 获取索引管理器
fn get_index_manager() -> &'static Arc<IndexManager> {
    INDEX_MANAGER.get().expect("IndexManager not initialized")
}

/// 获取所有本地壁纸（使用索引）
pub async fn get_local_wallpapers(directory: &Path) -> Result<Vec<LocalWallpaper>> {
    initialize_index_manager(directory.to_path_buf());
    let manager = get_index_manager();
    manager.get_all_wallpapers().await
}

/// 保存壁纸元数据（使用索引）
pub async fn save_wallpaper_metadata(
    wallpaper: &LocalWallpaper,
    _directory: &Path,
) -> Result<()> {
    let manager = get_index_manager();
    manager.upsert_wallpaper(wallpaper.clone()).await
}

/// 删除壁纸（同时更新索引）
pub async fn delete_wallpaper(start_date: &str, directory: &Path) -> Result<()> {
    let manager = get_index_manager();
    
    // 删除文件
    let image_path = directory.join(format!("{}.jpg", start_date));
    if image_path.exists() {
        fs::remove_file(&image_path).await?;
    }

    // 更新索引
    manager.remove_wallpaper(start_date).await?;
    Ok(())
}
```

#### 3.4 添加依赖

```toml
# src-tauri/Cargo.toml
[dependencies]
rmp-serde = "1.3"  # MessagePack 序列化
once_cell = "1.20"  # 懒加载静态变量
```

### 性能收益预估

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 启动加载时间 | ~150ms | ~30ms | **80%** |
| 元数据文件数 | 8 个 JSON | 1 个 MsgPack | **87%** |
| 元数据大小 | ~8KB | ~3KB | **62%** |
| 查询延迟 | ~50ms | ~5ms | **90%** |

---

## 四、内存高效的图片流式传输与压缩

### 当前状况分析

**文件**: `src-tauri/src/download_manager.rs`

**现状**:

- 使用 `response.bytes()` 将整个图片加载到内存
- 单张 UHD 图片约 2-3MB，8 张约 16-24MB
- 无压缩，原始 JPEG 直接保存
- 下载时内存占用峰值较高

**问题**:

- 内存占用高（8 张图片 = ~24MB 内存）
- 大文件下载时阻塞等待
- 无进度反馈

### 优化目标

- ✅ 流式下载，边下载边写入磁盘
- ✅ 减少内存占用（~1MB 缓冲区）
- ✅ 添加下载进度回调
- ⚠️ 可选：图片压缩（如果需要节省磁盘空间）

### 实施方案

#### 4.1 实现流式下载

```rust
// src-tauri/src/download_manager.rs
use tokio::io::AsyncWriteExt;
use futures::StreamExt;

/// 流式下载图片（内存高效）
async fn download_image_streaming(
    url: &str,
    save_path: &Path,
    progress_callback: Option<impl Fn(u64, u64)>,
) -> Result<()> {
    if save_path.exists() {
        return Ok(());
    }

    // 创建父目录
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    // 发起请求
    let response = HTTP_CLIENT.get(url).send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("HTTP {}", response.status());
    }

    // 获取文件大小
    let total_size = response.content_length().unwrap_or(0);

    // 创建临时文件
    let temp_path = save_path.with_extension("tmp");
    let mut file = fs::File::create(&temp_path).await?;

    // 流式写入（使用 4KB 缓冲区）
    let mut stream = response.bytes_stream();
    let mut downloaded = 0u64;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        
        downloaded += chunk.len() as u64;
        
        // 回调进度
        if let Some(ref callback) = progress_callback {
            callback(downloaded, total_size);
        }
    }

    // 确保数据写入磁盘
    file.sync_all().await?;

    // 原子重命名
    fs::rename(&temp_path, save_path).await?;

    Ok(())
}
```

#### 4.2 添加进度跟踪

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 下载进度跟踪器
pub struct DownloadProgress {
    pub completed: Arc<AtomicU64>,
    pub total: Arc<AtomicU64>,
}

impl DownloadProgress {
    pub fn new() -> Self {
        Self {
            completed: Arc::new(AtomicU64::new(0)),
            total: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn update(&self, completed: u64, total: u64) {
        self.completed.store(completed, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
    }

    pub fn get_percentage(&self) -> f64 {
        let completed = self.completed.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        if total == 0 {
            0.0
        } else {
            (completed as f64 / total as f64) * 100.0
        }
    }
}

/// 并发下载（带进度）
pub async fn download_images_with_progress(
    download_tasks: Vec<(String, PathBuf)>,
    max_concurrent: usize,
    progress: Arc<DownloadProgress>,
) -> Vec<Result<PathBuf>> {
    let total_size = download_tasks.len() as u64;
    progress.total.store(total_size, Ordering::Relaxed);

    stream::iter(download_tasks)
        .enumerate()
        .map(|(idx, (url, save_path))| {
            let progress = Arc::clone(&progress);
            async move {
                let result = download_image_streaming(
                    &url,
                    &save_path,
                    Some(|_, _| {
                        progress.completed.store(idx as u64 + 1, Ordering::Relaxed);
                    }),
                )
                .await;
                
                result.map(|_| save_path)
            }
        })
        .buffer_unordered(max_concurrent)
        .collect::<Vec<_>>()
        .await
}
```

#### 4.3 可选：图片压缩

```rust
// 可选功能：下载后压缩图片以节省磁盘空间
use image::ImageFormat;

async fn compress_image_if_needed(
    image_path: &Path,
    max_size_kb: u64,
) -> Result<()> {
    let metadata = fs::metadata(image_path).await?;
    let size_kb = metadata.len() / 1024;

    // 如果文件小于阈值，跳过压缩
    if size_kb <= max_size_kb {
        return Ok(());
    }

    // 读取图片
    let img = image::open(image_path)?;

    // 重新编码为 JPEG，质量 85%
    let temp_path = image_path.with_extension("compressed.jpg");
    img.save_with_format(&temp_path, ImageFormat::Jpeg)?;

    // 替换原文件
    fs::rename(&temp_path, image_path).await?;

    Ok(())
}
```

### 性能收益预估

| 指标 | 优化前 | 优化后 | 提升 |
|------|--------|--------|------|
| 单次下载内存 | ~3MB | ~4KB | **99.8%** |
| 8 张下载内存峰值 | ~24MB | ~16KB | **99.9%** |
| 磁盘写入效率 | 阻塞 | 流式 | **2-3x** |
| 用户体验 | 无反馈 | 实时进度 | ✅ |

---

## 实施优先级与时间表

### Phase 1: 高优先级（1-2 周）

- ✅ **并发下载与连接池** (3-4 天)
  - 创建全局 HTTP 客户端
  - 实现并发下载逻辑
  - 集成到现有代码
  - 测试与验证

- ✅ **React 组件优化** (2-3 天)
  - 添加 React.memo
  - 优化 useCallback
  - 实现骨架屏
  - 性能测试

### Phase 2: 中优先级（1-2 周）

- ✅ **元数据索引优化** (4-5 天)
  - 设计索引结构
  - 实现 IndexManager
  - 数据迁移逻辑
  - 兼容性测试

- ✅ **流式下载与进度** (2-3 天)
  - 实现流式下载
  - 添加进度跟踪
  - UI 进度显示

### Phase 3: 低优先级（可选）

- ⚠️ **图片压缩** (1-2 天)
- ⚠️ **虚拟列表** (仅当壁纸数量 > 20)

---

## 验证与测试

### 性能基准测试

```bash
# 下载性能测试
cargo bench --bench download_benchmark

# 渲染性能测试
pnpm run test:perf

# 内存占用测试
cargo test --release -- --nocapture memory_test
```

### 测试场景

1. **下载测试**
   - 测试 8 张壁纸并发下载时间
   - 验证连接池复用
   - 测试部分失败场景

2. **渲染测试**
   - 测试组件重渲染次数
   - 验证 memo 效果
   - 测试骨架屏显示

3. **存储测试**
   - 测试索引加载时间
   - 验证数据迁移
   - 测试并发读写

4. **内存测试**
   - 测试流式下载内存占用
   - 验证缓存清理
   - 测试内存泄漏

---

## 回滚计划

每个优化都应有独立的 feature flag，方便回滚：

```rust
// src-tauri/src/lib.rs
const ENABLE_CONCURRENT_DOWNLOAD: bool = true;
const ENABLE_INDEX_MANAGER: bool = true;
const ENABLE_STREAMING_DOWNLOAD: bool = true;
```

---

## 总结

### 预期整体收益

| 类别 | 优化项 | 预期提升 |
|------|--------|----------|
| **网络** | 并发下载 + 连接池 | 70-75% ⬆️ |
| **渲染** | React.memo + 骨架屏 | 50-87% ⬆️ |
| **存储** | 索引 + MessagePack | 80-90% ⬆️ |
| **内存** | 流式传输 | 99% ⬇️ |

### 用户感知提升

- ⚡ 启动速度：从 ~500ms 降至 ~200ms
- ⚡ 下载速度：从 ~20s 降至 ~5s
- ⚡ 交互响应：从 ~100ms 降至 ~50ms
- 💾 内存占用：从 ~50MB 降至 ~20MB

---

*文档版本: 1.0*  
*最后更新: 2025-10-27*
