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
  const { theme, applyThemeOnly } = useTheme();

  const [formData, setFormData] = useState<AppSettings>({
    auto_update: true,
    save_directory: null,
    keep_image_count: 8,
    launch_at_startup: false,
    theme: "system",
  });

  const [defaultDir, setDefaultDir] = useState<string>("");
  const [localTheme, setLocalTheme] = useState<Theme>(theme);

  useEffect(() => {
    if (settings) {
      setFormData(settings);
      // 同步主题到本地状态
      if (settings.theme) {
        setLocalTheme(settings.theme as Theme);
      }
    }
  }, [settings]);

  useEffect(() => {
    getDefaultDirectory().then((dir) => {
      if (dir) setDefaultDir(dir);
    });
  }, [getDefaultDirectory]);

  const handleSave = async () => {
    try {
      // 将主题包含在设置中一起保存
      const settingsWithTheme: AppSettings = {
        auto_update: formData.auto_update,
        save_directory: formData.save_directory,
        keep_image_count: formData.keep_image_count,
        launch_at_startup: formData.launch_at_startup,
        theme: localTheme,
      };

      await updateSettings(settingsWithTheme);

      // 如果主题改变了，应用主题到 UI（不需要再保存，因为已经保存过了）
      if (localTheme !== theme) {
        applyThemeOnly(localTheme);
      }
      onClose();
    } catch (err) {
      console.error("Save error:", err);
      alert("保存失败: " + err);
    }
  };

  const handleChange = (
    field: keyof AppSettings,
    value: string | number | boolean | null,
  ) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        defaultPath: formData.save_directory || defaultDir,
        title: "选择壁纸保存目录",
      });

      if (selected && typeof selected === "string") {
        handleChange("save_directory", selected);
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
          <h2>设置</h2>
          <button onClick={onClose} className="btn-close">
            ×
          </button>
        </div>

        <div className="settings-body">
          <div className="settings-section">
            <label className="settings-label checkbox-label">
              <input
                type="checkbox"
                checked={formData.launch_at_startup}
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
                checked={formData.auto_update}
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
                  checked={localTheme === "system"}
                  onChange={(e) => setLocalTheme(e.target.value as Theme)}
                />
                <span>跟随系统</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="light"
                  checked={localTheme === "light"}
                  onChange={(e) => setLocalTheme(e.target.value as Theme)}
                />
                <span>浅色模式</span>
              </label>
              <label className="radio-option">
                <input
                  type="radio"
                  name="theme"
                  value="dark"
                  checked={localTheme === "dark"}
                  onChange={(e) => setLocalTheme(e.target.value as Theme)}
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
                value={formData.keep_image_count}
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
            <label className="settings-label">
              保存目录:
              <div className="settings-dir-row">
                <div className="settings-dir-info">
                  {formData.save_directory ??
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
            </label>
            {formData.save_directory &&
              formData.save_directory !== defaultDir && (
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

        <div className="settings-footer">
          <div style={{ flex: 1, display: "flex", alignItems: "center" }}>
            {version && <span className="settings-version">v{version}</span>}
          </div>
          <button onClick={onClose} className="btn btn-secondary">
            取消
          </button>
          <button
            onClick={handleSave}
            className="btn btn-primary"
            disabled={loading}
          >
            保存
          </button>
        </div>
      </div>
    </div>
  );
}
