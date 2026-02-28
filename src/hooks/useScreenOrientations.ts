import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * 屏幕方向信息
 */
export interface ScreenOrientation {
  /** 屏幕索引 */
  screen_index: number;
  /** 是否为竖屏（高度 > 宽度） */
  is_portrait: boolean;
  /** 屏幕宽度（像素） */
  width: number;
  /** 屏幕高度（像素） */
  height: number;
}

/**
 * 获取屏幕方向信息的 Hook
 */
export function useScreenOrientations() {
  const [orientations, setOrientations] = useState<ScreenOrientation[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * 获取所有屏幕的方向信息
   */
  const fetchOrientations = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<ScreenOrientation[]>(
        "get_screen_orientations",
      );
      setOrientations(result);
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取屏幕方向失败";
      setError(errorMessage);
      console.error("Failed to get screen orientations:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  // 组件挂载时自动获取一次
  useEffect(() => {
    fetchOrientations();
  }, [fetchOrientations]);

  /**
   * 判断指定屏幕是否为竖屏
   */
  const isPortrait = useCallback(
    (screenIndex: number = 0) => {
      const screen = orientations.find((o) => o.screen_index === screenIndex);
      return screen?.is_portrait ?? false;
    },
    [orientations],
  );

  /**
   * 获取指定屏幕的尺寸
   */
  const getScreenSize = useCallback(
    (screenIndex: number = 0) => {
      const screen = orientations.find((o) => o.screen_index === screenIndex);
      return screen ? { width: screen.width, height: screen.height } : null;
    },
    [orientations],
  );

  return {
    orientations,
    loading,
    error,
    fetchOrientations,
    isPortrait,
    getScreenSize,
  };
}
