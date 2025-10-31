/* eslint-env browser */
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { renderWithI18n } from "../test/test-utils";
import { Settings } from "./Settings";
import { ThemeProvider } from "../contexts/ThemeContext";
import * as dialog from "@tauri-apps/plugin-dialog";
import * as useSettingsModule from "../hooks/useSettings";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-dialog");

// Helper to render with ThemeProvider and I18nProvider
const renderWithTheme = (component: React.ReactElement) => {
  return renderWithI18n(<ThemeProvider>{component}</ThemeProvider>);
};

describe("Settings", () => {
  const mockOnClose = vi.fn();
  let mockUpdateSettings: ReturnType<typeof vi.fn>;
  let mockGetDefaultDirectory: ReturnType<typeof vi.fn>;

  const mockSettings = {
    auto_update: true,
    save_directory: null,
    launch_at_startup: false,
    theme: "system" as const,
    language: "auto" as const,
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock invoke for ThemeContext initialization
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      return Promise.resolve(undefined);
    });

    mockUpdateSettings = vi.fn().mockResolvedValue(undefined);
    mockGetDefaultDirectory = vi
      .fn()
      .mockResolvedValue("/Users/Test/Pictures/BingWallpapers");

    vi.spyOn(useSettingsModule, "useSettings").mockReturnValue({
      settings: mockSettings,
      loading: false,
      error: null,
      fetchSettings: vi.fn(),
      updateSettings: mockUpdateSettings,
      getDefaultDirectory: mockGetDefaultDirectory,
    });
  });

  it("should render settings modal", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Should show either the settings content or loading state
    await waitFor(() => {
      expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
    });
  });

  it("should close modal when close button is clicked", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const closeButton = await screen.findByText("×", {}, { timeout: 3000 });
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should have form inputs when loaded", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const checkbox = await screen.findByLabelText(
      /自动更新/i,
      {},
      { timeout: 3000 },
    );

    expect(checkbox).toBeInTheDocument();
  });

  it("should toggle auto-update checkbox", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const checkbox = (await screen.findByLabelText(
      /自动更新/i,
      {},
      { timeout: 3000 },
    )) as HTMLInputElement; // eslint-disable-line no-undef

    const initialValue = checkbox.checked;
    fireEvent.click(checkbox);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          auto_update: !initialValue,
        }),
      );
    });
  });

  it("should toggle launch at startup checkbox", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const checkbox = (await screen.findByLabelText(
      /开机自启动/i,
      {},
      { timeout: 3000 },
    )) as HTMLInputElement; // eslint-disable-line no-undef

    const initialValue = checkbox.checked;
    fireEvent.click(checkbox);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          launch_at_startup: !initialValue,
        }),
      );
    });
  });

  it("should open folder picker when select folder button clicked", async () => {
    const mockOpen = vi.fn().mockResolvedValue("/test/path");
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const selectButton = await screen.findByText(
      /选择文件夹/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(selectButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalledWith(
        expect.objectContaining({
          directory: true,
          multiple: false,
        }),
      );
    });
  });

  it("should update save directory when folder selected", async () => {
    const mockOpen = vi.fn().mockResolvedValue("/new/folder");
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const selectButton = await screen.findByText(
      /选择文件夹/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(selectButton);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          save_directory: "/new/folder",
        }),
      );
    });
  });

  it("should not update save directory when folder selection cancelled", async () => {
    const mockOpen = vi.fn().mockResolvedValue(null);
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const selectButton = await screen.findByText(
      /选择文件夹/i,
      {},
      { timeout: 3000 },
    );

    fireEvent.click(selectButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalled();
    });

    // updateSettings should not be called when dialog is cancelled
    expect(mockUpdateSettings).not.toHaveBeenCalled();
  });

  it("should handle folder selection error", async () => {
    const mockOpen = vi.fn().mockRejectedValue(new Error("Permission denied"));
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    // Mock console.error
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const selectButton = await screen.findByText(
      /选择文件夹/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(selectButton);

    await waitFor(
      () => {
        expect(consoleErrorSpy).toHaveBeenCalled();
      },
      { timeout: 3000 },
    );

    consoleErrorSpy.mockRestore();
  });

  it("should show restore default directory button when custom directory is set", async () => {
    // Mock settings with custom directory
    vi.spyOn(useSettingsModule, "useSettings").mockReturnValue({
      settings: { ...mockSettings, save_directory: "/custom/folder" },
      loading: false,
      error: null,
      fetchSettings: vi.fn(),
      updateSettings: mockUpdateSettings,
      getDefaultDirectory: mockGetDefaultDirectory,
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Wait for component to render
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // Restore default button should appear
    expect(screen.getByText(/恢复默认目录/i)).toBeInTheDocument();
  });

  it("should restore default directory when restore button clicked", async () => {
    // Mock settings with custom directory
    vi.spyOn(useSettingsModule, "useSettings").mockReturnValue({
      settings: { ...mockSettings, save_directory: "/custom/folder" },
      loading: false,
      error: null,
      fetchSettings: vi.fn(),
      updateSettings: mockUpdateSettings,
      getDefaultDirectory: mockGetDefaultDirectory,
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Wait for component to render
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // Click restore default
    const restoreButton = screen.getByText(/恢复默认目录/i);
    fireEvent.click(restoreButton);

    // Should call updateSettings with null to restore default
    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          save_directory: null,
        }),
      );
    });
  });

  it("should toggle theme selection", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Wait for component to render
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // Find and click light theme radio
    const lightThemeRadio = screen.getByLabelText(/浅色/i) as HTMLInputElement; // eslint-disable-line no-undef
    fireEvent.click(lightThemeRadio);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          theme: "light",
        }),
      );
    });
  });

  it("should toggle language selection", async () => {
    const mockSetLanguage = vi.fn();
    const mockOnLanguageChange = vi.fn();

    // Mock useI18n hook
    vi.doMock("../i18n/I18nContext", () => ({
      useI18n: () => ({
        t: (key: string) => key,
        setLanguage: mockSetLanguage,
      }),
    }));

    renderWithTheme(
      <Settings
        onClose={mockOnClose}
        onLanguageChange={mockOnLanguageChange}
      />,
    );

    // Wait for component to render
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // Find and click English language radio
    const enRadio = screen.getByLabelText(/English/i) as HTMLInputElement; // eslint-disable-line no-undef
    fireEvent.click(enRadio);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          language: "en-US",
        }),
      );
    });
  });

  it("should handle settings update error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    const alertSpy = vi.spyOn(window, "alert").mockImplementation(() => {});

    mockUpdateSettings.mockRejectedValueOnce(new Error("Update failed"));

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const checkbox = (await screen.findByLabelText(
      /自动更新/i,
      {},
      { timeout: 3000 },
    )) as HTMLInputElement; // eslint-disable-line no-undef

    fireEvent.click(checkbox);

    // Wait for the update to be attempted
    await waitFor(
      () => {
        expect(mockUpdateSettings).toHaveBeenCalled();
      },
      { timeout: 1000 },
    );

    // Wait for error handling
    await waitFor(
      () => {
        expect(consoleErrorSpy).toHaveBeenCalled();
      },
      { timeout: 1000 },
    );

    consoleErrorSpy.mockRestore();
    alertSpy.mockRestore();
  });

  it("should handle getDefaultDirectory error", async () => {
    const consoleErrorSpy = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});

    mockGetDefaultDirectory.mockRejectedValueOnce(
      new Error("Failed to get default directory"),
    );

    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Component should still render even if getDefaultDirectory fails
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();

    consoleErrorSpy.mockRestore();
  });

  it("should show loading state when settings are loading", () => {
    vi.spyOn(useSettingsModule, "useSettings").mockReturnValue({
      settings: null,
      loading: true,
      error: null,
      fetchSettings: vi.fn(),
      updateSettings: mockUpdateSettings,
      getDefaultDirectory: mockGetDefaultDirectory,
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    expect(screen.getByText(/加载设置中.../i)).toBeInTheDocument();
  });
});
