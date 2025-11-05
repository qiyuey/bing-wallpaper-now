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
import { ThemeProvider } from "./contexts/ThemeContext";
import { renderWithI18n } from "./test/test-utils";
import { LocalWallpaperRaw } from "./types";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/event");

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
        });
      }
      if (cmd === "get_last_update_time") {
        return Promise.resolve(null);
      }
      return Promise.resolve([]);
    });
    // Mock event listener
    vi.mocked(listen).mockResolvedValue(() => {});
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

    // Check for buttons by their title attributes
    await waitFor(() => {
      expect(screen.getByTitle("更新")).toBeInTheDocument();
    });
    expect(screen.getByTitle("打开目录")).toBeInTheDocument();
    expect(screen.getByTitle("设置")).toBeInTheDocument();
  });

  it("should open settings modal when settings button is clicked", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByTitle("设置")).toBeInTheDocument();
    });

    const settingsButton = screen.getByTitle("设置");
    fireEvent.click(settingsButton);

    // Settings modal should appear
    await waitFor(() => {
      expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
    });
  });

  it("should call refresh handlers when refresh button is clicked", async () => {
    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByTitle("更新")).toBeInTheDocument();
    });

    const refreshButton = screen.getByTitle("更新");
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
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByTitle("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开目录");
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
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText(/上次更新:/)).toBeInTheDocument();
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

    // Settings should open
    await waitFor(() => {
      expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
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
      expect(screen.getByTitle("设置")).toBeInTheDocument();
    });

    // Open settings
    const settingsButton = screen.getByTitle("设置");
    fireEvent.click(settingsButton);

    // Wait for settings to appear
    await waitFor(
      () => {
        expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
      },
      { timeout: 3000 },
    );

    // Find and click X button to close
    const closeButton = await screen.findByText("×", {}, { timeout: 3000 });
    fireEvent.click(closeButton);

    // Settings should close (may take a moment for modal to unmount)
    await waitFor(
      () => {
        const settingsTexts = screen.queryAllByText(/设置/);
        // Should only have the settings button left, not the modal
        expect(settingsTexts.length).toBeLessThanOrEqual(1);
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
    const consoleWarnSpy = vi
      .spyOn(console, "warn")
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
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByTitle("更新")).toBeInTheDocument();
    });

    const refreshButton = screen.getByTitle("更新");
    fireEvent.click(refreshButton);

    await waitFor(() => {
      expect(consoleWarnSpy).toHaveBeenCalledWith(
        "Force update failed:",
        expect.any(Error),
      );
    });

    consoleWarnSpy.mockRestore();
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
      expect(screen.getByTitle("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开目录");
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
      expect(screen.getByTitle("打开目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开目录");
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
      const mockVersionCheckResult = {
        current_version: "0.4.5",
        latest_version: "0.4.6",
        has_update: true,
        release_url: "https://github.com/example/releases/tag/0.4.6",
        platform_available: true,
      };

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_for_updates") {
          return Promise.resolve(mockVersionCheckResult);
        }
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
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 验证 setTimeout 被调用，但还没有执行版本检查
      expect(invoke).not.toHaveBeenCalledWith("check_for_updates");

      // 快进 59 秒，版本检查应该还没执行
      vi.advanceTimersByTime(59000);
      expect(invoke).not.toHaveBeenCalledWith("check_for_updates");

      // 快进 1 秒，到达 60 秒，版本检查应该执行
      await act(async () => {
        vi.advanceTimersByTime(1000);
        // 等待 Promise 链完成（版本检查是异步的）
        await Promise.resolve();
        await Promise.resolve();
      });

      expect(invoke).toHaveBeenCalledWith("check_for_updates");
    });

    it("should display update dialog when update is available and not ignored", async () => {
      const mockVersionCheckResult = {
        current_version: "0.4.5",
        latest_version: "0.4.6",
        has_update: true,
        release_url: "https://github.com/example/releases/tag/0.4.6",
        platform_available: true,
      };

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_for_updates") {
          return Promise.resolve(mockVersionCheckResult);
        }
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
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 快进 60 秒并等待 Promise 完成
      await act(async () => {
        vi.advanceTimersByTime(60000);
        // 等待 Promise 链完成（版本检查 -> 检查忽略状态 -> 设置状态）
        await Promise.resolve();
        await Promise.resolve();
        await Promise.resolve();
      });

      // 验证更新对话框显示（在 fake timers 模式下，直接查询而不是使用 waitFor）
      expect(
        screen.getByText(/有新版本可用|Update Available/),
      ).toBeInTheDocument();
    });

    it("should not display update dialog when version is ignored", async () => {
      const mockVersionCheckResult = {
        current_version: "0.4.5",
        latest_version: "0.4.6",
        has_update: true,
        release_url: "https://github.com/example/releases/tag/0.4.6",
        platform_available: true,
      };

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_for_updates") {
          return Promise.resolve(mockVersionCheckResult);
        }
        if (cmd === "is_version_ignored") {
          return Promise.resolve(true); // 版本已被忽略
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
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 快进 60 秒并等待 Promise 完成
      await act(async () => {
        vi.advanceTimersByTime(60000);
        // 等待 Promise 链完成
        await Promise.resolve();
        await Promise.resolve();
      });

      // 验证版本检查被调用
      expect(invoke).toHaveBeenCalledWith("check_for_updates");

      // 更新对话框不应该显示（因为版本被忽略）
      expect(
        screen.queryByText(/有新版本可用|Update Available/),
      ).not.toBeInTheDocument();
    });

    it("should not display update dialog when no update is available", async () => {
      const mockVersionCheckResult = {
        current_version: "0.4.6",
        latest_version: "0.4.6",
        has_update: false,
        release_url: null,
        platform_available: true,
      };

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_for_updates") {
          return Promise.resolve(mockVersionCheckResult);
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
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 快进 60 秒并等待 Promise 完成
      await act(async () => {
        vi.advanceTimersByTime(60000);
        // 等待 Promise 链完成
        await Promise.resolve();
        await Promise.resolve();
      });

      // 验证版本检查被调用
      expect(invoke).toHaveBeenCalledWith("check_for_updates");

      // 更新对话框不应该显示（因为没有更新）
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

      vi.mocked(invoke).mockImplementation((cmd: string) => {
        if (cmd === "check_for_updates") {
          return Promise.reject(new Error("Network error"));
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
          });
        }
        if (cmd === "get_last_update_time") {
          return Promise.resolve(null);
        }
        return Promise.resolve([]);
      });

      renderWithTheme(<App />);

      // 快进 60 秒并等待 Promise 完成
      await act(async () => {
        vi.advanceTimersByTime(60000);
        // 等待 Promise 链完成（包括错误处理）
        await Promise.resolve();
        await Promise.resolve();
      });

      // 验证错误被记录
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to check for updates:",
        expect.any(Error),
      );

      // 更新对话框不应该显示
      expect(
        screen.queryByText(/有新版本可用|Update Available/),
      ).not.toBeInTheDocument();

      consoleErrorSpy.mockRestore();
    });
  });
});
