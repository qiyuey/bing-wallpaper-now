import { useState, useEffect, useRef } from "react";
import { getVersion } from "@tauri-apps/api/app";
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

const CHECK_TIMEOUT_MS = 15_000;
const LATEST_JSON_URL =
  "https://github.com/qiyuey/bing-wallpaper-now/releases/latest/download/latest.json";

type ManualCheckFallbackResult = "no-update" | "update-available" | "unknown";

interface LatestJson {
  version?: string;
}

function parseVersion(version: string) {
  const [core, preRelease] = version.replace(/^v/i, "").split("-", 2);
  return {
    numbers: core.split(".").map((part) => Number.parseInt(part, 10) || 0),
    preRelease,
  };
}

function compareVersions(a: string, b: string): number {
  const parsedA = parseVersion(a);
  const parsedB = parseVersion(b);
  const maxLength = Math.max(parsedA.numbers.length, parsedB.numbers.length);

  for (let i = 0; i < maxLength; i += 1) {
    const diff = (parsedA.numbers[i] ?? 0) - (parsedB.numbers[i] ?? 0);
    if (diff !== 0) return diff;
  }

  if (parsedA.preRelease && !parsedB.preRelease) return -1;
  if (!parsedA.preRelease && parsedB.preRelease) return 1;
  if (!parsedA.preRelease && !parsedB.preRelease) return 0;
  return parsedA.preRelease.localeCompare(parsedB.preRelease);
}

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = window.setTimeout(
      () => reject(new Error(`Update check timed out after ${ms}ms`)),
      ms,
    );
    promise.then(resolve, reject).finally(() => window.clearTimeout(timer));
  });
}

async function checkLatestJsonFallback(): Promise<ManualCheckFallbackResult> {
  try {
    const [currentVersion, response] = await Promise.all([
      getVersion(),
      window.fetch(LATEST_JSON_URL, { cache: "no-store" }),
    ]);

    if (!response.ok) return "unknown";

    const latest = (await response.json()) as LatestJson;
    if (!latest.version) return "unknown";

    return compareVersions(latest.version, currentVersion) > 0
      ? "update-available"
      : "no-update";
  } catch (err) {
    console.error("Failed to check latest.json fallback:", err);
    return "unknown";
  }
}

export function useUpdateCheck() {
  const { t } = useI18n();
  const tRef = useRef(t);
  useEffect(() => {
    tRef.current = t;
  });

  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const pendingCheckRef = useRef<Promise<UpdateInfo | null> | null>(null);

  const performCheckRef = useRef(
    async (options: { showNoUpdate: boolean }): Promise<UpdateInfo | null> => {
      if (pendingCheckRef.current) {
        await pendingCheckRef.current.catch(() => {});
      }

      const promise = (async (): Promise<UpdateInfo | null> => {
        const currentT = tRef.current;
        try {
          const update = await withTimeout(
            check({ timeout: 10000 }),
            CHECK_TIMEOUT_MS,
          );

          if (!update) {
            if (options.showNoUpdate) {
              showSystemNotification(
                currentT("checkForUpdates"),
                currentT("noUpdateAvailable"),
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
              currentT("checkForUpdates"),
              currentT("noUpdateAvailable"),
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
            const fallbackResult = await checkLatestJsonFallback();
            showSystemNotification(
              currentT("checkForUpdates"),
              currentT(
                fallbackResult === "no-update"
                  ? "noUpdateAvailable"
                  : "updateCheckFailed",
              ),
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
  );

  // Listen for tray-triggered manual update check event.
  // Uses ref pattern (like useTrayEvents) so the listener is registered once
  // and survives re-renders caused by i18n loading transitions.
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen("tray-check-updates", async () => {
          const info = await performCheckRef.current({
            showNoUpdate: true,
          });
          if (info && mounted) {
            try {
              await invoke("show_main_window");
            } catch (err) {
              console.error("Failed to show main window for update:", err);
            }
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
  }, []);

  // Auto-check on startup after 60s delay (silent -- no notification on "no update")
  useEffect(() => {
    let mounted = true;

    const timeoutId = window.setTimeout(async () => {
      if (!mounted) return;

      const info = await performCheckRef.current({ showNoUpdate: false });
      if (info && mounted) {
        setUpdateInfo(info);
      }
    }, 60000);

    return () => {
      mounted = false;
      window.clearTimeout(timeoutId);
    };
  }, []);

  return { updateInfo, setUpdateInfo };
}
