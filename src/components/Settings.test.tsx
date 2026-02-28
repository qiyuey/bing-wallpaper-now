import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { renderWithI18n } from "../test/test-utils";
import { Settings } from "./Settings";
import { ThemeProvider } from "../contexts/ThemeContext";
import * as dialog from "@tauri-apps/plugin-dialog";
import * as useSettingsModule from "../hooks/useSettings";
import { invoke } from "@tauri-apps/api/core";
import type { AppSettings } from "../types";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-dialog");

// Helper to render with ThemeProvider and I18nProvider
const renderWithTheme = (component: React.ReactElement) => {
  return renderWithI18n(<ThemeProvider>{component}</ThemeProvider>);
};

// 定义函数类型
type UpdateSettingsFn = (newSettings: AppSettings) => Promise<void>;
type GetDefaultDirectoryFn = () => Promise<string | null>;

describe("Settings", () => {
  const mockOnClose = vi.fn();
  // 按照正确的方式声明 mock 变量
  const mockUpdateSettings = vi.fn<UpdateSettingsFn>();
  const mockGetDefaultDirectory = vi.fn<GetDefaultDirectoryFn>();

  const mockSettings = {
    auto_update: true,
    save_directory: null,
    launch_at_startup: false,
    theme: "system" as const,
    language: "zh-CN" as const,
    resolved_language: "zh-CN" as const,
    mkt: "zh-CN" as const,
  };

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock invoke for ThemeContext initialization, MarketStatus and market groups
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
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
            markets: [
              { code: "zh-CN", label: "中国大陆" },
              { code: "ja-JP", label: "日本" },
            ],
          },
          {
            region: "americas",
            markets: [{ code: "en-US", label: "United States" }],
          },
        ]);
      }
      return Promise.resolve(undefined);
    });

    // 重置 mock 的实现
    mockUpdateSettings.mockResolvedValue(undefined);
    mockGetDefaultDirectory.mockResolvedValue(
      "/Users/Test/Pictures/BingWallpapers",
    );

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
      // Check for settings title or loading state
      const title = screen.queryByRole("heading", { name: /设置/i });
      const loading = screen.queryByText(/加载设置中.../i);
      expect(title || loading).toBeInTheDocument();
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
      /自动应用新壁纸/i,
      {},
      { timeout: 3000 },
    );

    expect(checkbox).toBeInTheDocument();
  });

  it("should toggle auto-update checkbox", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    const checkbox = (await screen.findByLabelText(
      /自动应用新壁纸/i,
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

  it("should toggle theme selection to light", async () => {
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

  it("should toggle theme selection to dark", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    // Wait for component to render
    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // Find and click dark theme radio
    const darkThemeRadio = screen.getByLabelText(/深色/i) as HTMLInputElement; // eslint-disable-line no-undef
    fireEvent.click(darkThemeRadio);

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          theme: "dark",
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
      /自动应用新壁纸/i,
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

    // Check for settings title or loading state
    const title = screen.queryByRole("heading", { name: /设置/i });
    const loading = screen.queryByText(/加载设置中.../i);
    expect(title || loading).toBeInTheDocument();

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

  // ─── mkt-mismatch 告警（pull 模式） ───

  it("should show mkt mismatch warning when get_market_status returns mismatch", async () => {
    // 覆盖 invoke mock，让 get_market_status 返回 mismatch 状态
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "en-US",
          effective_mkt: "zh-CN",
          is_mismatch: true,
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
      return Promise.resolve(undefined);
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    await waitFor(() => {
      // 告警文案包含"实际返回了 zh-CN"
      expect(screen.getByText(/实际返回了 zh-CN/)).toBeInTheDocument();
    });

    // 应有 dismiss 按钮
    expect(screen.getByLabelText("dismiss")).toBeInTheDocument();
  });

  it("should not show mkt mismatch warning when no mismatch", async () => {
    // 默认 mock 已返回 is_mismatch: false
    renderWithTheme(<Settings onClose={mockOnClose} />);

    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // 不应有 dismiss 按钮（即没有告警）
    expect(screen.queryByLabelText("dismiss")).not.toBeInTheDocument();
  });

  it("should dismiss mkt mismatch warning when dismiss button is clicked", async () => {
    // 覆盖 invoke mock，让 get_market_status 返回 mismatch 状态
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") {
        return Promise.resolve(mockSettings);
      }
      if (cmd === "get_market_status") {
        return Promise.resolve({
          requested_mkt: "en-US",
          effective_mkt: "zh-CN",
          is_mismatch: true,
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
      return Promise.resolve(undefined);
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const dismissButton = await screen.findByLabelText("dismiss");
    fireEvent.click(dismissButton);

    // 点击 dismiss 后告警消失
    await waitFor(() => {
      expect(screen.queryByLabelText("dismiss")).not.toBeInTheDocument();
    });
  });

  // ─── mkt 切换时序 ───

  it("should await settings save before triggering refresh on mkt change", async () => {
    const callOrder: string[] = [];

    // 模拟 updateSettings 有延迟
    mockUpdateSettings.mockImplementation(async () => {
      callOrder.push("updateSettings");
    });

    const mockOnLanguageChange = vi.fn(() => {
      callOrder.push("onLanguageChange");
    });

    renderWithTheme(
      <Settings
        onClose={mockOnClose}
        onLanguageChange={mockOnLanguageChange}
      />,
    );

    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    // 找到 mkt 下拉选择器并切换
    const mktSelect = screen.getByDisplayValue(/中国大陆/i);
    fireEvent.change(mktSelect, { target: { value: "ja-JP" } });

    // 等待两个回调都执行完成
    await waitFor(() => {
      expect(mockOnLanguageChange).toHaveBeenCalled();
    });

    // 验证顺序：updateSettings 必须在 onLanguageChange 之前
    expect(callOrder).toEqual(["updateSettings", "onLanguageChange"]);
  });

  it("should update mkt setting when market dropdown is changed", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    await screen.findByText(/选择文件夹/i, {}, { timeout: 3000 });

    const mktSelect = screen.getByDisplayValue(/中国大陆/i);
    fireEvent.change(mktSelect, { target: { value: "en-US" } });

    await waitFor(() => {
      expect(mockUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          mkt: "en-US",
        }),
      );
    });
  });

  // ─── Helper: mock invoke with extra command handler ───

  function mockInvokeWith(handler: (cmd: string) => unknown | undefined) {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_settings") return Promise.resolve(mockSettings);
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
      const result = handler(cmd);
      if (result !== undefined) return result as Promise<unknown>;
      return Promise.resolve(undefined);
    });
  }

  // ─── 数据导入 ───

  it("should render import data section", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    await screen.findByText(/数据导入/i, {}, { timeout: 3000 });

    const importButton = screen.getByText(/选择目录并导入/i);
    expect(importButton).toBeInTheDocument();
    expect(importButton).not.toBeDisabled();
  });

  it("should open directory picker when import button clicked", async () => {
    const mockOpen = vi.fn().mockResolvedValue(null);
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalledWith(
        expect.objectContaining({
          directory: true,
          multiple: false,
        }),
      );
    });
  });

  it("should show success message after successful import", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/import/source");
    mockInvokeWith((cmd) => {
      if (cmd === "import_wallpapers") {
        return Promise.resolve({
          metadata_new: 35,
          metadata_updated: 7,
          metadata_skipped: 0,
          images_copied: 10,
          images_skipped: 5,
          images_failed: 0,
          mkt_count: 2,
        });
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(screen.getByText(/新增 35 条/)).toBeInTheDocument();
      expect(screen.getByText(/更新 7 条/)).toBeInTheDocument();
    });
  });

  it("should show up-to-date message when nothing changed", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/import/source");
    mockInvokeWith((cmd) => {
      if (cmd === "import_wallpapers") {
        return Promise.resolve({
          metadata_new: 0,
          metadata_updated: 0,
          metadata_skipped: 0,
          images_copied: 0,
          images_skipped: 5,
          images_failed: 0,
          mkt_count: 1,
        });
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(screen.getByText(/所有数据已是最新/)).toBeInTheDocument();
    });
  });

  it("should not show up-to-date message when import has skipped metadata", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/import/source");
    mockInvokeWith((cmd) => {
      if (cmd === "import_wallpapers") {
        return Promise.resolve({
          metadata_new: 0,
          metadata_updated: 0,
          metadata_skipped: 3,
          images_copied: 0,
          images_skipped: 0,
          images_failed: 0,
          mkt_count: 1,
        });
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(screen.getByText(/新增 0 条/)).toBeInTheDocument();
      expect(screen.queryByText(/所有数据已是最新/)).not.toBeInTheDocument();
    });
  });

  it("should show error message on import failure", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/bad/source");
    mockInvokeWith((cmd) => {
      if (cmd === "import_wallpapers") {
        return Promise.reject("SAME_DIRECTORY");
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(screen.getByText(/不能从当前壁纸目录导入/)).toBeInTheDocument();
    });
  });

  it("should not import when directory selection is cancelled", async () => {
    const mockOpen = vi.fn().mockResolvedValue(null);
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const importButton = await screen.findByText(
      /选择目录并导入/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(importButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalled();
    });

    expect(invoke).not.toHaveBeenCalledWith(
      "import_wallpapers",
      expect.anything(),
    );
  });

  // ─── 数据导出 ───

  it("should render export data section", async () => {
    renderWithTheme(<Settings onClose={mockOnClose} />);

    await screen.findByText(/数据导出/i, {}, { timeout: 3000 });

    const exportButton = screen.getByText(/选择目录并导出/i);
    expect(exportButton).toBeInTheDocument();
    expect(exportButton).not.toBeDisabled();
  });

  it("should open directory picker when export button clicked", async () => {
    const mockOpen = vi.fn().mockResolvedValue(null);
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const exportButton = await screen.findByText(
      /选择目录并导出/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalledWith(
        expect.objectContaining({
          directory: true,
          multiple: false,
        }),
      );
    });
  });

  it("should show success message after successful export", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/export/target");
    mockInvokeWith((cmd) => {
      if (cmd === "export_wallpapers") {
        return Promise.resolve({
          metadata_new: 25,
          metadata_updated: 5,
          metadata_skipped: 0,
          images_copied: 8,
          images_skipped: 3,
          images_failed: 0,
          mkt_count: 1,
        });
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const exportButton = await screen.findByText(
      /选择目录并导出/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(screen.getByText(/新增 25 条/)).toBeInTheDocument();
    });
  });

  it("should not show up-to-date message when export has failed images", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/export/target");
    mockInvokeWith((cmd) => {
      if (cmd === "export_wallpapers") {
        return Promise.resolve({
          metadata_new: 0,
          metadata_updated: 0,
          metadata_skipped: 0,
          images_copied: 0,
          images_skipped: 0,
          images_failed: 2,
          mkt_count: 1,
        });
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const exportButton = await screen.findByText(
      /选择目录并导出/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(screen.getByText(/2 张图片复制失败/)).toBeInTheDocument();
      expect(
        screen.queryByText(/目标目录数据已是最新/),
      ).not.toBeInTheDocument();
    });
  });

  it("should show error message on export to same directory", async () => {
    vi.mocked(dialog.open).mockResolvedValue("/same/dir");
    mockInvokeWith((cmd) => {
      if (cmd === "export_wallpapers") {
        return Promise.reject("SAME_DIRECTORY");
      }
    });

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const exportButton = await screen.findByText(
      /选择目录并导出/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(screen.getByText(/不能导出到当前壁纸目录/)).toBeInTheDocument();
    });
  });

  it("should not export when directory selection is cancelled", async () => {
    const mockOpen = vi.fn().mockResolvedValue(null);
    vi.mocked(dialog.open).mockImplementation(mockOpen);

    renderWithTheme(<Settings onClose={mockOnClose} />);

    const exportButton = await screen.findByText(
      /选择目录并导出/i,
      {},
      { timeout: 3000 },
    );
    fireEvent.click(exportButton);

    await waitFor(() => {
      expect(mockOpen).toHaveBeenCalled();
    });

    expect(invoke).not.toHaveBeenCalledWith(
      "export_wallpapers",
      expect.anything(),
    );
  });
});
