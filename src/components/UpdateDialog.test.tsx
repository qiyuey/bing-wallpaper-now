import { describe, it, expect, beforeEach, vi } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { UpdateDialog } from "./UpdateDialog";
import { renderWithI18n } from "../test/test-utils";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { Update, DownloadEvent } from "@tauri-apps/plugin-updater";

vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-process");
vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
  Update: vi.fn(),
}));

function createMockUpdate(
  overrides?: Partial<{
    downloadAndInstall: (
      onEvent?: (progress: DownloadEvent) => void,
    ) => Promise<void>;
  }>,
): Update {
  return {
    available: true,
    currentVersion: "0.4.0",
    version: "0.5.0",
    body: "Release notes",
    rawJson: {},
    downloadAndInstall:
      overrides?.downloadAndInstall ?? vi.fn().mockResolvedValue(undefined),
    download: vi.fn().mockResolvedValue(undefined),
    install: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
  } as unknown as Update;
}

describe("UpdateDialog", () => {
  const mockOnClose = vi.fn();
  const mockOnIgnore = vi.fn();

  let mockUpdate: Update;

  const getDefaultProps = () => ({
    version: "0.5.0",
    body: "Release notes",
    update: mockUpdate,
    onClose: mockOnClose,
    onIgnore: mockOnIgnore,
  });

  beforeEach(() => {
    vi.clearAllMocks();
    mockUpdate = createMockUpdate();
    vi.mocked(relaunch).mockResolvedValue(undefined);
  });

  it("should render update dialog with version", () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    expect(
      screen.getByText(/有新版本可用|Update Available/),
    ).toBeInTheDocument();
    expect(screen.getByText(/0\.5\.0/)).toBeInTheDocument();
  });

  it("should call openUrl when view release button is clicked", async () => {
    vi.mocked(openUrl).mockResolvedValue(undefined);

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const viewButton = screen.getByRole("button", {
      name: /详情|View/,
    });
    fireEvent.click(viewButton);

    await waitFor(() => {
      expect(openUrl).toHaveBeenCalledWith(
        "https://github.com/qiyuey/bing-wallpaper-now/releases/tag/0.5.0",
      );
    });
  });

  it("should show install button", () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    expect(
      screen.getByRole("button", {
        name: /安装|Install/,
      }),
    ).toBeInTheDocument();
  });

  it("should call update.downloadAndInstall when install button is clicked", async () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const installButton = screen.getByRole("button", {
      name: /安装|Install/,
    });
    fireEvent.click(installButton);

    await waitFor(() => {
      expect(mockUpdate.downloadAndInstall).toHaveBeenCalledWith(
        expect.any(Function),
      );
    });
  });

  it("should call relaunch after successful download and install", async () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(relaunch).toHaveBeenCalled();
    });
  });

  it("should show error when download fails", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockRejectedValue(new Error("Network error")),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(screen.getByText(/下载失败|Download Failed/)).toBeInTheDocument();
    });
  });

  it("should clear progress state after download fails", async () => {
    let progressCallback: ((progress: DownloadEvent) => void) | undefined;

    mockUpdate = createMockUpdate({
      downloadAndInstall: vi
        .fn()
        .mockImplementation((onEvent?: (progress: DownloadEvent) => void) => {
          progressCallback = onEvent;
          return new Promise((_resolve, reject) => {
            setTimeout(() => reject(new Error("Connection lost")), 50);
          });
        }),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(progressCallback).toBeDefined();
    });

    progressCallback!({ event: "Started", data: { contentLength: 100000 } });
    progressCallback!({ event: "Progress", data: { chunkLength: 75000 } });

    await waitFor(() => {
      expect(screen.getByText(/75%/)).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText(/下载失败|Download Failed/)).toBeInTheDocument();
      expect(screen.queryByText(/75%/)).not.toBeInTheDocument();
    });
  });

  it("should call invoke and callbacks when ignore button is clicked", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const ignoreButton = screen.getByRole("button", {
      name: /忽略|Ignore/,
    });
    fireEvent.click(ignoreButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("add_ignored_update_version", {
        version: "0.5.0",
      });
    });
    expect(mockOnIgnore).toHaveBeenCalled();
    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should close dialog when ignore fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("Failed"));

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const ignoreButton = screen.getByRole("button", {
      name: /忽略|Ignore/,
    });
    fireEvent.click(ignoreButton);

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("add_ignored_update_version", {
        version: "0.5.0",
      });
    });
    expect(mockOnIgnore).not.toHaveBeenCalled();
    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should call onClose when close button is clicked", () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const closeButton = screen.getByRole("button", { name: /关闭|Close/ });
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should handle openUrl error gracefully", async () => {
    vi.mocked(openUrl).mockRejectedValue(new Error("Failed to open URL"));

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    const viewButton = screen.getByRole("button", {
      name: /详情|View/,
    });
    fireEvent.click(viewButton);

    await waitFor(() => {
      expect(openUrl).toHaveBeenCalled();
    });
  });

  it("should keep close button and show cancel button while downloading", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /关闭|Close/ }),
      ).toBeInTheDocument();
      expect(
        screen.getByRole("button", { name: /取消下载|Cancel Download/ }),
      ).toBeInTheDocument();
      expect(
        screen.queryByRole("button", { name: /忽略|Ignore/ }),
      ).not.toBeInTheDocument();
    });
  });

  it("should call onClose when cancel download button is clicked during download", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /取消下载|Cancel Download/ }),
      ).toBeInTheDocument();
    });

    fireEvent.click(
      screen.getByRole("button", { name: /取消下载|Cancel Download/ }),
    );

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should call onClose when close button is clicked during download", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /下载中|Downloading/ }),
      ).toBeDisabled();
    });

    const closeButton = screen.getByRole("button", { name: /关闭|Close/ });
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should not relaunch when download completes after cancel", async () => {
    let resolveDownload: () => void;
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockImplementation(() => {
        return new Promise<void>((resolve) => {
          resolveDownload = resolve;
        });
      }),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: /取消下载|Cancel Download/ }),
      ).toBeInTheDocument();
    });

    fireEvent.click(
      screen.getByRole("button", { name: /取消下载|Cancel Download/ }),
    );

    resolveDownload!();

    await waitFor(() => {
      expect(mockOnClose).toHaveBeenCalled();
    });
    expect(relaunch).not.toHaveBeenCalled();
  });

  it("should disable buttons while downloading", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: /详情|View/ })).toBeDisabled();
      expect(
        screen.getByRole("button", { name: /下载中|Downloading/ }),
      ).toBeDisabled();
    });
  });

  it("should show downloading status text when download starts", async () => {
    mockUpdate = createMockUpdate({
      downloadAndInstall: vi.fn().mockReturnValue(new Promise(() => {})),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      const statusTexts = screen.getAllByText(/下载中|Downloading/);
      expect(statusTexts.length).toBeGreaterThanOrEqual(1);
      const statusParagraph = statusTexts.find((el) => el.tagName === "P");
      expect(statusParagraph).toBeInTheDocument();
    });
  });

  it("should show download progress via event callback", async () => {
    let progressCallback: ((progress: DownloadEvent) => void) | undefined;

    mockUpdate = createMockUpdate({
      downloadAndInstall: vi
        .fn()
        .mockImplementation((onEvent?: (progress: DownloadEvent) => void) => {
          progressCallback = onEvent;
          return new Promise(() => {});
        }),
    });

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(progressCallback).toBeDefined();
    });

    progressCallback!({ event: "Started", data: { contentLength: 100000 } });
    progressCallback!({ event: "Progress", data: { chunkLength: 50000 } });

    await waitFor(() => {
      expect(screen.getByText(/50%/)).toBeInTheDocument();
    });
  });

  it("should show download complete status", async () => {
    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(
        screen.getByText(/下载完成|Download complete/),
      ).toBeInTheDocument();
    });
  });

  it("should handle relaunch failure gracefully", async () => {
    vi.mocked(relaunch).mockRejectedValue(new Error("Relaunch failed"));

    renderWithI18n(<UpdateDialog {...getDefaultProps()} />);

    fireEvent.click(screen.getByRole("button", { name: /安装|Install/ }));

    await waitFor(() => {
      expect(screen.getByText(/下载失败|Download Failed/)).toBeInTheDocument();
    });
  });
});
