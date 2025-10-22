import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { WallpaperGrid } from "./WallpaperGrid";
import { LocalWallpaper } from "../types";

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
    render(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={true}
      />,
    );

    expect(screen.getByText("加载中...")).toBeInTheDocument();
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
  });

  it("should render wallpapers when provided", () => {
    render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    expect(screen.getByText("Test Wallpaper 2")).toBeInTheDocument();
  });

  it("should render correct number of wallpaper cards", () => {
    const { container } = render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    const wallpaperCards = container.querySelectorAll(".wallpaper-card");
    expect(wallpaperCards.length).toBe(mockWallpapers.length);
  });

  it("should use wallpaper id as key", () => {
    const { container } = render(
      <WallpaperGrid
        wallpapers={mockWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    const grid = container.querySelector(".wallpaper-grid");
    expect(grid).toBeInTheDocument();
    expect(grid?.children.length).toBe(mockWallpapers.length);
  });

  it("should default loading to false when not provided", () => {
    render(
      <WallpaperGrid wallpapers={[]} onSetWallpaper={mockOnSetWallpaper} />,
    );

    expect(screen.getByText("暂无壁纸")).toBeInTheDocument();
  });

  it("should render wallpaper grid with single wallpaper", () => {
    const singleWallpaper = [mockWallpapers[0]];

    render(
      <WallpaperGrid
        wallpapers={singleWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
      />,
    );

    expect(screen.getByText("Test Wallpaper 1")).toBeInTheDocument();
    expect(screen.queryByText("Test Wallpaper 2")).not.toBeInTheDocument();
  });
});
