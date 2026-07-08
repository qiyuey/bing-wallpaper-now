import type { Page } from "@playwright/test";

export interface TauriCall {
  cmd: string;
  args: unknown;
}

export async function installTauriMock(page: Page) {
  await page.addInitScript(() => {
    type TauriCall = {
      cmd: string;
      args: unknown;
    };

    type TauriEvent = {
      id: number;
      event: string;
      payload: unknown;
    };

    type Callback = (payload: unknown) => void;

    interface E2EState {
      calls: TauriCall[];
      events: TauriEvent[];
      notifications: Array<{ title: string; body?: string }>;
    }

    interface TauriInternals {
      transformCallback: (callback: Callback, once?: boolean) => number;
      unregisterCallback: (id: number) => void;
      runCallback: (id: number, payload: unknown) => void;
      invoke: (cmd: string, args?: unknown) => Promise<unknown>;
      convertFileSrc: (filePath: string) => string;
    }

    interface TauriEventInternals {
      unregisterListener: (event: string, eventId: number) => void;
    }

    const e2eState: E2EState = {
      calls: [],
      events: [],
      notifications: [],
    };

    let nextCallbackId = 1;
    let nextEventId = 1;
    let settings = {
      auto_update: true,
      save_directory: null,
      launch_at_startup: false,
      theme: "system",
      language: "en-US",
      resolved_language: "en-US",
      mkt: "zh-CN",
    };
    const callbacks = new Map<number, { callback: Callback; once: boolean }>();
    const eventCallbacks = new Map<
      number,
      { event: string; callbackId: number }
    >();

    function makeWallpaperSvg(filePath: string) {
      const label = filePath.includes("20260707") ? "E2E Desert" : "E2E Alpine";
      const accent = filePath.includes("20260707") ? "#c8833f" : "#2d8a8a";
      const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="675" viewBox="0 0 1200 675">
        <defs>
          <linearGradient id="g" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0" stop-color="#17324d"/>
            <stop offset="0.55" stop-color="${accent}"/>
            <stop offset="1" stop-color="#f2d096"/>
          </linearGradient>
        </defs>
        <rect width="1200" height="675" fill="url(#g)"/>
        <circle cx="930" cy="170" r="88" fill="#f6d36d" opacity="0.9"/>
        <path d="M0 500 L250 330 L430 485 L610 265 L815 505 L1010 360 L1200 505 L1200 675 L0 675 Z" fill="#0d2233" opacity="0.78"/>
        <path d="M0 575 C220 530 410 650 655 575 C860 512 1015 600 1200 548 L1200 675 L0 675 Z" fill="#faf6e9" opacity="0.82"/>
        <text x="64" y="108" font-family="Arial, sans-serif" font-size="48" font-weight="700" fill="#ffffff">${label}</text>
      </svg>`;

      return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`;
    }

    function resolveLanguage(language: string) {
      return language === "zh-CN" ? "zh-CN" : "en-US";
    }

    function record(cmd: string, args: unknown) {
      e2eState.calls.push({ cmd, args: args ?? null });
    }

    function runStoredCallback(id: number, payload: unknown) {
      const entry = callbacks.get(id);
      if (!entry) return;

      entry.callback(payload);
      if (entry.once) {
        callbacks.delete(id);
      }
    }

    function invoke(cmd: string, args?: unknown): Promise<unknown> {
      record(cmd, args);

      switch (cmd) {
        case "get_settings":
          return Promise.resolve({ ...settings });
        case "update_settings": {
          const newSettings = (
            args as { newSettings?: Partial<typeof settings> }
          )?.newSettings;
          if (newSettings) {
            settings = {
              ...settings,
              ...newSettings,
              resolved_language: resolveLanguage(
                newSettings.language ?? settings.language,
              ),
            };
          }
          return Promise.resolve(null);
        }
        case "get_local_wallpapers":
          return Promise.resolve([
            {
              t: "E2E Alpine Lake",
              c: "Alpine lake in the Dolomites (Italy)",
              l: "https://www.bing.com/search?q=e2e-alpine-lake",
              d: "20260708",
              u: "/th?id=OHR.E2EAlpine",
            },
            {
              t: "E2E Desert Dunes",
              c: "Desert dunes at sunrise (Namibia)",
              l: "https://www.bing.com/search?q=e2e-desert-dunes",
              d: "20260707",
              u: "/th?id=OHR.E2EDesert",
            },
          ]);
        case "get_wallpaper_directory":
        case "get_default_wallpaper_directory":
          return Promise.resolve("/tmp/bing-wallpaper-now-e2e");
        case "get_current_wallpaper_path":
          return Promise.resolve(null);
        case "get_last_update_time":
          return Promise.resolve("2026-07-08 15:00:00");
        case "get_update_in_progress":
          return Promise.resolve(false);
        case "get_market_status":
          return Promise.resolve({
            requested_mkt: settings.mkt,
            effective_mkt: settings.mkt,
            is_mismatch: false,
          });
        case "get_supported_mkts":
          return Promise.resolve([
            {
              region: "asia_pacific",
              markets: [
                { code: "zh-CN", label: "China" },
                { code: "ja-JP", label: "Japan" },
              ],
            },
            {
              region: "americas",
              markets: [{ code: "en-US", label: "United States" }],
            },
          ]);
        case "get_wallpaper_data_stats":
          return Promise.resolve({
            count: 2,
            earliest_end_date: "20260707",
            latest_end_date: "20260708",
          });
        case "get_screen_orientations":
          return Promise.resolve([
            {
              screen_index: 0,
              is_portrait: false,
              width: 1280,
              height: 800,
            },
          ]);
        case "force_update":
        case "set_desktop_wallpaper":
        case "ensure_wallpaper_directory_exists":
        case "show_main_window":
        case "mark_frontend_ready":
        case "report_frontend_error":
        case "add_ignored_update_version":
        case "import_wallpapers":
        case "export_wallpapers":
        case "plugin:opener|open_path":
        case "plugin:opener|open_url":
        case "plugin:dialog|message":
        case "plugin:process|restart":
          return Promise.resolve(null);
        case "is_version_ignored":
        case "plugin:notification|is_permission_granted":
          return Promise.resolve(false);
        case "plugin:dialog|open":
        case "plugin:updater|check":
          return Promise.resolve(null);
        case "plugin:app|version":
          return Promise.resolve("1.5.3");
        case "plugin:event|listen": {
          const eventId = nextEventId;
          nextEventId += 1;
          const listenArgs = args as { event?: string; handler?: number };
          eventCallbacks.set(eventId, {
            event: listenArgs.event ?? "unknown",
            callbackId: listenArgs.handler ?? -1,
          });
          return Promise.resolve(eventId);
        }
        case "plugin:event|unlisten": {
          const eventId = (args as { eventId?: number })?.eventId;
          if (typeof eventId === "number") {
            eventCallbacks.delete(eventId);
          }
          return Promise.resolve(null);
        }
        case "plugin:event|emit":
        case "plugin:event|emit_to": {
          const event = (args as { event?: string })?.event ?? "unknown";
          const payload = (args as { payload?: unknown })?.payload;
          e2eState.events.push({ id: nextEventId++, event, payload });
          for (const listener of eventCallbacks.values()) {
            if (listener.event === event) {
              runStoredCallback(listener.callbackId, {
                id: nextEventId,
                event,
                payload,
              });
            }
          }
          return Promise.resolve(null);
        }
        default:
          throw new Error(`Unhandled Tauri E2E command: ${cmd}`);
      }
    }

    const internals: TauriInternals = {
      transformCallback(callback, once = false) {
        const id = nextCallbackId;
        nextCallbackId += 1;
        callbacks.set(id, { callback, once });
        return id;
      },
      unregisterCallback(id) {
        callbacks.delete(id);
      },
      runCallback(id, payload) {
        runStoredCallback(id, payload);
      },
      invoke,
      convertFileSrc(filePath) {
        return makeWallpaperSvg(filePath);
      },
    };

    const eventInternals: TauriEventInternals = {
      unregisterListener(_event, eventId) {
        eventCallbacks.delete(eventId);
      },
    };

    Object.defineProperty(window, "__BWN_E2E__", {
      configurable: true,
      value: e2eState,
    });
    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: internals,
    });
    Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
      configurable: true,
      value: eventInternals,
    });

    class MockNotification {
      static permission = "granted";

      static requestPermission() {
        return Promise.resolve("granted");
      }

      title: string;
      body?: string;

      constructor(title: string, options?: NotificationOptions) {
        this.title = title;
        this.body = options?.body;
        e2eState.notifications.push({ title, body: options?.body });
      }
    }

    try {
      Object.defineProperty(window, "Notification", {
        configurable: true,
        value: MockNotification,
      });
    } catch {
      // Browser notification availability is not relevant for these tests.
    }
  });
}

export async function getTauriCalls(page: Page): Promise<TauriCall[]> {
  return page.evaluate(() => window.__BWN_E2E__?.calls ?? []);
}

declare global {
  interface Window {
    __BWN_E2E__?: {
      calls: TauriCall[];
    };
  }
}
