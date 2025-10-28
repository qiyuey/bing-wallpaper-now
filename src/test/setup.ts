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
    invoke: vi.fn((cmd: string, _args?: unknown) => {
      // Provide safe defaults for tests
      if (cmd === "get_settings") {
        return Promise.resolve({
          theme: "system",
          auto_update: true,
          save_directory: null,
          keep_image_count: 30,
          launch_at_startup: false,
        });
      }
      if (cmd === "update_settings") {
        return Promise.resolve(null);
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

// Mock @tauri-apps/api/core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}));

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

// Mock @tauri-apps/api/event
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}));

// Extend expect matchers
expect.extend({});
