import { describe, it, expect, beforeEach, vi } from "vitest";
import { screen, fireEvent, waitFor } from "@testing-library/react";
import { UpdateDialog } from "./UpdateDialog";
import { renderWithI18n } from "../test/test-utils";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/core");

describe("UpdateDialog", () => {
  const mockOnClose = vi.fn();
  const mockOnIgnore = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render update dialog with version", () => {
    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    expect(
      screen.getByText(/有新版本可用|Update Available/),
    ).toBeInTheDocument();
    expect(screen.getByText(/0\.5\.0/)).toBeInTheDocument();
  });

  it("should call openUrl when go to update button is clicked", async () => {
    vi.mocked(openUrl).mockResolvedValue(undefined);

    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    const updateButton = screen.getByRole("button", {
      name: /前往更新|Go to Update/,
    });
    fireEvent.click(updateButton);

    await waitFor(() => {
      expect(openUrl).toHaveBeenCalledWith(
        "https://github.com/example/releases/tag/0.5.0",
      );
      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  it("should call invoke and callbacks when ignore button is clicked", async () => {
    vi.mocked(invoke).mockResolvedValue(undefined);

    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    const ignoreButton = screen.getByRole("button", {
      name: /此版本不再提醒|Ignore This Version/,
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

    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    const ignoreButton = screen.getByRole("button", {
      name: /此版本不再提醒|Ignore This Version/,
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
    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    const closeButton = screen.getByRole("button", { name: /关闭|Close/ });
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should handle openUrl error gracefully", async () => {
    vi.mocked(openUrl).mockRejectedValue(new Error("Failed to open URL"));

    renderWithI18n(
      <UpdateDialog
        version="0.5.0"
        releaseUrl="https://github.com/example/releases/tag/0.5.0"
        onClose={mockOnClose}
        onIgnore={mockOnIgnore}
      />,
    );

    const updateButton = screen.getByRole("button", {
      name: /前往更新|Go to Update/,
    });
    fireEvent.click(updateButton);

    await waitFor(() => {
      expect(openUrl).toHaveBeenCalled();
      expect(mockOnClose).toHaveBeenCalled();
    });
  });
});
