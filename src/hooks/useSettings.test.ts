import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "./useSettings";
import { AppSettings } from "../types";

vi.mock("@tauri-apps/api/core");

describe("useSettings", () => {
  const mockSettings: AppSettings = {
    auto_update: true,
    update_interval: 24,
    keep_count: 30,
    wallpaper_directory: "C:\\Users\\Test\\Wallpapers",
    auto_apply_latest: true,
    autostart: false,
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should initialize with default values", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    // Initially loading will be true as useEffect starts fetching
    expect(result.current.settings).toBeNull();
    expect(result.current.error).toBeNull();

    // Wait for settings to load
    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });

  it("should fetch settings on mount", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(invoke).toHaveBeenCalledWith("get_settings");
    expect(result.current.settings).toEqual(mockSettings);
    expect(result.current.error).toBeNull();
  });

  it("should handle fetch errors", async () => {
    const errorMessage = "Failed to fetch settings";
    vi.mocked(invoke).mockRejectedValue(errorMessage);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    expect(result.current.error).toBe(errorMessage);
    expect(result.current.settings).toBeNull();
  });

  it("should update settings successfully", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const updatedSettings: AppSettings = {
      ...mockSettings,
      auto_update: false,
      keep_image_count: 50,
    };

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "update_settings") {
        return Promise.resolve(undefined);
      }
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    await result.current.updateSettings(updatedSettings);

    expect(invoke).toHaveBeenCalledWith("update_settings", {
      newSettings: updatedSettings,
    });

    await waitFor(() => {
      expect(result.current.settings).toEqual(updatedSettings);
      expect(result.current.error).toBeNull();
    });
  });

  it("should handle update settings errors", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const errorMessage = "Failed to update settings";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "update_settings") {
        return Promise.reject(errorMessage);
      }
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    const updatedSettings: AppSettings = {
      ...mockSettings,
      auto_update: false,
    };

    await expect(result.current.updateSettings(updatedSettings)).rejects.toBe(
      errorMessage,
    );

    await waitFor(() => {
      expect(result.current.error).toBe(errorMessage);
    });
  });

  it("should get default directory successfully", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const defaultDir = "C:\\Users\\Default\\Pictures\\Wallpapers";
    vi.mocked(invoke).mockResolvedValue(defaultDir);

    const directory = await result.current.getDefaultDirectory();

    expect(invoke).toHaveBeenCalledWith("get_default_wallpaper_directory");
    expect(directory).toBe(defaultDir);
  });

  it("should handle get default directory errors", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const errorMessage = "Failed to get default directory";
    vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

    const directory = await result.current.getDefaultDirectory();

    expect(directory).toBeNull();
  });

  it("should expose fetchSettings function", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    expect(typeof result.current.fetchSettings).toBe("function");
  });

  it("should expose updateSettings function", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    expect(typeof result.current.updateSettings).toBe("function");
  });

  it("should expose getDefaultDirectory function", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    expect(typeof result.current.getDefaultDirectory).toBe("function");
  });

  it("should manually fetch settings via fetchSettings", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    const newSettings: AppSettings = {
      ...mockSettings,
      auto_update: false,
    };

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(newSettings);
      }
      return Promise.resolve(undefined);
    });

    await result.current.fetchSettings();

    await waitFor(() => {
      expect(result.current.settings).toEqual(newSettings);
    });
  });

  it("should set loading state during fetch", async () => {
    let resolveInvoke: (value: AppSettings) => void;
    const invokePromise = new Promise<AppSettings>((resolve) => {
      resolveInvoke = resolve;
    });

    vi.mocked(invoke).mockReturnValue(invokePromise);

    const { result } = renderHook(() => useSettings());

    // Initially loading should be true (during mount fetch)
    expect(result.current.loading).toBe(true);

    resolveInvoke!(mockSettings);

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });

  it("should set loading state during update", async () => {
    vi.mocked(invoke).mockResolvedValue(mockSettings);

    const { result } = renderHook(() => useSettings());

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });

    let resolveUpdate: (value: undefined) => void;
    const updatePromise = new Promise<undefined>((resolve) => {
      resolveUpdate = resolve;
    });

    vi.mocked(invoke).mockReturnValue(updatePromise);

    const updatePromiseResult = result.current.updateSettings({
      ...mockSettings,
      auto_update: false,
    });

    // Should be loading during update
    await waitFor(() => {
      expect(result.current.loading).toBe(true);
    });

    resolveUpdate!(undefined);
    await updatePromiseResult;

    await waitFor(() => {
      expect(result.current.loading).toBe(false);
    });
  });
});
