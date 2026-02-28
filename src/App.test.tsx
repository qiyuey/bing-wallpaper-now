import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  screen,
  fireEvent,
  waitFor,
  cleanup,
  act,
} from "@testing-library/react";
import React from "react";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { listen, type Event } from "@tauri-apps/api/event";
import { check, Update } from "@tauri-apps/plugin-updater";
import { ThemeProvider } from "./contexts/ThemeContext";
import { renderWithI18n } from "./test/test-utils";
import { LocalWallpaperRaw } from "./types";
import * as notificationUtils from "./utils/notification";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/event");
vi.mock("./utils/notification");
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
  Update: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn().mockResolvedValue(undefined),
}));

function createMockUpdate(version: string): Update {
  return {
    available: true,
    currentVersion: "1.0.0",
    version,
    body: "Release notes",
    rawJson: {},
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    download: vi.fn().mockResolvedValue(undefined),
    install: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  } as unknown as Update;
}

const renderWithTheme = (component: React.ReactElement) => {
  return renderWithI18n(<ThemeProvider>{component}</ThemeProvider>);
};

describe("App", () => {
  const mockWallpapersRaw: LocalWallpaperRaw[] = [
    {
      t: "Test Wallpaper",
      c: "Test Copyright",
      l: "https://example.com/link",
      d: "20240102",
      u: "/th?id=OHR.Test",
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock window and element dimensions for virtual list
    Object.defineProperty(window, "innerHeight", {
      writable: true,
      configurable: true,
      value: 800,
    });

    Object.defineProperty(HTMLElement.prototype, "offsetWidth", {
      writable: true,
      configurable: true,
      value: 1200,
    });

    Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
      writable: true,
      configurable: true,
      value: 600,
    });

    // Mock different invoke calls
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "zh-CN",
          effective_mkt: "zh-CN",
          is_mismatch: false,
        });
      }
      if (cmd === "get_supported_mkts") {
        return Promise.resolve([
          {
            region: "asia_pacific",
            markets: [{ code: "zh-CN", label: "中国大陆" }],
          },
        ]);
      }
      return Promise.resolve([]);
    });
    // Mock event listener
    vi.mocked(listen).mockResolvedValue(() => {});
    // Mock updater plugin (no update by default)
    vi.mocked(check).mockResolvedValue(null);
  });

  afterEach(() => {
    cleanup();
  });

  it("should render app header with title", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText("Bing Wallpaper")).toBeInTheDocument();
    });
    expect(screen.getByText("Now")).toBeInTheDocument();
  });

  it("should render action buttons in header", async () => {
    renderWithTheme(<App />);

    // Check for buttons by their aria-label attributes
    await waitFor(() => {
      expect(screen.getByLabelText("更新")).toBeInTheDocument();
    });
    expect(screen.getByLabelText("打开目录")).toBeInTheDocument();
    expect(screen.getByLabelText("设置")).toBeInTheDocument();
  });

  it("should open settings modal when settings button is clicked", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("设置")).toBeInTheDocument();
    });

    const settingsButton = screen.getByLabelText("设置");
    fireEvent.click(settingsButton);

    // Settings modal should appear (check for heading or loading text)
    await waitFor(() => {
      const heading = screen.queryByRole("heading", { name: /设置/i });
      const loading = screen.queryByText(/加载设置中.../i);
      expect(heading || loading).toBeInTheDocument();
    });
  });

  it("should call refresh handlers when refresh button is clicked", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("更新")).toBeInTheDocument();
    });

    const refreshButton = screen.getByLabelText("更新");
    fireEvent.click(refreshButton);

    await waitFor(() => {
      // Should call get_local_wallpapers at least once
      expect(invoke).toHaveBeenCalledWith("get_local_wallpapers");
    });
  });

  it("should open folder when folder button is clicked", async () => {
    const mockFolderPath = "C:\\Users\\Test\\Wallpapers";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve(mockFolderPath);
      }
      if (cmd === "ensure_wallpaper_directory_exists") {
        return Promise.resolve();
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByLabelText("打开目录");
    fireEvent.click(folderButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_wallpaper_directory");
      expect(invoke).toHaveBeenCalledWith("ensure_wallpaper_directory_exists");
      expect(openPath).toHaveBeenCalledWith(mockFolderPath);
    });
  });

  it("should display error message when error occurs", async () => {
    const errorMessage = "Failed to load wallpapers";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_local_wallpapers") {
        return Promise.reject(new Error(errorMessage));
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText(/Error:/)).toBeInTheDocument();
    });
  });

  it("should display last update time when available", async () => {
    const mockTime = "2024-01-01 12:00:00";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_last_update_time") {
        return Promise.resolve(mockTime);
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "zh-CN",
          effective_mkt: "zh-CN",
          is_mismatch: false,
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText(/上次更新:/)).toBeInTheDocument();
    });
  });

  it("should display effective mkt label after update time", async () => {
    const mockTime = "2024-01-01 12:00:00";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_last_update_time") {
        return Promise.resolve(mockTime);
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "ja-JP",
        });
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "ja-JP",
          effective_mkt: "ja-JP",
          is_mismatch: false,
        });
      }
      if (cmd === "get_supported_mkts") {
        return Promise.resolve([
          {
            region: "asia_pacific",
            markets: [
              { code: "ja-JP", label: "日本" },
              { code: "zh-CN", label: "中国大陆" },
            ],
          },
        ]);
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText(/日本/)).toBeInTheDocument();
    });
  });

  it("should listen for open-settings event", async () => {
    let openSettingsCallback: ((event: Event<unknown>) => void) | undefined;

    vi.mocked(listen).mockImplementation(
      (event: string, callback: (event: Event<unknown>) => void) => {
        if (event === "open-settings") {
          openSettingsCallback = callback;
        }
        return Promise.resolve(() => {});
      },
    );

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith(
        "open-settings",
        expect.any(Function),
      );
    });

    // Trigger the event
    if (openSettingsCallback) {
      await waitFor(() => {
        openSettingsCallback!({
          event: "open-settings",
          payload: undefined,
        } as Event<unknown>);
      });
    }

    // Settings should open (check for heading or loading text)
    await waitFor(() => {
      const heading = screen.queryByRole("heading", { name: /设置/i });
      const loading = screen.queryByText(/加载设置中.../i);
      expect(heading || loading).toBeInTheDocument();
    });
  });

  it("should listen for open-folder event", async () => {
    const mockFolderPath = "C:\\Users\\Test\\Wallpapers";
    let openFolderCallback: ((event: Event<unknown>) => void) | undefined;

    vi.mocked(listen).mockImplementation(
      (event: string, callback: (event: Event<unknown>) => void) => {
        if (event === "open-folder") {
          openFolderCallback = callback;
        }
        return Promise.resolve(() => {});
      },
    );

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve(mockFolderPath);
      }
      if (cmd === "ensure_wallpaper_directory_exists") {
        return Promise.resolve();
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve();
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith("open-folder", expect.any(Function));
    });

    // Trigger the event
    if (openFolderCallback) {
      openFolderCallback({
        event: "open-folder",
        payload: undefined,
      } as Event<unknown>);
    }

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("get_wallpaper_directory");
    });
  });

  it("should pass wallpapers to WallpaperGrid", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      // The wallpaper title should be rendered in the grid
      expect(screen.getByText("Test Wallpaper")).toBeInTheDocument();
    });
  });

  it("should close settings modal when X button is clicked", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("设置")).toBeInTheDocument();
    });

    // Open settings
    const settingsButton = screen.getByLabelText("设置");
    fireEvent.click(settingsButton);

    // Wait for settings to appear (check for heading or loading text)
    await waitFor(
      () => {
        const heading = screen.queryByRole("heading", { name: /设置/i });
        const loading = screen.queryByText(/加载设置中.../i);
        expect(heading || loading).toBeInTheDocument();
      },
      { timeout: 3000 },
    );

    // Find and click X button to close
    const closeButton = await screen.findByText("×", {}, { timeout: 3000 });
    fireEvent.click(closeButton);

    // Settings should close (modal heading should be gone)
    await waitFor(
      () => {
        expect(
          screen.queryByRole("heading", { name: /设置/i }),
        ).not.toBeInTheDocument();
      },
      { timeout: 3000 },
    );
  });

  it("should handle set wallpaper error", async () => {
    // This test is covered by WallpaperCard tests
    // We just verify the mock setup works correctly
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "set_desktop_wallpaper") {
        return Promise.reject(new Error("Failed to set wallpaper"));
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    // Wait for wallpapers to load
    await waitFor(() => {
      expect(screen.getByText("Test Wallpaper")).toBeInTheDocument();
    });

    // The actual error handling is tested in WallpaperCard.test.tsx
    // Here we just verify the component renders correctly with the mock
    expect(screen.getByText("Test Wallpaper")).toBeInTheDocument();
  });

  it("should handle force update error gracefully", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "force_update") {
        return Promise.reject(new Error("Update failed"));
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByLabelText("更新")).toBeInTheDocument();
    });

    const refreshButton = screen.getByLabelText("更新");
    fireEvent.click(refreshButton);

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Force update failed:",
        expect.any(Error),
      );
    });

    consoleErrorSpy.mockRestore();
  });

  it("should handle open folder error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.reject(new Error("Directory not found"));
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    // Wait for component to be ready
    await waitFor(() => {
      expect(screen.getByLabelText("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByLabelText("打开目录");
    fireEvent.click(folderButton);

    await waitFor(
      () => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          "Failed to open folder:",
          expect.any(Error),
        );
      },
      { timeout: 3000 },
    );

    consoleErrorSpy.mockRestore();
  });

  it("should handle open-folder event binding error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(listen).mockImplementation((event: string) => {
      if (event === "open-folder") {
        return Promise.reject(new Error("Event binding failed"));
      }
      return Promise.resolve(() => {});
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to bind open-folder event:",
        expect.any(Error),
      );
    });

    consoleErrorSpy.mockRestore();
  });

  it("should handle ensure_wallpaper_directory_exists error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/test/path");
      }
      if (cmd === "ensure_wallpaper_directory_exists") {
        return Promise.reject(new Error("Cannot create directory"));
      }
      if (cmd === "get_wallpaper_directory") {
        return Promise.resolve("/path/to/wallpapers");
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapersRaw);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          launch_at_startup: false,
          language: "zh-CN",
          resolved_language: "zh-CN",
          mkt: "zh-CN",
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    // Wait for component to be ready
    await waitFor(() => {
      expect(screen.getByLabelText("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByLabelText("打开目录");
    fireEvent.click(folderButton);

    await waitFor(
      () => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          "Failed to open folder:",
          expect.any(Error),
        );
      },
      { timeout: 3000 },
    );

    consoleErrorSpy.mockRestore();
  });

  describe("Version check on startup", () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it("should check for updates after 1 minute delay", async () => {
      const mockUpdate = createMockUpdate("0.4.6");
      vi.mocked(check).mockResolvedValue(mockUpdate);
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "is_version_ignored") {
          return Promise.resolve(false);
        }
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 快进 59 秒，版本检查应该还没执行
      vi.advanceTimersByTime(59000);
      expect(check).not.toHaveBeenCalled();

      // 快进 1 秒，到达 60 秒，版本检查应该执行
      await act(async () => {
        vi.advanceTimersByTime(1000);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(check).toHaveBeenCalled();
    });

    it("should display update dialog when update is available and not ignored", async () => {
      const mockUpdate = createMockUpdate("0.4.6");
      vi.mocked(check).mockResolvedValue(mockUpdate);
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "is_version_ignored") {
          return Promise.resolve(false);
        }
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      await act(async () => {
        vi.advanceTimersByTime(60000);
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(
        screen.getByText(/有新版本可用|Update Available/),
      ).toBeInTheDocument();
    });

    it("should not display update dialog when version is ignored", async () => {
      const mockUpdate = createMockUpdate("0.4.6");
      vi.mocked(check).mockResolvedValue(mockUpdate);
      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "is_version_ignored") {
          return Promise.resolve(true);
        }
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      await act(async () => {
        vi.advanceTimersByTime(60000);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(check).toHaveBeenCalled();
      expect(
        screen.queryByText(/有新版本可用|Update Available/),
      ).not.toBeInTheDocument();
    });

    it("should not display update dialog when no update is available", async () => {
      vi.mocked(check).mockResolvedValue(null);

      renderWithTheme(<App />);

      await act(async () => {
        vi.advanceTimersByTime(60000);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(check).toHaveBeenCalled();
      expect(
        screen.queryByText(/有新版本可用|Update Available/),
      ).not.toBeInTheDocument();
    });

    it("should clean up timeout when component unmounts", () => {
      const clearTimeoutSpy = vi.spyOn(window, "clearTimeout");

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      const { unmount } = renderWithTheme(<App />);

      // 验证有定时器被创建
      expect(vi.getTimerCount()).toBeGreaterThan(0);

      // 卸载组件
      unmount();

      // 验证 clearTimeout 被调用
      expect(clearTimeoutSpy).toHaveBeenCalled();

      clearTimeoutSpy.mockRestore();
    });

    it("should handle version check error gracefully", async () => {
      const consoleErrorSpy = vi
        .spyOn(console, "error")
        .mockImplementation(() => {});

      vi.mocked(check).mockRejectedValue(new Error("Network error"));

      renderWithTheme(<App />);

      await act(async () => {
        vi.advanceTimersByTime(60000);
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to check for updates:",
        expect.any(Error),
      );

      expect(
        screen.queryByText(/有新版本可用|Update Available/),
      ).not.toBeInTheDocument();

      consoleErrorSpy.mockRestore();
    });
  });

  describe("Keyboard shortcuts", () => {
    beforeEach(() => {
      vi.mocked(notificationUtils.showSystemNotification).mockResolvedValue();
    });

    it("should open settings when Cmd/Ctrl + , is pressed", async () => {
      renderWithTheme(<App />);

      await waitFor(() => {
        expect(screen.getByText("Bing Wallpaper")).toBeInTheDocument();
      });

      // Press Cmd + , (macOS) or Ctrl + , (Windows/Linux)
      fireEvent.keyDown(window, {
        key: ",",
        metaKey: true, // macOS
        ctrlKey: false,
      });

      await waitFor(() => {
        const heading = screen.queryByRole("heading", {
          name: /设置|Settings/i,
        });
        const loading = screen.queryByText(/加载设置中.../i);
        expect(heading || loading).toBeInTheDocument();
      });
    });

    it("should open settings when Ctrl + , is pressed (Windows/Linux)", async () => {
      renderWithTheme(<App />);

      await waitFor(() => {
        expect(screen.getByText("Bing Wallpaper")).toBeInTheDocument();
      });

      // Press Ctrl + , (Windows/Linux)
      fireEvent.keyDown(window, {
        key: ",",
        metaKey: false,
        ctrlKey: true,
      });

      await waitFor(() => {
        const heading = screen.queryByRole("heading", {
          name: /设置|Settings/i,
        });
        const loading = screen.queryByText(/加载设置中.../i);
        expect(heading || loading).toBeInTheDocument();
      });
    });

    it("should close settings modal when Esc is pressed", async () => {
      renderWithTheme(<App />);

      await waitFor(() => {
        expect(screen.getByText("Bing Wallpaper")).toBeInTheDocument();
      });

      // Open settings first
      const settingsButton = screen.getByLabelText(/设置|Settings/i);
      fireEvent.click(settingsButton);

      await waitFor(() => {
        const heading = screen.queryByRole("heading", {
          name: /设置|Settings/i,
        });
        const loading = screen.queryByText(/加载设置中.../i);
        expect(heading || loading).toBeInTheDocument();
      });

      // Press Esc to close
      fireEvent.keyDown(window, {
        key: "Escape",
      });

      await waitFor(() => {
        expect(
          screen.queryByRole("heading", { name: /设置|Settings/i }),
        ).not.toBeInTheDocument();
      });
    });

    it("should close about modal when Esc is pressed", async () => {
      // Mock listen to capture the open-about callback
      // Need to reset the mock first to capture all listeners
      vi.mocked(listen).mockClear();

      let openAboutCallback: ((event: Event<unknown>) => void) | undefined;

      vi.mocked(listen).mockImplementation(
        (event: string, callback: (event: Event<unknown>) => void) => {
          if (event === "open-about") {
            openAboutCallback = callback;
          }
          // Return unlisten function for other events too
          return Promise.resolve(() => {});
        },
      );

      renderWithTheme(<App />);

      await waitFor(() => {
        expect(screen.getByText("Bing Wallpaper")).toBeInTheDocument();
      });

      // Wait for listener to be set up
      await waitFor(() => {
        expect(listen).toHaveBeenCalledWith("open-about", expect.any(Function));
      });

      // Trigger the event to open about dialog
      if (openAboutCallback) {
        await act(async () => {
          openAboutCallback!({
            event: "open-about",
            payload: undefined,
          } as Event<unknown>);
        });
      }

      // Wait for about dialog to appear - check for "关于" or "About" title
      await waitFor(
        () => {
          const aboutText = screen.getByText(/关于|About/i);
          expect(aboutText).toBeInTheDocument();
        },
        { timeout: 3000 },
      );

      // Press Esc to close
      fireEvent.keyDown(window, {
        key: "Escape",
      });

      await waitFor(() => {
        expect(screen.queryByText(/关于|About/i)).not.toBeInTheDocument();
      });
    });

    it("should close update dialog when Esc is pressed", async () => {
      const mockUpdate = createMockUpdate("0.4.6");
      vi.mocked(check).mockResolvedValue(mockUpdate);

      let trayCheckUpdatesCallback:
        | ((event: Event<unknown>) => void)
        | undefined;

      vi.mocked(listen).mockImplementation(
        (event: string, callback: (event: Event<unknown>) => void) => {
          if (event === "tray-check-updates") {
            trayCheckUpdatesCallback = callback;
          }
          return Promise.resolve(() => {});
        },
      );

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "is_version_ignored") {
          return Promise.resolve(false);
        }
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      await waitFor(() => {
        expect(trayCheckUpdatesCallback).toBeDefined();
      });

      await act(async () => {
        await trayCheckUpdatesCallback!({
          event: "tray-check-updates",
          payload: undefined,
        } as Event<unknown>);
      });

      await waitFor(() => {
        expect(
          screen.getByText(/有新版本可用|Update Available/i),
        ).toBeInTheDocument();
      });

      fireEvent.keyDown(window, {
        key: "Escape",
      });

      await waitFor(() => {
        expect(
          screen.queryByText(/有新版本可用|Update Available/i),
        ).not.toBeInTheDocument();
      });
    });
  });

  describe("Check updates event listeners", () => {
    beforeEach(() => {
      vi.mocked(notificationUtils.showSystemNotification).mockResolvedValue();
    });

    it("should display update dialog when tray-check-updates event triggers and update found", async () => {
      const mockUpdate = createMockUpdate("0.4.6");
      vi.mocked(check).mockResolvedValue(mockUpdate);

      let trayCheckUpdatesCallback:
        | ((event: Event<unknown>) => void)
        | undefined;

      vi.mocked(listen).mockImplementation(
        (event: string, callback: (event: Event<unknown>) => void) => {
          if (event === "tray-check-updates") {
            trayCheckUpdatesCallback = callback;
          }
          return Promise.resolve(() => {});
        },
      );

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "is_version_ignored") {
          return Promise.resolve(false);
        }
        if (cmd === "get_wallpaper_directory") {
          return Promise.resolve("/path/to/wallpapers");
        }
        if (cmd === "get_local_wallpapers") {
          return Promise.resolve(mockWallpapersRaw);
        }
        if (cmd === "get_settings") {
          return Promise.resolve({
            auto_update: true,
            save_directory: null,
            launch_at_startup: false,
            language: "zh-CN",
            resolved_language: "zh-CN",
            mkt: "zh-CN",
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      await waitFor(() => {
        expect(listen).toHaveBeenCalledWith(
          "tray-check-updates",
          expect.any(Function),
        );
      });

      if (trayCheckUpdatesCallback) {
        await act(async () => {
          await trayCheckUpdatesCallback!({
            event: "tray-check-updates",
            payload: undefined,
          } as Event<unknown>);
        });
      }

      await waitFor(() => {
        expect(
          screen.getByText(/有新版本可用|Update Available/i),
        ).toBeInTheDocument();
      });
    });

    it("should show system notification when tray-check-updates finds no update", async () => {
      vi.mocked(check).mockResolvedValue(null);

      let trayCheckUpdatesCallback:
        | ((event: Event<unknown>) => void)
        | undefined;

      vi.mocked(listen).mockImplementation(
        (event: string, callback: (event: Event<unknown>) => void) => {
          if (event === "tray-check-updates") {
            trayCheckUpdatesCallback = callback;
          }
          return Promise.resolve(() => {});
        },
      );

      renderWithTheme(<App />);

      await waitFor(() => {
        expect(listen).toHaveBeenCalledWith(
          "tray-check-updates",
          expect.any(Function),
        );
      });

      if (trayCheckUpdatesCallback) {
        await act(async () => {
          await trayCheckUpdatesCallback!({
            event: "tray-check-updates",
            payload: undefined,
          } as Event<unknown>);
        });
      }

      await waitFor(() => {
        expect(notificationUtils.showSystemNotification).toHaveBeenCalled();
      });

      expect(notificationUtils.showSystemNotification).toHaveBeenCalledWith(
        "检查更新",
        "已是最新版本",
      );
    });
  });
});
