import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { WallpaperGrid } from "./WallpaperGrid";
import { LocalWallpaper } from "../types";

// Mock window size and element dimensions for virtual list
beforeEach(() => {
  Object.defineProperty(window, "innerHeight", {
    writable: true,
    configurable: true,
    value: 800,
  });

  // Mock offsetWidth and offsetHeight for container
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
});

describe("WallpaperGrid", () => {
  const mockWallpapers: LocalWallpaper[] = [
    {
      id: "20240101",
      start_date: "20240101",
      end_date: "20240102",
      title: "Test Wallpaper 1",
      copyright: "Test Copyright 1",
      copyright_link: "https://example.com/link1",
      file_path: "/path/to/wallpaper1.jpg",
      download_time: "2024-01-01T00:00:00Z",
    },
    {
      id: "20240102",
      start_date: "20240102",
      end_date: "20240103",
      title: "Test Wallpaper 2",
      copyright: "Test Copyright 2",
      copyright_link: "https://example.com/link2",
      file_path: "/path/to/wallpaper2.jpg",
      download_time: "2024-01-02T00:00:00Z",
    },
  ];

  const mockOnSetWallpaper = vi.fn();

  it("should render loading state when loading is true", () => {
    const { container } = render(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={true}
      />,
    );

    const spinner = container.querySelector(".spinner");
    expect(spinner).toBeInTheDocument();
    const loadingDiv = container.querySelector(".wallpaper-grid-loading");
    expect(loadingDiv).toBeInTheDocument();
  });

  it("should render empty state when no wallpapers are provided", () => {
    render(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    expect(screen.getByText("暂无壁纸")).toBeInTheDocument();
    expect(
      screen.getByText("点击上方刷新按钮获取最新壁纸"),
    ).toBeInTheDocument();
  });

  it("should render wallpapers when provided", async () => {
    render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    });
    expect(screen.getByText("Test Wallpaper 2")).toBeInTheDocument();
  });

  it("should render correct number of wallpaper cards", async () => {
    const { container } = render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    await waitFor(() => {
      const wallpaperCards = container.querySelectorAll(".wallpaper-card");
      expect(wallpaperCards.length).toBe(mockWallpapers.length);
    });
  });

  it("should render virtual list container", () => {
    const { container } = render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    const virtualContainer = container.querySelector(".wallpaper-container");
    expect(virtualContainer).toBeInTheDocument();
  });

  it("should default loading to false when not provided", () => {
    render(
      <WallpaperGrid wallpapers={[]} onSetWallpaper={mockOnSetWallpaper} />,
    );

    // 当 loading 默认为 false 且没有壁纸时，应该显示空状态
    expect(screen.getByText("暂无壁纸")).toBeInTheDocument();
    expect(
      screen.getByText("点击上方刷新按钮获取最新壁纸"),
    ).toBeInTheDocument();
  });

  it("should render wallpaper grid with single wallpaper", async () => {
    const singleWallpaper = [mockWallpapers[0]];

    render(
      <WallpaperGrid
        wallpapers={singleWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    });
    expect(screen.queryByText("Test Wallpaper 2")).not.toBeInTheDocument();
  });
});
