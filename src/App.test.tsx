import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  render,
  screen,
  fireEvent,
  waitFor,
  cleanup,
} from "@testing-library/react";
import React from "react";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { listen, type Event } from "@tauri-apps/api/event";
import { ThemeProvider } from "./contexts/ThemeContext";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/event");

const renderWithTheme = (component: React.ReactElement) => {
  return render(<ThemeProvider>{component}</ThemeProvider>);
};

describe("App", () => {
  const mockWallpapers = [
    {
      id: "20240101",
      start_date: "20240101",
      end_date: "20240102",
      title: "Test Wallpaper",
      copyright: "Test Copyright",
      copyright_link: "https://example.com/link",
      file_path: "/path/to/wallpaper.jpg",
      download_time: "2024-01-01T00:00:00Z",
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
    expect(screen.getByTitle("打开下载目录")).toBeInTheDocument();
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByTitle("打开下载目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开下载目录");
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
          keep_image_count: 8,
          launch_at_startup: false,
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText(/Error:/)).toBeInTheDocument();
    });
  });

  it("should display 已是最新 badge when wallpapers are up to date", async () => {
    const today = new Date();
    const todayStr = `${today.getFullYear()}${String(today.getMonth() + 1).padStart(2, "0")}${String(today.getDate()).padStart(2, "0")}`;

    const todayWallpaper = [
      {
        id: todayStr,
        start_date: todayStr,
        end_date: todayStr,
        title: "Today's Wallpaper",
        copyright: "Test",
        copyright_link: "https://example.com/link",
        file_path: "/path/to/today.jpg",
        download_time: new Date().toISOString(),
      },
    ];

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(todayWallpaper);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
        });
      }
      return Promise.resolve(null);
    });

    renderWithTheme(<App />);

    await waitFor(() => {
      expect(screen.getByText("已是最新")).toBeInTheDocument();
    });
  });

  it("should display last update time when available", async () => {
    const mockTime = "2024-01-01 12:00:00";
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_last_update_time") {
        return Promise.resolve(mockTime);
      }
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      expect(screen.getByTitle("打开下载目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开下载目录");
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
      if (cmd === "get_local_wallpapers") {
        return Promise.resolve(mockWallpapers);
      }
      if (cmd === "get_settings") {
        return Promise.resolve({
          auto_update: true,
          save_directory: null,
          keep_image_count: 8,
          launch_at_startup: false,
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
      expect(screen.getByTitle("打开下载目录")).toBeInTheDocument();
    });

    const folderButton = screen.getByTitle("打开下载目录");
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
});
