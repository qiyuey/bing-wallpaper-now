import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useBingWallpapers } from "./useBingWallpapers";

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
    const mockWallpapers = [
      {
        end_date: "20240102",
        title: "Test Wallpaper",
        copyright: "Test Copyright",
        copyright_link: "https://example.com/link",
        urlbase: "/th?id=OHR.Test",
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
    const mockWallpapers = [
      {
        end_date: "20240102",
        title: "Test Wallpaper",
        copyright: "Test Copyright",
        copyright_link: "https://example.com/link",
        urlbase: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "force_update") {
        return Promise.resolve(undefined);
      }
      return Promise.resolve(mockWallpapers);
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
      return Promise.resolve([]);
    });

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.lastUpdateTime).toBe(mockTime);
    });
  });

  it("should handle fetchLocalWallpapers with showLoading parameter", async () => {
    const mockWallpapers = [
      {
        id: "20240101",
        end_date: "20240102",
        title: "Test",
        copyright: "Test",
        copyright_link: "https://example.com/link",
        urlbase: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapers);

    const { result } = renderHook(() => useBingWallpapers());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    await act(async () => {
      await result.current.fetchLocalWallpapers(true);
    });

    expect(result.current.localWallpapers).toEqual(mockWallpapers);
  });

  it("should not update state if wallpapers data hasn't changed", async () => {
    const mockWallpapers = [
      {
        id: "20240101",
        end_date: "20240102",
        title: "Test",
        copyright: "Test",
        copyright_link: "https://example.com/link",
        urlbase: "/th?id=OHR.Test",
      },
    ];

    vi.mocked(invoke).mockResolvedValue(mockWallpapers);

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
