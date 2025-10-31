import { memo, useCallback, useState, useEffect, useRef } from "react";
import { List, RowComponentProps } from "react-window";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";
import {
  CARDS_PER_ROW,
  calculateRowHeight,
  getCardsPerRow,
} from "../config/layout";
import { CONTAINER } from "../config/ui";
import { useI18n } from "../i18n/I18nContext";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
  wallpaperDirectory: string;
}

// 使用配置计算行高
const ROW_HEIGHT = calculateRowHeight();

// 骨架屏组件
const SkeletonCard = memo(() => (
  <div className="wallpaper-card-skeleton">
    <div className="skeleton-image" />
    <div className="skeleton-text skeleton-text-title" />
    <div className="skeleton-text skeleton-text-subtitle" />
    <div className="skeleton-button" />
  </div>
));

SkeletonCard.displayName = "SkeletonCard";

// Row 渲染组件的额外数据类型
interface RowData {
  wallpapers: LocalWallpaper[];
  cardsPerRow: number;
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  wallpaperDirectory: string;
}

export const WallpaperGrid = memo(function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
  wallpaperDirectory,
}: WallpaperGridProps) {
  const { t } = useI18n();
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);
  const [containerHeight, setContainerHeight] = useState(
    CONTAINER.DEFAULT_HEIGHT,
  );
  const [cardsPerRow, setCardsPerRow] = useState<number>(CARDS_PER_ROW.DESKTOP);

  // 监听容器尺寸变化（使用 ResizeObserver 自动获取可用高度）
  useEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const width = containerRef.current.offsetWidth;
        const height = containerRef.current.offsetHeight;
        setContainerWidth(width);
        setContainerHeight(height);

        // 使用配置函数根据宽度决定每行卡片数
        setCardsPerRow(getCardsPerRow(width));
      }
    };

    updateSize();

    // 使用 ResizeObserver 监听容器尺寸变化
    const resizeObserver = new ResizeObserver(() => {
      updateSize();
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    // 也监听窗口 resize 作为备份
    window.addEventListener("resize", updateSize);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener("resize", updateSize);
    };
  }, []);

  const handleSetWallpaper = useCallback(
    (wallpaper: LocalWallpaper) => {
      onSetWallpaper(wallpaper);
    },
    [onSetWallpaper],
  );

  // Row 组件
  const Row = ({
    index,
    style,
    wallpapers,
    cardsPerRow,
    onSetWallpaper,
    wallpaperDirectory,
  }: RowComponentProps<RowData>) => {
    const startIndex = index * cardsPerRow;
    const rowWallpapers = wallpapers.slice(
      startIndex,
      startIndex + cardsPerRow,
    );

    return (
      <div style={style} className="wallpaper-row">
        {rowWallpapers.map((wallpaper: LocalWallpaper) => (
          <div
            key={`${wallpaper.end_date}-${index}`}
            className="wallpaper-row-item"
          >
            <WallpaperCard
              wallpaper={wallpaper}
              onSetWallpaper={onSetWallpaper}
              wallpaperDirectory={wallpaperDirectory}
            />
          </div>
        ))}
      </div>
    );
  };

  if (loading) {
    return (
      <div ref={containerRef} className="wallpaper-container">
        <div className="wallpaper-grid-loading">
          <div className="spinner"></div>
        </div>
      </div>
    );
  }

  if (wallpapers.length === 0) {
    return (
      <div ref={containerRef} className="wallpaper-container">
        <div className="wallpaper-grid-empty">
          <p>{t("noWallpapers")}</p>
          <p className="wallpaper-grid-empty-hint">{t("noWallpapersHint")}</p>
        </div>
      </div>
    );
  }

  // 计算总行数
  const rowCount = Math.ceil(wallpapers.length / cardsPerRow);

  return (
    <div ref={containerRef} className="wallpaper-container">
      {containerWidth > 0 && containerHeight > 0 && (
        <List<RowData>
          rowCount={rowCount}
          rowHeight={ROW_HEIGHT}
          className="wallpaper-virtual-list"
          style={{ width: containerWidth, height: containerHeight }}
          rowComponent={Row}
          rowProps={{
            wallpapers,
            cardsPerRow,
            onSetWallpaper: handleSetWallpaper,
            wallpaperDirectory,
          }}
        />
      )}
    </div>
  );
});
