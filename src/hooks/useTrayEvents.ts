import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { createSafeUnlisten } from "../utils/eventListener";
import { EVENTS } from "../config/ui";

interface TrayEventCallbacks {
  onOpenSettings: () => void;
  onOpenAbout: () => void;
  onOpenFolder: () => void;
}

/**
 * Listens to tray-emitted events (open-settings, open-about, open-folder)
 * and dispatches to the provided callbacks.
 *
 * Uses refs so listeners are bound once and always call the latest callback.
 */
export function useTrayEvents(callbacks: TrayEventCallbacks) {
  const callbacksRef = useRef(callbacks);
  useEffect(() => {
    callbacksRef.current = callbacks;
  });

  useEffect(() => {
    let mounted = true;
    const unlisteners: (() => void)[] = [];

    const eventMap: [string, keyof TrayEventCallbacks][] = [
      [EVENTS.OPEN_SETTINGS, "onOpenSettings"],
      [EVENTS.OPEN_ABOUT, "onOpenAbout"],
      [EVENTS.OPEN_FOLDER, "onOpenFolder"],
    ];

    (async () => {
      for (const [event, key] of eventMap) {
        try {
          const unlistenFn = await listen(event, () => {
            callbacksRef.current[key]();
          });
          const safeUnlisten = createSafeUnlisten(unlistenFn);

          if (mounted) {
            unlisteners.push(safeUnlisten);
          } else {
            safeUnlisten();
            return;
          }
        } catch (e) {
          console.error(`Failed to bind ${event} event:`, e);
        }
      }
    })();

    return () => {
      mounted = false;
      unlisteners.forEach((fn) => fn());
    };
  }, []);
}
