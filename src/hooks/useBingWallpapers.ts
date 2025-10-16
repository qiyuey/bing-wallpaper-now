import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LocalWallpaper } from "../types";

/**
 * 必应壁纸 Hook（扩展版）
 * 说明：
 *  - 后端负责周期/零点更新与快速重试
 *  - 前端轮询获取：本地壁纸列表、更新进行中状态、最后更新时间
 *  - 提供手动触发一次更新的能力
 */
export function useBingWallpapers() {
  const [localWallpapers, setLocalWallpapers] = useState<LocalWallpaper[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [updating, setUpdating] = useState(false);
  const [lastUpdateTime, setLastUpdateTime] = useState<string | null>(null);

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
   * 后端状态轮询：更新进行中标记 & 最后更新时间
   */
  const pollStatus = async () => {
    try {
      const inProgress = await invoke<boolean>("get_update_in_progress");
      setUpdating(inProgress);
      const last = await invoke<string | null>("get_last_update_time");
      setLastUpdateTime(last);
    } catch {
      // 忽略错误，防止抖动
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
   * 成功后刷新本地列表与状态
   */
  const forceUpdate = async () => {
    setLoading(true);
    setError(null);
    try {
      await invoke("force_update");
      await fetchLocalWallpapers();
      await pollStatus();
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  };

  // 初始加载：只加载本地，并获取一次状态
  useEffect(() => {
    fetchLocalWallpapers();
    pollStatus();
  }, []);

  // 轮询后台状态（每 5 秒）
  useEffect(() => {
    let mounted = true;
    const interval = setInterval(() => {
      if (mounted) {
        pollStatus();
      }
    }, 5000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  return {
    localWallpapers,
    loading,
    error,
    updating,
    lastUpdateTime,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    cleanupWallpapers,
    forceUpdate,
  };
}
