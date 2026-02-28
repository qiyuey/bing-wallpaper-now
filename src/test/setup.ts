import "@testing-library/jest-dom";
import { expect, afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";

// Cleanup after each test
afterEach(() => {
  cleanup();
});

// Mock Tauri APIs
global.window = Object.create(window);
Object.defineProperty(window, "__TAURI_INTERNALS__", {
  value: {
    transformCallback: vi.fn(<T>(callback: T) => callback),
    invoke: vi.fn((cmd: string, args?: unknown) => {
      // Provide safe defaults for tests
      if (cmd === "get_settings") {
        return Promise.resolve({
          theme: "system",
          auto_update: true,
          save_directory: null,
          keep_image_count: 30,
          launch_at_startup: false,
          language: "zh-CN", // Default to Chinese for tests
        });
      }
      if (cmd === "update_settings") {
        // Return the updated settings so the hook can update state
        const updateArgs = args as { newSettings?: unknown };
        if (updateArgs && updateArgs.newSettings) {
          return Promise.resolve(updateArgs.newSettings);
        }
        return Promise.resolve(null);
      }
      if (cmd === "get_default_wallpaper_directory") {
        return Promise.resolve("/Users/Test/Pictures/BingWallpapers");
      }
      return Promise.resolve(undefined);
    }),
    event: {
      // ensure event module finds expected internals
      registerListener: vi.fn(() => 1),
      unregisterListener: vi.fn((_id: number) => {}),
    },
    // Some versions access these at the root level, not under .event
    registerListener: vi.fn(() => 1),
    unregisterListener: vi.fn((_id: number) => {}),
  },
  writable: true,
});

// Mock __TAURI_EVENT_PLUGIN_INTERNALS__ for newer Tauri versions
Object.defineProperty(window, "__TAURI_EVENT_PLUGIN_INTERNALS__", {
  value: {
    registerListener: vi.fn(() => 1),
    unregisterListener: vi.fn((_event: string, _id: number) => {}),
  },
  writable: true,
});

// Mock @tauri-apps/api modules directly to avoid internal errors
vi.mock("@tauri-apps/api/core", () => {
  return {
    invoke: (cmd: string, args?: unknown) => {
      // Access __TAURI_INTERNALS__ from global context
      const globalWindow = global.window as {
        __TAURI_INTERNALS__?: {
          invoke: (cmd: string, args?: unknown) => Promise<unknown>;
        };
      };
      if (globalWindow && globalWindow.__TAURI_INTERNALS__) {
        return globalWindow.__TAURI_INTERNALS__.invoke(cmd, args);
      }
      return Promise.resolve(undefined);
    },
    convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
  };
});

vi.mock("@tauri-apps/api/event", () => {
  return {
    listen: vi.fn(async (_event: string, _cb: (..._a: unknown[]) => void) => {
      // Return a stable unlisten function
      return () => {};
    }),
    emit: vi.fn(async () => {}),
  };
});

// Mock window.matchMedia for theme detection
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // Deprecated
    removeListener: vi.fn(), // Deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock ResizeObserver for tests
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock @tauri-apps/plugin-opener
vi.mock("@tauri-apps/plugin-opener", () => ({
  open: vi.fn(),
  openPath: vi.fn(),
  openUrl: vi.fn(),
}));

// Mock @tauri-apps/plugin-dialog
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
  message: vi.fn(),
}));

// Extend expect matchers
expect.extend({});
