import { useI18n } from "../i18n/I18nContext";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { showSystemNotification } from "../utils/notification";

interface UpdateDialogProps {
  version: string;
  releaseUrl: string;
  onClose: () => void;
  onIgnore: () => void;
}

export function UpdateDialog({
  version,
  releaseUrl,
  onClose,
  onIgnore,
}: UpdateDialogProps) {
  const { t } = useI18n();

  const handleUpdate = async () => {
    try {
      await openUrl(releaseUrl);
      onClose();
    } catch (err) {
      console.error("Failed to open release URL:", err);
      await showSystemNotification(t("error"), String(err));
      onClose();
    }
  };

  const handleIgnore = async () => {
    try {
      await invoke("add_ignored_update_version", { version });
      onIgnore();
      onClose();
    } catch (err) {
      console.error("Failed to ignore version:", err);
      await showSystemNotification(t("error"), String(err));
      onClose();
    }
  };

  return (
    <div className="settings-overlay">
      <div className="settings-modal" style={{ maxWidth: "500px" }}>
        <div className="settings-header">
          <div className="settings-header-left">
            <h2>{t("updateAvailable")}</h2>
          </div>
          <button onClick={onClose} className="btn-close">
            ×
          </button>
        </div>

        <div className="settings-body">
          <div style={{ marginBottom: "1rem" }}>
            <p>{t("updateAvailableMessage").replace("{version}", version)}</p>
          </div>

          <div
            style={{
              display: "flex",
              gap: "0.5rem",
              justifyContent: "flex-end",
            }}
          >
            <button
              onClick={handleIgnore}
              className="btn btn-secondary btn-small"
              type="button"
            >
              {t("ignoreThisVersion")}
            </button>
            <button
              onClick={handleUpdate}
              className="btn btn-primary btn-small"
              type="button"
            >
              {t("goToUpdate")}
            </button>
            <button
              onClick={onClose}
              className="btn btn-secondary btn-small"
              type="button"
            >
              {t("close")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
