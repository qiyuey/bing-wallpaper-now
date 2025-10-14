import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BingImageEntry, LocalWallpaper } from "../types";

/**
 * 必应壁纸 Hook
 */
export function useBingWallpapers() {
  const [bingImages, setBingImages] = useState<BingImageEntry[]>([]);
  const [localWallpapers, setLocalWallpapers] = useState<LocalWallpaper[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * 获取必应壁纸列表
   */
  const fetchBingImages = async (count: number = 8) => {
    setLoading(true);
    setError(null);
    try {
      const images = await invoke<BingImageEntry[]>("fetch_bing_images", {
        count,
      });
      setBingImages(images);
      setLoading(false);

      // 在后台异步下载所有获取到的壁纸（不阻塞UI）
      autoDownloadWallpapers(images);
    } catch (err) {
      setError(err as string);
      setLoading(false);
    }
  };

  /**
   * 自动下载壁纸（静默下载，不影响UI）
   */
  const autoDownloadWallpapers = async (images: BingImageEntry[]) => {
    // 静默下载，不显示加载状态
    for (const image of images) {
      try {
        await invoke<LocalWallpaper>("download_wallpaper", {
          imageEntry: image,
        });
      } catch (err) {
        // 静默失败，不影响用户体验
        console.log(`Auto-download failed for ${image.title}:`, err);
      }
    }
  };

  /**
   * 获取本地壁纸列表
   */
  const fetchLocalWallpapers = async () => {
    setLoading(true);
    setError(null);
    try {
      const wallpapers = await invoke<LocalWallpaper[]>("get_local_wallpapers");
      setLocalWallpapers(wallpapers);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  /**
   * 下载壁纸
   */
  const downloadWallpaper = async (imageEntry: BingImageEntry) => {
    setLoading(true);
    setError(null);
    try {
      const wallpaper = await invoke<LocalWallpaper>("download_wallpaper", {
        imageEntry,
      });
      setLocalWallpapers((prev) => [wallpaper, ...prev]);
      return wallpaper;
    } catch (err) {
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  };

  /**
   * 设置桌面壁纸
   */
  const setDesktopWallpaper = async (filePath: string) => {
    console.log("setDesktopWallpaper called with:", filePath);
    setLoading(true);
    setError(null);
    try {
      await invoke("set_desktop_wallpaper", { filePath });
      console.log("Wallpaper set successfully");
    } catch (err) {
      console.error("setDesktopWallpaper error:", err);
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  };

  /**
   * 下载并设置壁纸(一步到位)
   */
  const downloadAndSetWallpaper = async (imageEntry: BingImageEntry) => {
    console.log("downloadAndSetWallpaper called with:", imageEntry);
    const wallpaper = await downloadWallpaper(imageEntry);
    console.log("Downloaded wallpaper:", wallpaper);
    await setDesktopWallpaper(wallpaper.file_path);
    return wallpaper;
  };

  /**
   * 清理旧壁纸
   */
  const cleanupWallpapers = async () => {
    setLoading(true);
    setError(null);
    try {
      const deletedCount = await invoke<number>("cleanup_wallpapers");
      await fetchLocalWallpapers(); // 刷新列表
      return deletedCount;
    } catch (err) {
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  };

  // 初始加载
  useEffect(() => {
    fetchBingImages();
    fetchLocalWallpapers();
  }, []);

  return {
    bingImages,
    localWallpapers,
    loading,
    error,
    fetchBingImages,
    fetchLocalWallpapers,
    downloadWallpaper,
    setDesktopWallpaper,
    downloadAndSetWallpaper,
    cleanupWallpapers,
  };
}
