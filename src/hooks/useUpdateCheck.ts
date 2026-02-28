import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check, Update } from "@tauri-apps/plugin-updater";
import { createSafeUnlisten } from "../utils/eventListener";
import { showSystemNotification } from "../utils/notification";
import { useI18n } from "../i18n/I18nContext";

interface UpdateInfo {
  version: string;
  body?: string;
  update: Update;
}

export function useUpdateCheck() {
  const { t } = useI18n();
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const pendingCheckRef = useRef<Promise<UpdateInfo | null> | null>(null);

  const performCheck = useCallback(
    async (options: { showNoUpdate: boolean }): Promise<UpdateInfo | null> => {
      // If a check is already running, wait for it then run ours
      if (pendingCheckRef.current) {
        await pendingCheckRef.current.catch(() => {});
      }

      const promise = (async (): Promise<UpdateInfo | null> => {
        try {
          const update = await check({ timeout: 10000 });

          if (!update) {
            if (options.showNoUpdate) {
              showSystemNotification(
                t("checkForUpdates"),
                t("noUpdateAvailable"),
              );
            }
            return null;
          }

          const isIgnored = await invoke<boolean>("is_version_ignored", {
            version: update.version,
          });

          if (isIgnored && !options.showNoUpdate) {
            return null;
          }

          if (isIgnored && options.showNoUpdate) {
            showSystemNotification(
              t("checkForUpdates"),
              t("noUpdateAvailable"),
            );
            return null;
          }

          return {
            version: update.version,
            body: update.body,
            update,
          };
        } catch (err) {
          console.error("Failed to check for updates:", err);
          if (options.showNoUpdate) {
            showSystemNotification(
              t("checkForUpdates"),
              t("updateCheckFailed"),
            );
          }
          return null;
        } finally {
          pendingCheckRef.current = null;
        }
      })();

      pendingCheckRef.current = promise;
      return promise;
    },
    [t],
  );

  // Listen for tray-triggered manual update check event
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen("tray-check-updates", async () => {
          const info = await performCheck({ showNoUpdate: true });
          if (info && mounted) {
            setUpdateInfo(info);
          }
        });
        const safe = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safe;
        } else {
          safe();
        }
      } catch (e) {
        console.error("Failed to bind tray-check-updates event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [performCheck]);

  // Auto-check on startup after 60s delay (silent -- no notification on "no update")
  useEffect(() => {
    let mounted = true;

    const timeoutId = window.setTimeout(async () => {
      if (!mounted) return;

      const info = await performCheck({ showNoUpdate: false });
      if (info && mounted) {
        setUpdateInfo(info);
      }
    }, 60000);

    return () => {
      mounted = false;
      window.clearTimeout(timeoutId);
    };
  }, [performCheck]);

  return { updateInfo, setUpdateInfo };
}
