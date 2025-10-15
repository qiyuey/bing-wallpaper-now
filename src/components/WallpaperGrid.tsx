import { WallpaperCard } from "./WallpaperCard";
import { LocalWallpaper } from "../types";

interface WallpaperGridProps {
  wallpapers: LocalWallpaper[];
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
  loading?: boolean;
}

export function WallpaperGrid({
  wallpapers,
  onSetWallpaper,
  loading = false,
}: WallpaperGridProps) {
  if (loading) {
    return (
      <div className="wallpaper-grid-loading">
        <p>加载中...</p>
      </div>
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
      {wallpapers.map((wallpaper) => {
        return (
          <WallpaperCard
            key={wallpaper.id}
            wallpaper={wallpaper}
            onSetWallpaper={() => onSetWallpaper(wallpaper)}
          />
        );
      })}
    </div>
  );
}
