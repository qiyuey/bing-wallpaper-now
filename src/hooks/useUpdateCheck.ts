import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { createSafeUnlisten } from "../utils/eventListener";
import { showSystemNotification } from "../utils/notification";
import { EVENTS } from "../config/ui";
import { useI18n } from "../i18n/I18nContext";
import { VersionCheckResult } from "./useVersionCheck";

interface UpdateInfo {
  version: string;
  releaseUrl: string;
}

export function useUpdateCheck() {
  const { t } = useI18n();
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);

  // Listen for tray-triggered update check events
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;
    let unlistenNoUpdate: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen<VersionCheckResult>(
          EVENTS.CHECK_UPDATES_RESULT,
          (event) => {
            const result = event.payload;
            if (
              result.has_update &&
              result.latest_version &&
              result.release_url &&
              result.platform_available
            ) {
              setUpdateInfo({
                version: result.latest_version,
                releaseUrl: result.release_url,
              });
            }
          },
        );
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        const unlistenNoUpdateFn = await listen(
          EVENTS.CHECK_UPDATES_NO_UPDATE,
          () => {
            showSystemNotification(
              t("checkForUpdates"),
              t("noUpdateAvailable"),
            );
          },
        );
        const safeUnlistenNoUpdate = createSafeUnlisten(unlistenNoUpdateFn);

        if (mounted) {
          unlisten = safeUnlisten;
          unlistenNoUpdate = safeUnlistenNoUpdate;
        } else {
          safeUnlisten();
          safeUnlistenNoUpdate();
        }
      } catch (e) {
        console.error("Failed to bind check-updates events:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
      unlistenNoUpdate?.();
    };
  }, [t]);

  // Auto-check on startup after 60s delay (silent — no toast on "no update")
  useEffect(() => {
    let mounted = true;

    const timeoutId = window.setTimeout(async () => {
      try {
        if (!mounted) return;

        const result = await invoke<VersionCheckResult>("check_for_updates");

        if (!mounted) return;

        if (
          result.has_update &&
          result.latest_version &&
          result.release_url &&
          result.platform_available
        ) {
          const isIgnored = await invoke<boolean>("is_version_ignored", {
            version: result.latest_version,
          });

          if (!isIgnored && mounted) {
            setUpdateInfo({
              version: result.latest_version,
              releaseUrl: result.release_url,
            });
          }
        }
      } catch (err) {
        console.error("Failed to check for updates:", err);
      }
    }, 60000);

    return () => {
      mounted = false;
      window.clearTimeout(timeoutId);
    };
  }, []);

  return { updateInfo, setUpdateInfo };
}
