import { useState, useEffect } from "react";
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
  const fetchSettings = async () => {
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
  };

  /**
   * 更新设置
   */
  const updateSettings = async (newSettings: AppSettings) => {
    setLoading(true);
    setError(null);
    try {
      await invoke("update_settings", { newSettings });
      setSettings(newSettings);
    } catch (err) {
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
  }, []);

  return {
    settings,
    loading,
    error,
    fetchSettings,
    updateSettings,
    getDefaultDirectory,
  };
}
