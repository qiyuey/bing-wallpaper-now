import { useState, useEffect } from "react";
import { AppSettings } from "../types";
import { useSettings } from "../hooks/useSettings";
import { useTheme, Theme } from "../contexts/ThemeContext";
import { useI18n } from "../i18n/I18nContext";
import { open } from "@tauri-apps/plugin-dialog";

interface SettingsProps {
  onClose: () => void;
  version?: string;
  onLanguageChange?: () => void;
}

export function Settings({
  onClose,
  version,
  onLanguageChange,
}: SettingsProps) {
  const { settings, loading, updateSettings, getDefaultDirectory } =
    useSettings();
  const { applyThemeToUI } = useTheme();
  const { t, setLanguage } = useI18n();

  const [defaultDir, setDefaultDir] = useState<string>("");

  useEffect(() => {
    getDefaultDirectory().then((dir) => {
      if (dir) setDefaultDir(dir);
    });
  }, [getDefaultDirectory]);

  const handleChange = async (
    field: keyof AppSettings,
    value: string | number | boolean | null,
  ) => {
    if (!settings) return;

    try {
      const updatedSettings = { ...settings, [field]: value };
      await updateSettings(updatedSettings);

      // 如果是主题变化，立即应用到UI
      if (field === "theme" && typeof value === "string") {
        applyThemeToUI(value as Theme);
      }
      // 如果是语言变化，立即更新 i18n context
      if (field === "language" && typeof value === "string") {
        setLanguage(value as "auto" | "zh-CN" | "en-US");
        // 语言变化时，触发重新获取壁纸数据（以获取新语言的标题和描述）
        if (onLanguageChange) {
          onLanguageChange();
        }
      }
    } catch (err) {
      console.error("Update settings error:", err);
      alert(t("settingsSaveError") + ": " + err);
    }
  };

  const handleSelectFolder = async () => {
    if (!settings) return;

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.save_directory || defaultDir,
        title: t("selectDirectory"),
      });

      if (selected && typeof selected === "string") {
        await handleChange("save_directory", selected);
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
      alert(t("settingsFolderSelectError") + ": " + String(err));
    }
  };

  if (loading && !settings) {
    return <div className="settings-loading">{t("settingsLoading")}</div>;
  }

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <div className="settings-header">
          <div className="settings-header-left">
            <h2>{t("settingsTitle")}</h2>
            {version && <span className="settings-version">v{version}</span>}
          </div>
          <button onClick={onClose} className="btn-close">
            ×
          </button>
        </div>

        <div className="settings-body">
          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={settings?.launch_at_startup ?? false}
                onChange={(e) =>
                  handleChange("launch_at_startup", e.target.checked)
                }
              />
              <span>{t("launchAtStartup")}</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={settings?.auto_update ?? true}
                onChange={(e) => handleChange("auto_update", e.target.checked)}
              />
              <span>{t("autoUpdate")}</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label">{t("theme")}:</label>
            <div className="radio-group">
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="system"
                  checked={(settings?.theme ?? "system") === "system"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeSystem")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="light"
                  checked={(settings?.theme ?? "system") === "light"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeLight")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="dark"
                  checked={(settings?.theme ?? "system") === "dark"}
                  onChange={(e) =>
                    handleChange("theme", e.target.value as Theme)
                  }
                />
                <span>{t("themeDark")}</span>
              </label>
            </div>
          </div>

          <div className="settings-section">
            <label className="settings-label">{t("language")}:</label>
            <div className="radio-group">
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="auto"
                  checked={(settings?.language ?? "auto") === "auto"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageAuto")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="zh-CN"
                  checked={(settings?.language ?? "auto") === "zh-CN"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageZhCN")}</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="language"
                  value="en-US"
                  checked={(settings?.language ?? "auto") === "en-US"}
                  onChange={(e) => handleChange("language", e.target.value)}
                />
                <span>{t("languageEnUS")}</span>
              </label>
            </div>
          </div>

          <div className="settings-section">
            <label className="settings-label">
              {t("keepCount")}: {t("keepCountHint")}
              <input
                type="number"
                min="0"
                max="100000"
                value={settings?.keep_image_count ?? 0}
                onChange={(e) => {
                  const value = parseInt(e.target.value) || 0;
                  // 如果输入的值在 1-7 之间，自动设为 8
                  const normalizedValue = value > 0 && value < 8 ? 8 : value;
                  handleChange("keep_image_count", normalizedValue);
                }}
                className="settings-input"
              />
            </label>
          </div>

          <div className="settings-section">
            <div className="settings-label">{t("saveDirectory")}:</div>
            <div className="settings-dir-row">
              <div
                className="settings-dir-info"
                title={
                  settings?.save_directory ??
                  (defaultDir ? defaultDir : t("loading"))
                }
              >
                {settings?.save_directory ??
                  (defaultDir ? defaultDir : t("loading"))}
              </div>
              <button
                onClick={handleSelectFolder}
                className="btn btn-secondary btn-small"
                type="button"
              >
                {t("selectFolder")}
              </button>
            </div>
            {settings?.save_directory &&
              settings.save_directory !== defaultDir && (
                <button
                  onClick={() => handleChange("save_directory", null)}
                  className="btn btn-link btn-small"
                  type="button"
                >
                  {t("restoreDefault")}
                </button>
              )}
          </div>
        </div>
      </div>
    </div>
  );
}
