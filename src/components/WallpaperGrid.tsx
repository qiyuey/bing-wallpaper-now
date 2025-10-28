import { memo, useCallback, useState, useEffect, useRef } from "react";
import { List, RowComponentProps } from "react-window";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
}

// 行配置
const ROW_HEIGHT = 455; // 每行高度（图片280px + 内容区域 + 间距）
const CARDS_PER_ROW_DESKTOP = 3; // 桌面端每行3张
const CARDS_PER_ROW_TABLET = 2; // 平板端每行2张
const CARDS_PER_ROW_MOBILE = 1; // 移动端每行1张

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
}

export const WallpaperGrid = memo(function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
}: WallpaperGridProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [containerWidth, setContainerWidth] = useState(0);
  const [containerHeight, setContainerHeight] = useState(600);
  const [cardsPerRow, setCardsPerRow] = useState(CARDS_PER_ROW_DESKTOP);

  // 监听容器尺寸变化（使用 ResizeObserver 自动获取可用高度）
  useEffect(() => {
    const updateSize = () => {
      if (containerRef.current) {
        const width = containerRef.current.offsetWidth;
        const height = containerRef.current.offsetHeight;
        setContainerWidth(width);
        setContainerHeight(height);

        // 根据宽度决定每行卡片数
        if (width < 768) {
          setCardsPerRow(CARDS_PER_ROW_MOBILE);
        } else if (width < 1200) {
          setCardsPerRow(CARDS_PER_ROW_TABLET);
        } else {
          setCardsPerRow(CARDS_PER_ROW_DESKTOP);
        }
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
  }: RowComponentProps<RowData>) => {
    const startIndex = index * cardsPerRow;
    const rowWallpapers = wallpapers.slice(
      startIndex,
      startIndex + cardsPerRow,
    );

    return (
      <div style={style} className="wallpaper-row">
        {rowWallpapers.map((wallpaper: LocalWallpaper) => (
          <div key={wallpaper.id} className="wallpaper-row-item">
            <WallpaperCard
              wallpaper={wallpaper}
              onSetWallpaper={onSetWallpaper}
            />
          </div>
        ))}
      </div>
    );
  };

  if (loading || wallpapers.length === 0) {
    return (
      <div ref={containerRef} className="wallpaper-container">
        <div className="wallpaper-grid-loading">
          <div className="spinner"></div>
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
          }}
        />
      )}
    </div>
  );
});
