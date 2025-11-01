import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import { renderWithI18n } from "../test/test-utils";
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
      end_date: "20240102",
      title: "Test Wallpaper 1",
      copyright: "Test Copyright 1",
      copyright_link: "https://example.com/link1",
      urlbase: "/th?id=OHR.Test1",
    },
    {
      end_date: "20240103",
      title: "Test Wallpaper 2",
      copyright: "Test Copyright 2",
      copyright_link: "https://example.com/link2",
      urlbase: "/th?id=OHR.Test2",
    },
  ];

  const mockOnSetWallpaper = vi.fn();
  const mockWallpaperDirectory = "/path/to/wallpapers";

  it("should render loading state when loading is true", () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={true}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    const spinner = container.querySelector(".spinner");
    expect(spinner).toBeInTheDocument();
    const loadingDiv = container.querySelector(".wallpaper-grid-loading");
    expect(loadingDiv).toBeInTheDocument();
  });

  it("should render empty state when no wallpapers are provided", () => {
    renderWithI18n(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    expect(screen.getByText("暂无壁纸")).toBeInTheDocument();
    expect(
      screen.getByText(/点击上方刷新按钮获取最新壁纸/),
    ).toBeInTheDocument();
  });

  it("should render wallpapers when provided", async () => {
    renderWithI18n(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    });
    expect(screen.getByText("Test Wallpaper 2")).toBeInTheDocument();
  });

  it("should render correct number of wallpaper cards", async () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    await waitFor(() => {
      const wallpaperCards = container.querySelectorAll(".wallpaper-card");
      expect(wallpaperCards.length).toBe(mockWallpapers.length);
    });
  });

  it("should render virtual list container", () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    const virtualContainer = container.querySelector(".wallpaper-container");
    expect(virtualContainer).toBeInTheDocument();
  });

  it("should default loading to false when not provided", () => {
    renderWithI18n(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    // 当 loading 默认为 false 且没有壁纸时，应该显示空状态
    expect(screen.getByText("暂无壁纸")).toBeInTheDocument();
    expect(
      screen.getByText(/点击上方刷新按钮获取最新壁纸/),
    ).toBeInTheDocument();
  });

  it("should render wallpaper grid with single wallpaper", async () => {
    const singleWallpaper = [mockWallpapers[0]];

    renderWithI18n(
      <WallpaperGrid
        wallpapers={singleWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    });
    expect(screen.queryByText("Test Wallpaper 2")).not.toBeInTheDocument();
  });
});
