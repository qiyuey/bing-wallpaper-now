import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useBingWallpapers } from "./useBingWallpapers";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core");

// Mock event listener
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

describe("useBingWallpapers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default successful mock
    vi.mocked(invoke).mockResolvedValue([]);
  });

  it("should initialize with default values", () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(result.current.localWallpapers).toEqual([]);
    expect(result.current.loading).toBe(true);
  });

  it("should fetch local wallpapers on mount", async () => {
    const mockWallpapers = [
      {
        start_date: "20240101",
        title: "Test Wallpaper",
        copyright: "Test Copyright",
        file_path: "/path/to/wallpaper.jpg",
        url: "https://example.com/wallpaper.jpg",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapers);

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.localWallpapers).toEqual(mockWallpapers);
    expect(result.current.error).toBeNull();
  });

  it("should handle fetch errors", async () => {
    const errorMessage = "Failed to fetch wallpapers";
    vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe(`Error: ${errorMessage}`);
    expect(result.current.localWallpapers).toEqual([]);
  });

  it("should expose setDesktopWallpaper function", () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(typeof result.current.setDesktopWallpaper).toBe("function");
  });

  it("should expose forceUpdate function", () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(typeof result.current.forceUpdate).toBe("function");
  });

  it("should expose cleanupWallpapers function", () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(typeof result.current.cleanupWallpapers).toBe("function");
  });

  it("should expose fetchLocalWallpapers function", () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(typeof result.current.fetchLocalWallpapers).toBe("function");
  });
});
