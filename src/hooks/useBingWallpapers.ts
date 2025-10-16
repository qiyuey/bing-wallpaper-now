import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LocalWallpaper } from "../types";

/**
 * 必应壁纸 Hook（精简版）
 * 说明：
 *  - 远程 Bing 图片获取与下载现在由后端自动更新任务处理
 *  - GUI 仅关心：本地列表、设置壁纸、清理旧壁纸、手动触发一次更新
 */
export function useBingWallpapers() {
  const [localWallpapers, setLocalWallpapers] = useState<LocalWallpaper[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  /**
   * 设置桌面壁纸（不使用 loading 状态避免影响整体 UI）
   */
  const setDesktopWallpaper = async (filePath: string) => {
    try {
      await invoke("set_desktop_wallpaper", { filePath });
    } catch (err) {
      throw err;
    }
  };

  /**
   * 清理旧壁纸
   */
  const cleanupWallpapers = async () => {
    setLoading(true);
    setError(null);
    try {
      const deletedCount = await invoke<number>("cleanup_wallpapers");
      await fetchLocalWallpapers();
      return deletedCount;
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  };

  /**
   * 手动触发后台更新一次（force_update 已在后端执行拉取、下载、清理、自动应用）
   * 成功后刷新本地列表
   */
  const forceUpdate = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("force_update");
      await fetchLocalWallpapers();
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  };

  // 初始加载：只加载本地，后台自动更新任务会周期性增加新壁纸
  useEffect(() => {
    fetchLocalWallpapers();
  }, []);

  return {
    localWallpapers,
    loading,
    error,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    cleanupWallpapers,
    forceUpdate,
  };
}
