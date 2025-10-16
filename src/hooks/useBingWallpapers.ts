import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LocalWallpaper } from "../types";

/**
 * 必应壁纸 Hook（扩展版）
 * 说明：
 *  - 后端负责周期/零点更新与快速重试
 *  - 前端轮询获取：本地壁纸列表、最后更新时间
 *  - 提供手动触发一次更新的能力
 */
export function useBingWallpapers() {
  const [localWallpapers, setLocalWallpapers] = useState<LocalWallpaper[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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
   * 后端状态轮询：最后更新时间
   */
  const pollStatus = async () => {
    try {
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
   * 成功后更新本地列表与最后更新时间
   */
  const forceUpdate = async () => {
    setLoading(true);
    setError(null);
    try {
      // 计算今日日期字符串（与 start_date 格式一致：YYYYMMDD）
      const now = new Date();
      const todayStr = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(
        now.getDate(),
      ).padStart(2, "0")}`;

      // 若已是最新（列表第一项日期与今日匹配），不再触发后端 force_update，直接更新状态
      if (
        localWallpapers.length > 0 &&
        localWallpapers[0].start_date === todayStr
      ) {
        await pollStatus();
        setLoading(false);
        return;
      }

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

  // 计算是否已是最新（衍生状态，避免重复手动更新）
  const isUpToDate = (() => {
    if (!localWallpapers.length) return false;
    const now = new Date();
    const todayStr = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(
      now.getDate(),
    ).padStart(2, "0")}`;
    return localWallpapers[0].start_date === todayStr;
  })();

  return {
    localWallpapers,
    loading,
    error,
    lastUpdateTime,
    isUpToDate,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    cleanupWallpapers,
    forceUpdate,
  };
}
