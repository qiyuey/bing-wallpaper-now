import { memo, useCallback } from "react";
import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
  isFirstLoad?: boolean;
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
  isFirstLoad = false,
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
          {isFirstLoad && (
            <p
              style={{
                fontSize: "14px",
                color: "#666",
                marginTop: "8px",
                lineHeight: "1.5",
              }}
            >
              首次加载需下载壁纸，请稍候...
              <br />
              <span style={{ fontSize: "12px", color: "#999" }}>
                正在从 Bing 获取今日精美壁纸
              </span>
            </p>
          )}
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
        {isFirstLoad && (
          <p
            style={{
              fontSize: "14px",
              color: "#666",
              marginTop: "12px",
              padding: "16px",
              background: "#f5f5f5",
              borderRadius: "8px",
              lineHeight: "1.6",
            }}
          >
            🎨 首次启动需要下载壁纸，请耐心等待...
            <br />
            <span style={{ fontSize: "13px", color: "#999" }}>
              正在从 Bing 下载最新的高清壁纸（约 2-3MB）
            </span>
          </p>
        )}
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
