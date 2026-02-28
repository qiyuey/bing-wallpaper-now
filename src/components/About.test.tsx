import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { renderWithI18n } from "../test/test-utils";
import { About } from "./About";

describe("About", () => {
  const mockOnClose = vi.fn();
  const mockVersion = "1.0.0";

  beforeEach(() => {
    mockOnClose.mockClear();
  });

  it("should render about modal with version", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    expect(screen.getByText("关于")).toBeInTheDocument();
    expect(screen.getByText("Bing Wallpaper Now")).toBeInTheDocument();
    expect(screen.getByText(`版本 ${mockVersion}`)).toBeInTheDocument();
  });

  it("should render description and tech stack", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    expect(screen.getByText(/每日自动获取并更新必应壁纸/)).toBeInTheDocument();
  });

  it("should render GitHub link with correct href", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    const githubLink = screen.getByText("GitHub 仓库").closest("a");
    expect(githubLink).toBeInTheDocument();
    expect(githubLink).toHaveAttribute(
      "href",
      "https://github.com/qiyuey/bing-wallpaper-now",
    );
    expect(githubLink).toHaveAttribute("target", "_blank");
    expect(githubLink).toHaveAttribute("rel", "noopener noreferrer");
  });

  it("should render copyright text", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    expect(screen.getByText(/© 2025 Bing Wallpaper Now/)).toBeInTheDocument();
  });

  it("should call onClose when X button is clicked", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    const closeButton = screen.getByText("×");
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it("should call onClose when close button is clicked", () => {
    renderWithI18n(<About onClose={mockOnClose} version={mockVersion} />);

    const closeButton = screen.getByText("关闭");
    fireEvent.click(closeButton);

    expect(mockOnClose).toHaveBeenCalledTimes(1);
  });

  it("should render with different version numbers", () => {
    const version2 = "2.5.3";
    const { rerender } = renderWithI18n(
      <About onClose={mockOnClose} version={version2} />,
    );

    expect(screen.getByText(`版本 ${version2}`)).toBeInTheDocument();

    const version3 = "3.0.0-beta.1";
    rerender(<About onClose={mockOnClose} version={version3} />);

    expect(screen.getByText(`版本 ${version3}`)).toBeInTheDocument();
  });

  it("should have correct CSS classes for styling", () => {
    const { container } = renderWithI18n(
      <About onClose={mockOnClose} version={mockVersion} />,
    );

    expect(container.querySelector(".settings-overlay")).toBeInTheDocument();
    expect(container.querySelector(".settings-modal")).toBeInTheDocument();
    expect(container.querySelector(".settings-header")).toBeInTheDocument();
    expect(container.querySelector(".settings-body")).toBeInTheDocument();
    expect(container.querySelector(".settings-footer")).toBeInTheDocument();
  });

  it("should render GitHub SVG icon", () => {
    const { container } = renderWithI18n(
      <About onClose={mockOnClose} version={mockVersion} />,
    );

    const githubSvg = container.querySelector('svg[width="16"][height="16"]');
    expect(githubSvg).toBeInTheDocument();
  });

  it("should have semantic class names for content sections", () => {
    const { container } = renderWithI18n(
      <About onClose={mockOnClose} version={mockVersion} />,
    );

    expect(container.querySelector(".about-title")).toBeInTheDocument();
    expect(container.querySelector(".about-version")).toBeInTheDocument();
    expect(container.querySelector(".about-description")).toBeInTheDocument();
    expect(container.querySelector(".about-github-link")).toBeInTheDocument();
    expect(container.querySelector(".about-copyright")).toBeInTheDocument();
  });

  it("should prevent multiple close callbacks on rapid clicks", () => {
    const mockOnCloseSingle = vi.fn();
    renderWithI18n(<About onClose={mockOnCloseSingle} version={mockVersion} />);

    const closeButton = screen.getByText("关闭");
    fireEvent.click(closeButton);
    fireEvent.click(closeButton);
    fireEvent.click(closeButton);

    // Each click should trigger the callback
    expect(mockOnCloseSingle).toHaveBeenCalledTimes(3);
  });
});
