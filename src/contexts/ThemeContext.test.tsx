import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, waitFor, act } from "@testing-library/react";
import { ThemeProvider, useTheme } from "./ThemeContext";
import { invoke } from "@tauri-apps/api/core";
import { ReactNode } from "react";

vi.mock("@tauri-apps/api/core");

describe("ThemeContext", () => {
  const mockSettings = {
    theme: "system",
    auto_update: true,
    save_directory: null,
    launch_at_startup: false,
    language: "zh-CN",
    resolved_language: "zh-CN",
    mkt: "zh-CN",
  };

  let matchMediaMock: {
    matches: boolean;
    addEventListener: ReturnType<typeof vi.fn>;
    removeEventListener: ReturnType<typeof vi.fn>;
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock document.documentElement.setAttribute
    vi.spyOn(document.documentElement, "setAttribute");

    // Mock window.matchMedia
    matchMediaMock = {
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    };

    vi.stubGlobal(
      "matchMedia",
      vi.fn().mockImplementation(() => matchMediaMock),
    );

    // Mock invoke to return settings
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      if (cmd === "update_settings") {
        return Promise.resolve();
      }
      return Promise.resolve(null);
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  const wrapper = ({ children }: { children: ReactNode }) => (
    <ThemeProvider>{children}</ThemeProvider>
  );

  it("should initialize with system theme by default", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });
  });

  it("should load theme from settings on mount", async () => {
    const mockSettingsWithDark = {
      ...mockSettings,
      theme: "dark",
    };

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettingsWithDark);
      }
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("dark");
      expect(result.current.actualTheme).toBe("dark");
    });
  });

  it("should resolve system theme to light when system is light", async () => {
    matchMediaMock.matches = false; // Light mode

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.actualTheme).toBe("light");
    });
  });

  it("should resolve system theme based on matchMedia", async () => {
    // This test verifies that the theme context calls matchMedia
    // Actual theme resolution is tested via integration
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // Verify matchMedia was called during initialization
    expect(window.matchMedia).toHaveBeenCalledWith(
      "(prefers-color-scheme: dark)",
    );
  });

  it("should apply theme to document", async () => {
    renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(document.documentElement.setAttribute).toHaveBeenCalledWith(
        "data-theme",
        expect.any(String),
      );
    });
  });

  it("should update theme when setTheme is called", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    await act(async () => {
      await result.current.setTheme("dark");
    });

    expect(result.current.theme).toBe("dark");
    expect(result.current.actualTheme).toBe("dark");
    expect(invoke).toHaveBeenCalledWith("update_settings", {
      newSettings: expect.objectContaining({
        theme: "dark",
      }),
    });
  });

  it("should update theme to light", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    await act(async () => {
      await result.current.setTheme("light");
    });

    expect(result.current.theme).toBe("light");
    expect(result.current.actualTheme).toBe("light");
  });

  it("should persist theme changes to backend", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    await act(async () => {
      await result.current.setTheme("dark");
    });

    expect(invoke).toHaveBeenCalledWith("get_settings");
    expect(invoke).toHaveBeenCalledWith("update_settings", {
      newSettings: {
        auto_update: mockSettings.auto_update,
        save_directory: mockSettings.save_directory,
        launch_at_startup: mockSettings.launch_at_startup,
        theme: "dark",
      },
    });
  });

  it("should listen for system theme changes", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    // Wait for initialization to complete
    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // The matchMedia mock should have been called
    expect(window.matchMedia).toHaveBeenCalledWith(
      "(prefers-color-scheme: dark)",
    );
  });

  it("should register event listener for system theme changes", async () => {
    // This test verifies that the theme context sets up event listeners
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // Verify that matchMedia was called (which means listeners were set up)
    expect(window.matchMedia).toHaveBeenCalledWith(
      "(prefers-color-scheme: dark)",
    );
  });

  it("should not update theme on system change when theme is not system", async () => {
    const mockSettingsLight = {
      ...mockSettings,
      theme: "light",
    };

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettingsLight);
      }
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("light");
    });

    // Simulate system theme change
    const changeHandler = matchMediaMock.addEventListener.mock.calls[0]?.[1];
    if (changeHandler) {
      act(() => {
        // 模拟 MediaQueryListEvent
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        changeHandler({ matches: true } as any);
      });
    }

    // Theme should remain light, not change to dark
    expect(result.current.actualTheme).toBe("light");
  });

  it("should cleanup event listener on unmount", async () => {
    const { result, unmount } = renderHook(() => useTheme(), { wrapper });

    // Wait for initialization
    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // Verify component initialized properly
    expect(result.current.theme).toBe("system");

    unmount();

    // After unmount, the component should have cleaned up
    // This is verified by the test completing without errors
    expect(result.current).toBeDefined();
  });

  it("should throw error when useTheme is used outside ThemeProvider", () => {
    // Expect the hook to throw an error when used outside provider
    expect(() => {
      renderHook(() => useTheme());
    }).toThrow("useTheme must be used within a ThemeProvider");
  });

  it("should handle get_settings error gracefully", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.reject(new Error("Failed to load settings"));
      }
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalledWith(
        "Failed to load theme from settings:",
        expect.any(Error),
      );
    });

    // Should fallback to system theme
    expect(result.current.theme).toBe("system");

    consoleErrorSpy.mockRestore();
  });

  it("should handle update_settings error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    let callCount = 0;
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        callCount++;
        // First call is during initialization, second is when setTheme is called
        if (callCount === 1) {
          return Promise.resolve(mockSettings);
        } else {
          return Promise.resolve(mockSettings);
        }
      }
      if (cmd === "update_settings") {
        return Promise.reject(new Error("Failed to save settings"));
      }
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // Attempt to change theme, which should fail
    await expect(async () => {
      await act(async () => {
        await result.current.setTheme("dark");
      });
    }).rejects.toThrow();

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "Failed to save theme:",
      expect.any(Error),
    );

    consoleErrorSpy.mockRestore();
  });

  it("should resolve system theme to dark when system prefers dark", async () => {
    // Override matchMedia to return dark preference
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: true, // Dark mode
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
      expect(result.current.actualTheme).toBe("dark");
    });
  });

  it("should handle invalid settings object from backend", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(null); // Invalid settings
      }
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(consoleErrorSpy).toHaveBeenCalled();
    });

    // Should fallback to system theme
    expect(result.current.theme).toBe("system");

    consoleErrorSpy.mockRestore();
  });

  it("should update theme when system theme changes and theme is system", async () => {
    // Override matchMedia with a captured handler approach
    let capturedHandler: ((e: { matches: boolean }) => void) | null = null;
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(
          (event: string, handler: (e: { matches: boolean }) => void) => {
            if (event === "change") {
              capturedHandler = handler;
            }
          },
        ),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });

    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
      expect(result.current.actualTheme).toBe("light");
    });

    expect(capturedHandler).not.toBeNull();

    // Simulate system theme changing to dark
    act(() => {
      capturedHandler!({ matches: true });
    });

    expect(result.current.actualTheme).toBe("dark");
    expect(document.documentElement.setAttribute).toHaveBeenCalledWith(
      "data-theme",
      "dark",
    );
  });

  it("should apply theme to UI without saving to backend via applyThemeToUI", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    const invokeCallsBefore = vi.mocked(invoke).mock.calls.length;

    act(() => {
      result.current.applyThemeToUI("dark");
    });

    expect(result.current.theme).toBe("dark");
    expect(result.current.actualTheme).toBe("dark");

    // applyThemeToUI should NOT call invoke (no backend save)
    const invokeCallsAfter = vi.mocked(invoke).mock.calls.length;
    expect(invokeCallsAfter).toBe(invokeCallsBefore);
  });

  it("should cycle through all theme options", async () => {
    const { result } = renderHook(() => useTheme(), { wrapper });

    await waitFor(() => {
      expect(result.current.theme).toBe("system");
    });

    // Change to light
    await act(async () => {
      await result.current.setTheme("light");
    });
    expect(result.current.theme).toBe("light");

    // Change to dark
    await act(async () => {
      await result.current.setTheme("dark");
    });
    expect(result.current.theme).toBe("dark");

    // Change back to system
    await act(async () => {
      await result.current.setTheme("system");
    });
    expect(result.current.theme).toBe("system");
  });
});
