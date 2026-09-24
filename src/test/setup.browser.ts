import "@testing-library/jest-dom/vitest";
import { afterEach, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import "../theme.css";
import "../styles/globals.css";

afterEach(cleanup);

// Only the native bridge is mocked; layout and browser APIs remain real.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string) =>
    command === "get_settings"
      ? { language: "zh-CN", resolved_language: "zh-CN" }
      : undefined,
  ),
  // WallpaperCard appends a retry query; the fragment keeps it out of the data.
  convertFileSrc: vi.fn(
    () =>
      "data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='8' height='8'%3E%3C/svg%3E#",
  ),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));
