import { memo, useCallback, useMemo } from "react";
import { LocalWallpaper } from "../types";
import { openUrl } from "@tauri-apps/plugin-opener";
import { convertFileSrc } from "@tauri-apps/api/core";

interface WallpaperCardProps {
  wallpaper: LocalWallpaper;
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
}

// 使用 memo 优化，只在 props 变化时重新渲染
export const WallpaperCard = memo(function WallpaperCard({
  wallpaper,
  onSetWallpaper,
}: WallpaperCardProps) {
  // 使用 useCallback 避免函数重新创建
  const handleImageClick = useCallback(async () => {
    if (wallpaper.copyright_link) {
      try {
        await openUrl(wallpaper.copyright_link);
      } catch (err) {
        console.error("Failed to open link:", err);
      }
    }
  }, [wallpaper.copyright_link]);

  const handleSetWallpaper = useCallback(() => {
    onSetWallpaper(wallpaper);
  }, [wallpaper, onSetWallpaper]);

  // 解析标题和副标题（使用 useMemo 缓存结果）
  const { title, subtitle } = useMemo(() => {
    const title = wallpaper.title;
    const copyright = wallpaper.copyright;

    // 从 copyright 中提取括号外的内容作为副标题
    const match = copyright.match(/^([^(]+?)(?:\s*\(([^)]+)\))?$/);
    const subtitle = match ? match[1].trim() : copyright;

    return { title, subtitle };
  }, [wallpaper.title, wallpaper.copyright]);

  // 将本地文件路径转换为前端可访问的 URL（使用 useMemo 缓存）
  const imageUrl = useMemo(
    () => convertFileSrc(wallpaper.file_path),
    [wallpaper.file_path],
  );

  return (
    <div className="wallpaper-card">
      <div
        className="wallpaper-image-container"
        onClick={handleImageClick}
        style={{ cursor: "pointer" }}
        title="点击查看详情"
      >
        <img
          src={imageUrl}
          alt={title}
          className="wallpaper-image"
          loading="lazy"
          decoding="async"
        />
      </div>
      <div className="wallpaper-info">
        <h3 className="wallpaper-title">{title}</h3>
        {subtitle && <p className="wallpaper-subtitle">{subtitle}</p>}
      </div>
      <div className="wallpaper-actions">
        <button onClick={handleSetWallpaper} className="btn btn-primary">
          设置壁纸
        </button>
      </div>
    </div>
  );
});
