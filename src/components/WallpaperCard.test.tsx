import { describe, it, expect, vi, beforeEach } from "vitest";
import { screen, fireEvent } from "@testing-library/react";
import { renderWithI18n } from "../test/test-utils";
import { WallpaperCard } from "./WallpaperCard";
import type { LocalWallpaper } from "../types";

vi.mock("@tauri-apps/plugin-opener");
vi.mock("@tauri-apps/api/core", () => {
  const globalWindow = global.window as {
    __TAURI_INTERNALS__?: {
      invoke: (cmd: string, args?: unknown) => Promise<unknown>;
    };
  };
  return {
    invoke: (cmd: string, args?: unknown) => {
      if (globalWindow && globalWindow.__TAURI_INTERNALS__) {
        return globalWindow.__TAURI_INTERNALS__.invoke(cmd, args);
      }
      return Promise.resolve(undefined);
    },
    convertFileSrc: vi.fn((path: string) => `asset://localhost/${path}`),
  };
});
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
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
    renderWithI18n(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    expect(screen.getByText("测试壁纸")).toBeInTheDocument();
    expect(screen.getByText("测试地点")).toBeInTheDocument();
  });

  it("should render wallpaper image", () => {
    renderWithI18n(
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
    renderWithI18n(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    // 模拟图片加载完成
    const image = screen.getByAltText("测试壁纸") as HTMLImageElement;
    fireEvent.load(image);

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

    renderWithI18n(
      <WallpaperCard
        wallpaper={wallpaperWithCopyright}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    expect(screen.getByText("美景")).toBeInTheDocument();
    expect(screen.getByText("巴黎，法国")).toBeInTheDocument();
  });

  it("should call openUrl when image container is clicked", async () => {
    const { openUrl } = await import("@tauri-apps/plugin-opener");

    renderWithI18n(
      <WallpaperCard
        wallpaper={mockWallpaper}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    const imageContainer = screen.getByTitle("点击查看详情");
    fireEvent.click(imageContainer);

    expect(openUrl).toHaveBeenCalledWith(mockWallpaper.copyright_link);
  });

  it("should handle click when copyright_link is empty", () => {
    const wallpaperWithoutLink = { ...mockWallpaper, copyright_link: "" };

    renderWithI18n(
      <WallpaperCard
        wallpaper={wallpaperWithoutLink}
        onSetWallpaper={mockOnSetWallpaper}
      />,
    );

    const imageContainer = screen.getByTitle("点击查看详情");
    fireEvent.click(imageContainer);

    // Should not throw error
    expect(screen.getByText("测试壁纸")).toBeInTheDocument();
  });
});
