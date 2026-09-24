import { describe, it, expect, vi } from "vitest";
import { screen, waitFor } from "@testing-library/react";
import { renderWithI18n } from "../test/browser-utils";
import gridStyles from "./WallpaperGrid.module.css";
import fixtureStyles from "../test/browser-fixture.module.css";
import { WallpaperGrid } from "./WallpaperGrid";
import { LocalWallpaper } from "../types";

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

  const manyWallpapers = Array.from({ length: 90 }, (_, index) => ({
    ...mockWallpapers[0],
    end_date: String(20240101 + index),
    title: `Wallpaper ${index + 1}`,
  }));

  it("remeasures rows across container breakpoints without overlapping cards", async () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={manyWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    // Change only the container width: native ResizeObserver must drive this.
    for (const [className, columns] of [
      [fixtureStyles.viewport, 3],
      [`${fixtureStyles.viewport} ${fixtureStyles.narrow}`, 1],
      [`${fixtureStyles.viewport} ${fixtureStyles.medium}`, 2],
      [fixtureStyles.viewport, 3],
    ] as const) {
      container.className = className;
      await waitFor(() => {
        const rows = container.querySelectorAll(`.${gridStyles.row}`);
        expect(rows.length).toBeGreaterThan(1);
        expect(rows[0].querySelectorAll("h3")).toHaveLength(columns);
        const firstCard = rows[0].firstElementChild!.getBoundingClientRect();
        const nextCard = rows[1].firstElementChild!.getBoundingClientRect();
        expect(firstCard.height).toBeGreaterThan(200);
        expect(nextCard.top).toBeGreaterThanOrEqual(firstCard.bottom);
      });
    }
  });

  it("virtualizes a large collection and renders the last card after scrolling", async () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={manyWallpapers}
        onSetWallpaper={mockOnSetWallpaper}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    await waitFor(() => {
      expect(screen.getByText("Wallpaper 1")).toBeVisible();
      expect(container.querySelectorAll("h3").length).toBeLessThan(90);
    });
    expect(screen.queryByText("Wallpaper 90")).not.toBeInTheDocument();
    const list = container.querySelector<HTMLElement>(
      `.${gridStyles.virtualList}`,
    )!;
    // Dynamic measurement may refine the scroll height while scrolling.
    await waitFor(() => {
      list.scrollTo({ top: list.scrollHeight, behavior: "instant" });
      expect(screen.getByText("Wallpaper 90")).toBeVisible();
    });
    expect(container.querySelectorAll("h3").length).toBeLessThan(90);
    expect(screen.queryByText("Wallpaper 1")).not.toBeInTheDocument();
  });

  it("should render loading state when loading is true", () => {
    const { container } = renderWithI18n(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={true}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    expect(container.firstElementChild).toBeInTheDocument();
    expect(screen.queryByText("暂无壁纸")).not.toBeInTheDocument();
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

  it("should suppress empty state when requested", () => {
    renderWithI18n(
      <WallpaperGrid
        wallpapers={[]}
        onSetWallpaper={mockOnSetWallpaper}
        loading={false}
        showEmptyState={false}
        wallpaperDirectory={mockWallpaperDirectory}
      />,
    );

    expect(screen.queryByText("暂无壁纸")).not.toBeInTheDocument();
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
      const wallpaperCards = container.querySelectorAll("h3");
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

    const virtualContainer = container.firstElementChild;
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
