import { useState, useEffect } from "react";
import { AppSettings } from "../types";
import { useSettings } from "../hooks/useSettings";
import { useTheme, Theme } from "../contexts/ThemeContext";
import { useI18n } from "../i18n/I18nContext";
import { open } from "@tauri-apps/plugin-dialog";
import { useVersionCheck } from "../hooks/useVersionCheck";
import { openUrl } from "@tauri-apps/plugin-opener";

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
  const { checking, result, error, checkForUpdates } = useVersionCheck();

  const [defaultDir, setDefaultDir] = useState<string>("");

  useEffect(() => {
    getDefaultDirectory()
      .then((dir) => {
        if (dir) setDefaultDir(dir);
      })
      .catch((err) => {
        // 静默处理错误，不影响组件渲染
        console.error("Failed to get default directory:", err);
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

  const handleDownloadUpdate = async () => {
    if (result?.release_url) {
      try {
        await openUrl(result.release_url);
      } catch (err) {
        console.error("Failed to open release URL:", err);
        alert(t("updateCheckFailed"));
      }
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

          <div className="settings-section">
            <div className="settings-label">{t("checkForUpdates")}:</div>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                gap: "0.5rem",
              }}
            >
              <button
                onClick={checkForUpdates}
                className="btn btn-secondary btn-small"
                type="button"
                disabled={checking}
              >
                {checking ? t("checkingForUpdates") : t("checkForUpdates")}
              </button>
              {result && (
                <div
                  style={{
                    fontSize: "0.875rem",
                    padding: "0.5rem",
                    borderRadius: "6px",
                    backgroundColor: result.has_update
                      ? "var(--color-update-bg, rgba(59, 130, 246, 0.1))"
                      : "var(--color-success-bg, rgba(34, 197, 94, 0.1))",
                    color: result.has_update
                      ? "var(--color-update-text, rgb(59, 130, 246))"
                      : "var(--color-success-text, rgb(34, 197, 94))",
                  }}
                >
                  {result.has_update ? (
                    <div>
                      <div style={{ marginBottom: "0.5rem" }}>
                        {t("updateAvailable")}
                      </div>
                      <div
                        style={{
                          marginBottom: "0.5rem",
                          fontSize: "0.8125rem",
                        }}
                      >
                        {t("updateAvailableHint").replace(
                          "{version}",
                          result.latest_version || "",
                        )}
                      </div>
                      <button
                        onClick={handleDownloadUpdate}
                        className="btn btn-primary btn-small"
                        type="button"
                        style={{ width: "100%" }}
                      >
                        {t("downloadUpdate")}
                      </button>
                    </div>
                  ) : (
                    <div>{t("noUpdateAvailable")}</div>
                  )}
                </div>
              )}
              {error && (
                <div
                  style={{
                    fontSize: "0.875rem",
                    padding: "0.5rem",
                    borderRadius: "6px",
                    backgroundColor:
                      "var(--color-error-bg, rgba(239, 68, 68, 0.1))",
                    color: "var(--color-error-text, rgb(239, 68, 68))",
                  }}
                >
                  {t("updateCheckFailed")}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
