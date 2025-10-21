import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { WallpaperCard } from "./WallpaperCard";
import type { LocalWallpaper } from "../types";

vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
}));

describe("WallpaperCard", () => {
  const mockWallpaper: LocalWallpaper = {
    id: "20240101",
    start_date: "20240101",
    end_date: "20240102",
    title: "测试壁纸",
    copyright: "测试地点 (测试作者)",
    copyright_link: "https://example.com/details",
    file_path: "/path/to/wallpaper.jpg",
    download_time: "2024-01-01T00:00:00Z",
  };

  const mockOnSetWallpaper = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should render wallpaper card with title and subtitle", () => {
    render(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    expect(screen.getByText("测试壁纸")).toBeInTheDocument();
    expect(screen.getByText("测试地点")).toBeInTheDocument();
  });

  it("should render wallpaper image", () => {
    render(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    const image = screen.getByAltText("测试壁纸") as HTMLImageElement;
    expect(image).toBeInTheDocument();
    expect(image.src).toContain("asset://localhost/");
  });

  it("should call onSetWallpaper when button is clicked", () => {
    render(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    const button = screen.getByRole("button", { name: /设置壁纸/i });
    fireEvent.click(button);

    expect(mockOnSetWallpaper).toHaveBeenCalledTimes(1);
  });

  it("should parse copyright correctly", () => {
    const wallpaperWithCopyright = {
      ...mockWallpaper,
      title: "美景",
      copyright: "巴黎，法国 (摄影师名字)",
    };

    render(
      <WallpaperCard
        wallpaper={wallpaperWithCopyright}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    expect(screen.getByText("美景")).toBeInTheDocument();
    expect(screen.getByText("巴黎，法国")).toBeInTheDocument();
  });
});
