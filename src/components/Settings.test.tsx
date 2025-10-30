/* eslint-env browser */
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { Settings } from "./Settings";
import { ThemeProvider } from "../contexts/ThemeContext";
import * as dialog from "@tauri-apps/plugin-dialog";
import * as useSettingsModule from "../hooks/useSettings";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-dialog");

// Helper to render with ThemeProvider
const renderWithTheme = (component: React.ReactElement) => {
  return render(<ThemeProvider>{component}</ThemeProvider>);
};

describe("Settings", () => {
  const mockOnClose = vi.fn();
  let mockUpdateSettings: ReturnType<typeof vi.fn>;
  let mockGetDefaultDirectory: ReturnType<typeof vi.fn>;

  const mockSettings = {
    auto_update: true,
    save_directory: null,
    keep_image_count: 30,
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

  it("should update keep image count", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const input = (await screen.findByLabelText(
      /保留壁纸数量/i,
      {},
      { timeout: 3000 },
    )) as HTMLInputElement; // eslint-disable-line no-undef

    fireEvent.change(input, { target: { value: "20" } });

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          keep_image_count: 20,
        }),
      );
    });
  });

  it("should accept keep image count below 8", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const input = (await screen.findByLabelText(
      /保留壁纸数量/i,
      {},
      { timeout: 3000 },
    )) as HTMLInputElement; // eslint-disable-line no-undef

    // Try to set below minimum - the component will call updateSettings with this value
    fireEvent.change(input, { target: { value: "5" } });

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          keep_image_count: 5,
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
});
