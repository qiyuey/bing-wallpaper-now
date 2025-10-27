import { useState, useEffect, useCallback, useMemo } from "react";
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
  const [isFirstLoad, setIsFirstLoad] = useState(true); // 标记是否首次加载

  const [lastUpdateTime, setLastUpdateTime] = useState<string | null>(null);

  /**
   * 获取本地壁纸列表
   * @param showLoading 是否显示加载状态，默认 false 避免不必要的闪烁
   */
  const fetchLocalWallpapers = useCallback(async (showLoading = false) => {
    if (showLoading) {
      setLoading(true);
    }
    setError(null);
    try {
      const wallpapers = await invoke<LocalWallpaper[]>("get_local_wallpapers");
      // 只有数据真正变化时才更新状态，避免不必要的重渲染
      setLocalWallpapers((prev) => {
        if (JSON.stringify(prev) === JSON.stringify(wallpapers)) {
          return prev;
        }
        // 首次加载完成后，标记不再是首次加载
        if (isFirstLoad && wallpapers.length > 0) {
          setIsFirstLoad(false);
        }
        return wallpapers;
      });
    } catch (err) {
      setError(String(err));
    } finally {
      if (showLoading) {
        setLoading(false);
      }
    }
  }, [isFirstLoad]);

  /**
   * 后端状态轮询：最后更新时间
   */
  const pollStatus = useCallback(async () => {
    try {
      const last = await invoke<string | null>("get_last_update_time");
      setLastUpdateTime((prev) => (prev === last ? prev : last));
    } catch {
      // 忽略错误，防止抖动
    }
  }, []);

  /**
   * 设置桌面壁纸（不使用 loading 状态避免影响整体 UI）
   */
  const setDesktopWallpaper = useCallback(async (filePath: string) => {
    await invoke("set_desktop_wallpaper", { filePath });
  }, []);

  /**
   * 清理旧壁纸
   */
  const cleanupWallpapers = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const deletedCount = await invoke<number>("cleanup_wallpapers");
      await fetchLocalWallpapers(true);
      return deletedCount;
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  }, [fetchLocalWallpapers]);

  /**
   * 手动触发后台更新一次（force_update 已在后端执行拉取、下载、清理、自动应用）
   * 成功后更新本地列表与最后更新时间
   */
  const forceUpdate = useCallback(async () => {
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
      await fetchLocalWallpapers(true);
      await pollStatus();
    } catch (err) {
      setError(String(err));
      throw err;
    } finally {
      setLoading(false);
    }
  }, [localWallpapers, fetchLocalWallpapers, pollStatus]);

  // 初始加载：只加载本地，并获取一次状态（初始加载显示 loading）
  useEffect(() => {
    fetchLocalWallpapers(true);
    pollStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // 监听后端壁纸更新事件，自动刷新列表（静默刷新，不显示 loading）
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen("wallpaper-updated", () => {
          console.warn("收到壁纸更新事件，刷新列表...");
          // 静默刷新，不显示 loading
          fetchLocalWallpapers(false);
          pollStatus();
        });
      } catch (e) {
        console.error("Failed to bind wallpaper-updated event:", e);
      }
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [fetchLocalWallpapers, pollStatus]);

  // 轮询后台状态（降低频率到每 10 秒，减少性能开销）
  useEffect(() => {
    let mounted = true;
    const interval = setInterval(() => {
      if (mounted) {
        pollStatus();
      }
    }, 10000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [pollStatus]);

  // 计算是否已是最新（衍生状态，避免重复手动更新）
  const isUpToDate = useMemo(() => {
    if (!localWallpapers.length) return false;
    const now = new Date();
    const todayStr = `${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(
      now.getDate(),
    ).padStart(2, "0")}`;
    return localWallpapers[0].start_date === todayStr;
  }, [localWallpapers]);

  return {
    localWallpapers,
    loading,
    error,
    isFirstLoad,
    lastUpdateTime,
    isUpToDate,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    cleanupWallpapers,
    forceUpdate,
  };
}
