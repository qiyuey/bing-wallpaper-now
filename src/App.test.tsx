import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import App from "./App";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { listen, type Event } from "@tauri-apps/api/event";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/event");

describe("App", () => {
  const mockWallpapers = [
    {
      id: "20240101",
      start_date: "20240101",
      title: "Test Wallpaper",
      copyright: "Test Copyright",
      file_path: "/path/to/wallpaper.jpg",
      url: "https://example.com/wallpaper.jpg",
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
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

  it("should render app header with title", async () => {
    render(<App />);

    expect(screen.getByText("Bing Wallpaper Now")).toBeInTheDocument();
  });

  it("should render action buttons in header", async () => {
    render(<App />);

    // Check for buttons by their title attributes
    expect(screen.getByTitle("更新")).toBeInTheDocument();
    expect(screen.getByTitle("打开下载目录")).toBeInTheDocument();
    expect(screen.getByTitle("设置")).toBeInTheDocument();
  });

  it("should open settings modal when settings button is clicked", async () => {
    render(<App />);

    const settingsButton = screen.getByTitle("设置");
    fireEvent.click(settingsButton);

    // Settings modal should appear
    await waitFor(() => {
      expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
    });
  });

  it("should call refresh handlers when refresh button is clicked", async () => {
    render(<App />);

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

    render(<App />);

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

    render(<App />);

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
        title: "Today's Wallpaper",
        copyright: "Test",
        file_path: "/path/to/today.jpg",
        url: "https://example.com/today.jpg",
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

    render(<App />);

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

    render(<App />);

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

    render(<App />);

    await waitFor(() => {
      expect(listen).toHaveBeenCalledWith(
        "open-settings",
        expect.any(Function),
      );
    });

    // Trigger the event
    if (openSettingsCallback) {
      openSettingsCallback({
        event: "open-settings",
        payload: undefined,
      } as Event<unknown>);
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
      return Promise.resolve(mockWallpapers);
    });

    render(<App />);

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

    render(<App />);

    await waitFor(() => {
      // The wallpaper title should be rendered in the grid
      expect(screen.getByText("Test Wallpaper")).toBeInTheDocument();
    });
  });

  it("should close settings modal when onClose is called", async () => {
    render(<App />);

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

    // Find and click cancel button
    const cancelButton = await screen.findByText("取消", {}, { timeout: 3000 });
    fireEvent.click(cancelButton);

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
});
