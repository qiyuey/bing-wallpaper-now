import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useBingWallpapers } from "./useBingWallpapers";
import { LocalWallpaperRaw } from "../types";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core");

describe("useBingWallpapers", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default successful mock
    vi.mocked(invoke).mockResolvedValue([]);
  });

  it("should initialize with default values", async () => {
    const { result } = renderHook(() => useBingWallpapers());

    expect(result.current.localWallpapers).toEqual([]);
    expect(result.current.loading).toBe(true);

    // Wait for initial loading to complete
    await waitFor(() => expect(result.current.loading).toBe(false));
  });

  it("should fetch local wallpapers on mount", async () => {
    const mockWallpapersRaw: LocalWallpaperRaw[] = [
      {
        t: "Test Wallpaper",
        c: "Test Copyright",
        l: "https://example.com/link",
        d: "20240102",
        u: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapersRaw);

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    // 验证转换后的数据
    expect(result.current.localWallpapers).toEqual([
      {
        title: "Test Wallpaper",
        copyright: "Test Copyright",
        copyright_link: "https://example.com/link",
        end_date: "20240102",
        urlbase: "/th?id=OHR.Test",
      },
    ]);
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

  it("should expose setDesktopWallpaper function", async () => {
    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(typeof result.current.setDesktopWallpaper).toBe("function");
  });

  it("should expose forceUpdate function", async () => {
    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(typeof result.current.forceUpdate).toBe("function");
  });

  it("should expose fetchLocalWallpapers function", async () => {
    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => expect(result.current.loading).toBe(false));

    expect(typeof result.current.fetchLocalWallpapers).toBe("function");
  });

  it("should call setDesktopWallpaper successfully", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "set_desktop_wallpaper") {
        return Promise.resolve(undefined);
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve([]);
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await result.current.setDesktopWallpaper("/path/to/wallpaper.jpg");

    expect(invoke).toHaveBeenCalledWith("set_desktop_wallpaper", {
      filePath: "/path/to/wallpaper.jpg",
    });
  });

  it("should call forceUpdate and refresh wallpapers", async () => {
    const mockWallpapersRaw: LocalWallpaperRaw[] = [
      {
        t: "Test Wallpaper",
        c: "Test Copyright",
        l: "https://example.com/link",
        d: "20240102",
        u: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "force_update") {
        return Promise.resolve(undefined);
      }
      return Promise.resolve(mockWallpapersRaw);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.forceUpdate();
    });

    expect(invoke).toHaveBeenCalledWith("force_update");
  });

  it("should fetch lastUpdateTime from backend", async () => {
    const mockTime = "2024-01-01 12:00:00";

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_last_update_time") {
        return Promise.resolve(mockTime);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "zh-CN",
          effective_mkt: "zh-CN",
          is_mismatch: false,
        });
      }
      return Promise.resolve([]);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.lastUpdateTime).toBe(mockTime);
    });
  });

  it("should fetch effectiveMktLabel from backend with label", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "en-US",
          effective_mkt: "en-US",
          is_mismatch: false,
        });
      }
      if (cmd === "get_supported_mkts") {
        return Promise.resolve([
          {
            region: "americas",
            markets: [{ code: "en-US", label: "United States" }],
          },
        ]);
      }
      return Promise.resolve([]);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.effectiveMktLabel).toBe("United States");
    });
  });

  it("should handle fetchLocalWallpapers with showLoading parameter", async () => {
    const mockWallpapersRaw: LocalWallpaperRaw[] = [
      {
        t: "Test",
        c: "Test",
        l: "https://example.com/link",
        d: "20240102",
        u: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapersRaw);

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.fetchLocalWallpapers(true);
    });

    expect(result.current.localWallpapers).toEqual([
      {
        title: "Test",
        copyright: "Test",
        copyright_link: "https://example.com/link",
        end_date: "20240102",
        urlbase: "/th?id=OHR.Test",
      },
    ]);
  });

  it("should poll status when page becomes visible", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve([]);
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      if (cmd === "get_update_in_progress") {
        return Promise.resolve(false);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "zh-CN",
          effective_mkt: "zh-CN",
          is_mismatch: false,
        });
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const callsBefore = vi.mocked(invoke).mock.calls.length;

    // Simulate visibility change to "visible" after some time
    // First make the time gap > 10s so pollWhenActive triggers
    vi.useFakeTimers();
    vi.advanceTimersByTime(11000);

    // Fire visibilitychange event
    Object.defineProperty(document, "visibilityState", {
      value: "visible",
      writable: true,
    });
    document.dispatchEvent(new globalThis.Event("visibilitychange"));

    // Wait for any async operations
    await vi.advanceTimersByTimeAsync(100);
    vi.useRealTimers();

    const callsAfter = vi.mocked(invoke).mock.calls.length;
    // Should have made additional invoke calls for polling
    expect(callsAfter).toBeGreaterThan(callsBefore);
  });

  it("should poll status when window gains focus", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve([]);
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      if (cmd === "get_update_in_progress") {
        return Promise.resolve(false);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "zh-CN",
          effective_mkt: "zh-CN",
          is_mismatch: false,
        });
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const callsBefore = vi.mocked(invoke).mock.calls.length;

    // Simulate time passing > 10s so pollWhenActive triggers
    vi.useFakeTimers();
    vi.advanceTimersByTime(11000);

    // Fire focus event
    window.dispatchEvent(new globalThis.Event("focus"));

    // Wait for any async operations
    await vi.advanceTimersByTimeAsync(100);
    vi.useRealTimers();

    const callsAfter = vi.mocked(invoke).mock.calls.length;
    expect(callsAfter).toBeGreaterThan(callsBefore);
  });

  it("should not update state if wallpapers data hasn't changed", async () => {
    const mockWallpapersRaw: LocalWallpaperRaw[] = [
      {
        t: "Test",
        c: "Test",
        l: "https://example.com/link",
        d: "20240102",
        u: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapersRaw);

    const { result, rerender } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const firstWallpapers = result.current.localWallpapers;

    // Fetch again with same data
    await act(async () => {
      await result.current.fetchLocalWallpapers(false);
    });

    rerender();

    // Should be the same reference (no state update)
    expect(result.current.localWallpapers).toBe(firstWallpapers);
  });
});
