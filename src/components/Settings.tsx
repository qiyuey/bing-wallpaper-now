import { useState, useEffect } from "react";
import { AppSettings } from "../types";
import { useSettings } from "../hooks/useSettings";
import { open } from "@tauri-apps/plugin-dialog";

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const { settings, loading, updateSettings, getDefaultDirectory } =
    useSettings();

  const [formData, setFormData] = useState<AppSettings>({
    auto_update: true,
    update_interval_hours: 24,
    save_directory: null,
    keep_image_count: 8,
    launch_at_startup: false,
  });

  const [defaultDir, setDefaultDir] = useState<string>("");

  useEffect(() => {
    if (settings) {
      setFormData(settings);
    }
  }, [settings]);

  useEffect(() => {
    getDefaultDirectory().then((dir) => {
      if (dir) setDefaultDir(dir);
    });
  }, []);

  const handleSave = async () => {
    try {
      await updateSettings(formData);
      alert("设置已保存");
      onClose();
    } catch (err) {
      alert("保存失败: " + err);
    }
  };

  const handleChange = (field: keyof AppSettings, value: any) => {
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
            <label className="settings-label">
              更新间隔(小时):
              <input
                type="number"
                min="1"
                max="168"
                value={formData.update_interval_hours}
                onChange={(e) =>
                  handleChange(
                    "update_interval_hours",
                    parseInt(e.target.value) || 24,
                  )
                }
                className="settings-input"
              />
            </label>
          </div>

          <div className="settings-section">
            <label className="settings-label">
              保留壁纸数量:
              <input
                type="number"
                min="8"
                max="200"
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
                  {formData.save_directory || defaultDir || "默认目录"}
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
