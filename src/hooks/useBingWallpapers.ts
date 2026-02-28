import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  LocalWallpaper,
  LocalWallpaperRaw,
  MarketGroup,
  MarketStatus,
  normalizeWallpapers,
} from "../types";
import { createSafeUnlisten } from "../utils/eventListener";

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
  const [effectiveMkt, setEffectiveMkt] = useState<string | null>(null);
  const [mktLabelMap, setMktLabelMap] = useState<Map<string, string>>(
    new Map(),
  );

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
      const wallpapersRaw = await invoke<LocalWallpaperRaw[]>(
        "get_local_wallpapers",
      );
      // 转换短字段名为完整字段名
      const wallpapers = normalizeWallpapers(wallpapersRaw);
      // 只有数据真正变化时才更新状态，避免不必要的重渲染
      // 注意：使用深度比较确保 title 和 copyright 变化时也能检测到
      setLocalWallpapers((prev) => {
        // 先比较数组长度
        if (prev.length !== wallpapers.length) {
          return wallpapers;
        }
        // 比较每个 wallpaper 的关键字段（title 和 copyright 可能因语言变化而变化）
        const hasChanges = prev.some((p, i) => {
          const w = wallpapers[i];
          return (
            p.end_date !== w.end_date ||
            p.title !== w.title ||
            p.copyright !== w.copyright
          );
        });
        return hasChanges ? wallpapers : prev;
      });
    } catch (err) {
      setError(String(err));
    } finally {
      if (showLoading) {
        setLoading(false);
      }
    }
  }, []);

  /**
   * 后端状态轮询：最后更新时间 + 当前生效的 mkt
   */
  const pollStatus = useCallback(async () => {
    try {
      const last = await invoke<string | null>("get_last_update_time");
      setLastUpdateTime((prev) => (prev === last ? prev : last));
    } catch {
      // 忽略错误，防止抖动
    }
    try {
      const status = await invoke<MarketStatus>("get_market_status");
      if (status?.effective_mkt) {
        setEffectiveMkt((prev) =>
          prev === status.effective_mkt ? prev : status.effective_mkt,
        );
      }
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
   * 手动触发后台更新一次（force_update 已在后端执行拉取、下载、清理、自动应用）
   * 成功后更新本地列表与最后更新时间
   */
  const forceUpdate = useCallback(
    async (_force: boolean = false) => {
      setLoading(true);
      setError(null);
      try {
        await invoke("force_update");
        await fetchLocalWallpapers(true);
        await pollStatus();
      } catch (err) {
        setError(String(err));
        throw err;
      } finally {
        setLoading(false);
      }
    },
    [fetchLocalWallpapers, pollStatus],
  );

  // 初始加载：加载本地壁纸、轮询状态、获取市场列表
  useEffect(() => {
    fetchLocalWallpapers(true);
    pollStatus();
    invoke<MarketGroup[]>("get_supported_mkts")
      .then((groups) => {
        const map = new Map<string, string>();
        for (const group of groups) {
          for (const market of group.markets) {
            map.set(market.code, market.label);
          }
        }
        setMktLabelMap(map);
      })
      .catch((err) => {
        console.error("Failed to load supported mkts:", err);
      });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Use refs to keep stable references to the callback functions
  const fetchLocalWallpapersRef = useRef(fetchLocalWallpapers);
  const pollStatusRef = useRef(pollStatus);

  // Update refs when functions change
  useEffect(() => {
    fetchLocalWallpapersRef.current = fetchLocalWallpapers;
  }, [fetchLocalWallpapers]);

  useEffect(() => {
    pollStatusRef.current = pollStatus;
  }, [pollStatus]);

  // 监听后端壁纸更新事件，自动刷新列表（静默刷新，不显示 loading）
  // 使用空依赖数组和 ref，确保监听器只创建一次，避免重复创建
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let mounted = true;

    (async () => {
      try {
        if (!mounted) return;

        const unlistenFn = await listen("wallpaper-updated", () => {
          // 使用 ref 来获取最新的函数，避免闭包陷阱
          fetchLocalWallpapersRef.current(false);
          pollStatusRef.current();
        });

        // Wrap unlisten to make it safe (handles React StrictMode double-mount)
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safeUnlisten;
        } else {
          safeUnlisten(); // Cleanup immediately if unmounted
        }
      } catch (e) {
        console.error("Failed to bind wallpaper-updated event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, []); // Empty deps - listener created once, never recreated

  // 优化：智能轮询后台状态
  // 使用页面可见性 API 和焦点检测，在应用获得焦点或变为可见时才轮询
  useEffect(() => {
    let mounted = true;
    let intervalId: ReturnType<typeof setInterval> | null = null;
    let lastPollTime = Date.now();

    const pollWhenActive = () => {
      if (!mounted) return;

      const now = Date.now();
      const timeSinceLastPoll = now - lastPollTime;

      // 如果距离上次轮询超过 10 秒，立即轮询一次
      if (timeSinceLastPoll >= 10000) {
        pollStatus();
        lastPollTime = now;
      }
    };

    const handleVisibilityChange = () => {
      if (!mounted) return;

      // 当页面变为可见时，立即轮询一次
      if (document.visibilityState === "visible") {
        pollWhenActive();
      }
    };

    const handleFocus = () => {
      if (!mounted) return;
      pollWhenActive();
    };

    // 监听页面可见性变化
    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("focus", handleFocus);

    // 定期轮询（降低频率到每 30 秒，减少性能开销）
    // 但通过可见性和焦点检测，实际轮询频率会更高
    intervalId = setInterval(() => {
      if (mounted && document.visibilityState === "visible") {
        pollWhenActive();
      }
    }, 30000); // 30 秒间隔

    // 初始轮询一次
    pollStatus();

    return () => {
      mounted = false;
      if (intervalId) {
        clearInterval(intervalId);
      }
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("focus", handleFocus);
    };
  }, [pollStatus]);

  // 派生值：mkt code → 显示名称（如 "zh-CN" → "中国大陆"）
  const effectiveMktLabel =
    effectiveMkt && mktLabelMap.size > 0
      ? (mktLabelMap.get(effectiveMkt) ?? null)
      : null;

  return {
    localWallpapers,
    loading,
    error,
    lastUpdateTime,
    effectiveMktLabel,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    forceUpdate,
  };
}
