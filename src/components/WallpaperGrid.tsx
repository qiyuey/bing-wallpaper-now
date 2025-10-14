import { WallpaperCard } from "./WallpaperCard";
import { BingImageEntry } from "../types";

interface WallpaperGridProps {
  images: BingImageEntry[];
  onSetWallpaper: (image: BingImageEntry) => void;
  loading?: boolean;
}

export function WallpaperGrid({
  images,
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

  if (images.length === 0) {
    return (
      <div className="wallpaper-grid-empty">
        <p>暂无壁纸</p>
      </div>
    );
  }

  return (
    <div className="wallpaper-grid">
      {images.map((image) => {
        return (
          <WallpaperCard
            key={image.hsh}
            image={image}
            onSetWallpaper={() => onSetWallpaper(image)}
          />
        );
      })}
    </div>
  );
}
