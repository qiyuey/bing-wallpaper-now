import { useState, useCallback, useRef } from "react";
import { useI18n } from "../i18n/I18nContext";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { Update, DownloadEvent } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { showSystemNotification } from "../utils/notification";
import { cn } from "../utils/cn";
import styles from "./UpdateDialog.module.css";
import modalStyles from "../styles/modal.module.css";
import btnStyles from "../styles/buttons.module.css";

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
    <div className={modalStyles.overlay}>
      <div className={modalStyles.modal} style={{ maxWidth: "580px" }}>
        <div className={modalStyles.header}>
          <div className={modalStyles.headerLeft}>
            <h2>{t("updateAvailable")}</h2>
          </div>
          <button
            onClick={downloading ? handleCancelDownload : onClose}
            className={btnStyles.btnClose}
            aria-label={t("close")}
            disabled={downloadComplete}
          >
            ×
          </button>
        </div>

        <div className={modalStyles.body}>
          <div className={styles.message}>
            <p>{t("updateAvailableMessage").replace("{version}", version)}</p>
          </div>

          {statusText && (
            <div className={styles.statusBlock}>
              <p className={styles.statusText}>{statusText}</p>
              {downloading && !downloadComplete && (
                <div className={styles.progressTrack}>
                  <div
                    className={cn(
                      styles.progressFill,
                      progressPercent == null &&
                        styles.progressFillIndeterminate,
                    )}
                    style={{
                      width:
                        progressPercent != null ? `${progressPercent}%` : "40%",
                    }}
                  />
                </div>
              )}
            </div>
          )}

          {downloadError && (
            <div className={styles.errorBox}>
              {t("downloadFailed")}: {downloadError}
            </div>
          )}

          <div className={styles.actions}>
            {downloading ? (
              <button
                onClick={handleCancelDownload}
                className={cn(
                  btnStyles.btn,
                  btnStyles.btnSecondary,
                  btnStyles.btnSmall,
                )}
                type="button"
                disabled={downloadComplete}
              >
                {t("cancelDownload")}
              </button>
            ) : (
              <button
                onClick={handleIgnore}
                className={cn(
                  btnStyles.btn,
                  btnStyles.btnSecondary,
                  btnStyles.btnSmall,
                )}
                type="button"
              >
                {t("ignoreThisVersion")}
              </button>
            )}
            <button
              onClick={handleViewRelease}
              className={cn(
                btnStyles.btn,
                btnStyles.btnSecondary,
                btnStyles.btnSmall,
              )}
              type="button"
              disabled={downloading}
            >
              {t("viewRelease")}
            </button>
            <button
              onClick={handleDownloadAndInstall}
              className={cn(
                btnStyles.btn,
                btnStyles.btnPrimary,
                btnStyles.btnSmall,
              )}
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
