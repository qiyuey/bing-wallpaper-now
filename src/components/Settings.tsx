import { useState, useEffect } from "react";
import { AppSettings } from "../types";
import { useSettings } from "../hooks/useSettings";
import { useTheme, Theme } from "../contexts/ThemeContext";
import { open } from "@tauri-apps/plugin-dialog";

interface SettingsProps {
  onClose: () => void;
  version?: string;
}

export function Settings({ onClose, version }: SettingsProps) {
  const { settings, loading, updateSettings, getDefaultDirectory } =
    useSettings();
  const { applyThemeToUI } = useTheme();

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
    } catch (err) {
      console.error("Update settings error:", err);
      alert("保存设置失败: " + err);
    }
  };

  const handleSelectFolder = async () => {
    if (!settings) return;

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: settings.save_directory || defaultDir,
        title: "选择壁纸保存目录",
      });

      if (selected && typeof selected === "string") {
        await handleChange("save_directory", selected);
      }
    } catch (err) {
      console.error("Failed to select folder:", err);
      alert("选择文件夹失败: " + String(err));
    }
  };

  if (loading && !settings) {
    return <div className="settings-loading">加载设置中...</div>;
  }

  return (
    <div className="settings-overlay">
      <div className="settings-modal">
        <div className="settings-header">
          <div className="settings-header-left">
            <h2>设置</h2>
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
              <span>开机自启动</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={settings?.auto_update ?? true}
                onChange={(e) => handleChange("auto_update", e.target.checked)}
              />
              <span>自动更新壁纸</span>
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label">主题:</label>
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
                <span>跟随系统</span>
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
                <span>浅色模式</span>
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
                <span>深色模式</span>
              </label>
            </div>
          </div>

          <div className="settings-section">
            <label className="settings-label">
              保留壁纸数量:
              <input
                type="number"
                min="8"
                max="10000"
                value={settings?.keep_image_count ?? 8}
                onChange={(e) =>
                  handleChange(
                    "keep_image_count",
                    parseInt(e.target.value) || 8,
                  )
                }
                className="settings-input"
              />
            </label>
          </div>

          <div className="settings-section">
            <div className="settings-label">保存目录:</div>
            <div className="settings-dir-row">
              <div
                className="settings-dir-info"
                title={
                  settings?.save_directory ??
                  (defaultDir ? defaultDir : "加载中...")
                }
              >
                {settings?.save_directory ??
                  (defaultDir ? defaultDir : "加载中...")}
              </div>
              <button
                onClick={handleSelectFolder}
                className="btn btn-secondary btn-small"
                type="button"
              >
                选择文件夹
              </button>
            </div>
            {settings?.save_directory &&
              settings.save_directory !== defaultDir && (
                <button
                  onClick={() => handleChange("save_directory", null)}
                  className="btn btn-link btn-small"
                  type="button"
                >
                  恢复默认目录
                </button>
              )}
          </div>
        </div>
      </div>
    </div>
  );
}
