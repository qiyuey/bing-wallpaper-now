import { memo, useCallback, useMemo, useState, useEffect } from "react";
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
  const [imageLoading, setImageLoading] = useState(true);
  const [imageError, setImageError] = useState(false);
  const [retryCount, setRetryCount] = useState(0);

  // 当图片路径变化时重置状态
  useEffect(() => {
    setImageLoading(true);
    setImageError(false);
    setRetryCount(0);
  }, [wallpaper.file_path]);

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

  const handleImageLoad = useCallback(() => {
    setImageLoading(false);
    setImageError(false);
  }, []);

  const handleImageError = useCallback(() => {
    // 图片加载失败，可能是文件还未下载完成
    // 自动重试几次（每3秒一次，最多3次）
    if (retryCount < 3) {
      window.setTimeout(() => {
        setRetryCount((prev) => prev + 1);
      }, 3000);
    } else {
      setImageLoading(false);
      setImageError(true);
    }
  }, [retryCount]);

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
        {imageError ? (
          // 图片加载失败 - 显示错误状态
          <div className="wallpaper-image-placeholder">
            <p style={{ fontSize: "14px", color: "#fff" }}>加载失败</p>
            <p style={{ fontSize: "12px", color: "rgba(255,255,255,0.7)", marginTop: "4px" }}>
              图片可能还在下载中
            </p>
          </div>
        ) : (
          <>
            {imageLoading && (
              <div className="wallpaper-image-placeholder">
                <div className="spinner"></div>
                <p style={{ marginTop: "12px", fontSize: "12px", color: "rgba(255,255,255,0.8)" }}>
                  {retryCount > 0 ? `加载中 (${retryCount}/3)...` : "加载中..."}
                </p>
              </div>
            )}
            <img
              key={`${wallpaper.file_path}-${retryCount}`}
              src={imageUrl}
              alt={title}
              className="wallpaper-image"
              loading="lazy"
              decoding="async"
              onLoad={handleImageLoad}
              onError={handleImageError}
              style={{ display: imageLoading ? "none" : "block" }}
            />
          </>
        )}
      </div>
      <div className="wallpaper-info">
        <h3 className="wallpaper-title">{title}</h3>
        {subtitle && <p className="wallpaper-subtitle">{subtitle}</p>}
      </div>
      <div className="wallpaper-actions">
        <button
          onClick={handleSetWallpaper}
          className="btn btn-primary"
          disabled={imageLoading || imageError}
          title={
            imageLoading
              ? "图片加载中，请稍候..."
              : imageError
                ? "图片加载失败"
                : "设置为桌面壁纸"
          }
        >
          {imageLoading ? "加载中..." : imageError ? "加载失败" : "设置壁纸"}
        </button>
      </div>
    </div>
  );
});
