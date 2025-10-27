import { memo, useCallback } from "react";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
}

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

export const WallpaperGrid = memo(function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
}: WallpaperGridProps) {
  // 使用 useCallback 避免传递不稳定的 props
  const handleSetWallpaper = useCallback(
    (wallpaper: LocalWallpaper) => {
      onSetWallpaper(wallpaper);
    },
    [onSetWallpaper],
  );

  if (loading) {
    return (
      <>
        <div className="wallpaper-grid-loading">
          <p>加载中...</p>
        </div>
        <div className="wallpaper-grid">
          {/* 显示 8 个骨架屏 */}
          {Array.from({ length: 8 }, (_, i) => (
            <SkeletonCard key={i} />
          ))}
        </div>
      </>
    );
  }

  if (wallpapers.length === 0) {
    return (
      <div className="wallpaper-grid-empty">
        <p>暂无壁纸</p>
      </div>
    );
  }

  return (
    <div className="wallpaper-grid">
      {wallpapers.map((wallpaper) => (
        <WallpaperCard
          key={wallpaper.id}
          wallpaper={wallpaper}
          onSetWallpaper={handleSetWallpaper}
        />
      ))}
    </div>
  );
});
