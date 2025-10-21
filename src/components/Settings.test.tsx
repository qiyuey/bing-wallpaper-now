import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Settings } from "./Settings";

vi.mock("@tauri-apps/api/core");
vi.mock("@tauri-apps/plugin-dialog");

describe("Settings", () => {
  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render settings modal", () => {
    render(<Settings onClose={mockOnClose} />);

    // Should show either the settings content or loading state
    expect(screen.getByText(/设置|加载设置中.../i)).toBeInTheDocument();
  });

  it("should close when cancel button is clicked", async () => {
    render(<Settings onClose={mockOnClose} />);

    // Wait for settings to load, then find cancel button
    const cancelButton = await screen.findByText("取消", {}, { timeout: 3000 });
    fireEvent.click(cancelButton);

    expect(mockOnClose).toHaveBeenCalled();
  });

  it("should have form inputs when loaded", async () => {
    render(<Settings onClose={mockOnClose} />);

    // Wait for the auto-update checkbox to appear
    const checkbox = await screen.findByLabelText(
      /自动更新/i,
      {},
      { timeout: 3000 },
    );

    expect(checkbox).toBeInTheDocument();
  });
});
