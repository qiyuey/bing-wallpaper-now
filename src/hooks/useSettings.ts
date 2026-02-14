import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { AppSettings } from "../types";

/**
 * 应用设置 Hook
 */
export function useSettings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /**
   * 获取设置
   */
  const fetchSettings = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const appSettings = await invoke<AppSettings>("get_settings");
      setSettings(appSettings);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  }, []);

  /**
   * 更新设置
   */
  const updateSettings = async (newSettings: AppSettings) => {
    setLoading(true);
    setError(null);
    try {
      // Tauri 2 的参数传递：使用驼峰命名，Tauri 会自动转换为 Rust 的蛇形命名
      await invoke("update_settings", {
        newSettings: {
          auto_update: newSettings.auto_update,
          save_directory: newSettings.save_directory,
          launch_at_startup: newSettings.launch_at_startup,
          theme: newSettings.theme,
          language: newSettings.language,
          mkt: newSettings.mkt,
        },
      });
      // 从后端重新获取设置（含 resolved_language 等后端计算字段），确保前端状态完全一致
      const refreshed = await invoke<AppSettings>("get_settings");
      setSettings(refreshed);
    } catch (err) {
      console.error("updateSettings error:", err);
      setError(err as string);
      throw err;
    } finally {
      setLoading(false);
    }
  };

  /**
   * 获取默认壁纸目录
   */
  const getDefaultDirectory = async () => {
    try {
      return await invoke<string>("get_default_wallpaper_directory");
    } catch (err) {
      console.error("Failed to get default directory:", err);
      return null;
    }
  };

  // 初始加载
  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  return {
    settings,
    loading,
    error,
    fetchSettings,
    updateSettings,
    getDefaultDirectory,
  };
}
