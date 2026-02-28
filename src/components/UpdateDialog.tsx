import { useState, useCallback, useRef } from "react";
import { useI18n } from "../i18n/I18nContext";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { showSystemNotification } from "../utils/notification";

interface DownloadProgress {
  contentLength?: number;
  downloaded: number;
  percent: number | null;
}

interface UpdateDialogProps {
  version: string;
  body?: string;
  update: Update;
  onClose: () => void;
  onIgnore: () => void;
}

export function UpdateDialog({
  version,
  update,
  onClose,
  onIgnore,
}: UpdateDialogProps) {
  const { t } = useI18n();
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<DownloadProgress | null>(null);
  const [downloadError, setDownloadError] = useState<string | null>(null);
  const [downloadComplete, setDownloadComplete] = useState(false);
  const cancelledRef = useRef(false);

  const releaseUrl = `https://github.com/qiyuey/bing-wallpaper-now/releases/tag/${version}`;

  const handleViewRelease = async () => {
    try {
      await openUrl(releaseUrl);
    } catch (err) {
      console.error("Failed to open release URL:", err);
      await showSystemNotification(t("error"), String(err));
    }
  };

  const handleDownloadAndInstall = useCallback(async () => {
    setDownloading(true);
    setDownloadError(null);
    setProgress(null);
    cancelledRef.current = false;

    try {
      let downloaded = 0;
      let contentLength: number | undefined;

      await update.downloadAndInstall((event: DownloadEvent) => {
        switch (event.event) {
          case "Started":
            contentLength = event.data.contentLength;
            setProgress({ contentLength, downloaded: 0, percent: 0 });
            break;
          case "Progress":
            downloaded += event.data.chunkLength;
            setProgress({
              contentLength,
              downloaded,
              percent: contentLength
                ? (downloaded / contentLength) * 100
                : null,
            });
            break;
          case "Finished":
            setProgress((prev) => (prev ? { ...prev, percent: 100 } : prev));
            break;
        }
      });

      if (cancelledRef.current) return;

      setDownloadComplete(true);

      await relaunch();
    } catch (err) {
      if (cancelledRef.current) return;
      console.error("Failed to download/install update:", err);
      setDownloadError(String(err));
      setProgress(null);
      setDownloading(false);
    }
  }, [update]);

  const handleCancelDownload = useCallback(() => {
    cancelledRef.current = true;
    setDownloading(false);
    setProgress(null);
    onClose();
  }, [onClose]);

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

  const progressPercent =
    progress?.percent != null ? Math.round(progress.percent) : null;
  const statusText = downloadComplete
    ? t("downloadComplete")
    : progressPercent != null
      ? t("downloadProgress").replace("{percent}", String(progressPercent))
      : downloading
        ? t("downloading")
        : null;

  return (
    <div className="settings-overlay">
      <div className="settings-modal" style={{ maxWidth: "580px" }}>
        <div className="settings-header">
          <div className="settings-header-left">
            <h2>{t("updateAvailable")}</h2>
          </div>
          <button
            onClick={downloading ? handleCancelDownload : onClose}
            className="btn-close"
            aria-label={t("close")}
            disabled={downloadComplete}
          >
            ×
          </button>
        </div>

        <div className="settings-body">
          <div style={{ marginBottom: "1rem" }}>
            <p>{t("updateAvailableMessage").replace("{version}", version)}</p>
          </div>

          {statusText && (
            <div style={{ marginBottom: "1rem" }}>
              <p style={{ fontSize: "0.875rem", marginBottom: "0.5rem" }}>
                {statusText}
              </p>
              {downloading && !downloadComplete && (
                <div
                  style={{
                    width: "100%",
                    height: "6px",
                    backgroundColor: "var(--border-color, #e0e0e0)",
                    borderRadius: "3px",
                    overflow: "hidden",
                  }}
                >
                  <div
                    style={{
                      width:
                        progressPercent != null ? `${progressPercent}%` : "40%",
                      height: "100%",
                      backgroundColor: "var(--primary-color, #0078d4)",
                      borderRadius: "3px",
                      transition: "width 0.3s ease",
                      ...(progressPercent == null
                        ? {
                            animation:
                              "indeterminate-progress 1.5s infinite linear",
                          }
                        : {}),
                    }}
                  />
                </div>
              )}
            </div>
          )}

          {downloadError && (
            <div
              style={{
                marginBottom: "1rem",
                padding: "0.5rem",
                borderRadius: "4px",
                backgroundColor: "var(--error-bg, #fef2f2)",
                color: "var(--error-color, #dc2626)",
                fontSize: "0.875rem",
              }}
            >
              {t("downloadFailed")}: {downloadError}
            </div>
          )}

          <div
            style={{
              display: "flex",
              gap: "0.5rem",
              justifyContent: "flex-end",
              flexWrap: "wrap",
            }}
          >
            {downloading ? (
              <button
                onClick={handleCancelDownload}
                className="btn btn-secondary btn-small"
                type="button"
                disabled={downloadComplete}
              >
                {t("cancelDownload")}
              </button>
            ) : (
              <button
                onClick={handleIgnore}
                className="btn btn-secondary btn-small"
                type="button"
              >
                {t("ignoreThisVersion")}
              </button>
            )}
            <button
              onClick={handleViewRelease}
              className="btn btn-secondary btn-small"
              type="button"
              disabled={downloading}
            >
              {t("viewRelease")}
            </button>
            <button
              onClick={handleDownloadAndInstall}
              className="btn btn-primary btn-small"
              type="button"
              disabled={downloading}
            >
              {downloading ? t("downloading") : t("downloadAndInstall")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
