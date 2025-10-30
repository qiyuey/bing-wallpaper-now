import { memo, useCallback, useMemo, useState, useEffect } from "react";
import { LocalWallpaper } from "../types";
import { openUrl } from "@tauri-apps/plugin-opener";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useI18n } from "../i18n/I18nContext";

interface WallpaperCardProps {
  wallpaper: LocalWallpaper;
  onSetWallpaper: (wallpaper: LocalWallpaper) => void;
}

// 图片加载成功缓存（组件外部，避免重复加载）
const loadedImagesCache = new Set<string>();

// 使用 memo 优化，只在 props 变化时重新渲染
export const WallpaperCard = memo(function WallpaperCard({
  wallpaper,
  onSetWallpaper,
}: WallpaperCardProps) {
  const { t } = useI18n();
  // 检查图片是否已加载过
  const isImageCached = loadedImagesCache.has(wallpaper.file_path);

  const [imageLoading, setImageLoading] = useState(!isImageCached);
  const [imageError, setImageError] = useState(false);
  const [retryCount, setRetryCount] = useState(0);
  const [waitingForDownload, setWaitingForDownload] = useState(!isImageCached); // 是否正在等待后端下载

  // 当图片路径变化时重置状态（但如果图片已缓存则不重置）
  useEffect(() => {
    const isCached = loadedImagesCache.has(wallpaper.file_path);
    setImageLoading(!isCached);
    setImageError(false);
    setRetryCount(0);
    setWaitingForDownload(!isCached);
  }, [wallpaper.file_path]);

  // 监听后端下载完成事件，自动重新加载对应的图片
  useEffect(() => {
    const unlisten = listen<string>("image-downloaded", (event) => {
      // 从文件路径中提取日期（例如：/path/to/20251026.jpg -> 20251026）
      const dateFromPath = wallpaper.file_path.match(/(\d{8})\.jpg$/)?.[1];

      // 如果下载完成的图片就是当前这张
      if (event.payload === dateFromPath) {
        setWaitingForDownload(false); // 标记已收到下载通知
        setImageLoading(true);
        setImageError(false);
        setRetryCount((prev) => prev + 1);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
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
    setWaitingForDownload(false); // 图片加载成功，不再等待
    // 将成功加载的图片路径加入缓存
    loadedImagesCache.add(wallpaper.file_path);
  }, [wallpaper.file_path]);

  const handleImageError = useCallback(() => {
    // 图片加载失败，可能是文件还未下载完成（UHD图片较大，下载时间较长）
    // 如果还在等待后端下载，则保持加载状态，不显示错误
    // 只有在收到下载通知后加载失败，才显示错误
    if (!waitingForDownload) {
      setImageLoading(false);
      setImageError(true);
    }
    // 否则保持 imageLoading=true，继续显示加载中状态
  }, [waitingForDownload]);

  // 手动重试加载（仅在真正加载失败时使用，比如文件已下载但前端加载出错）
  const handleManualRetry = useCallback(() => {
    setImageLoading(true);
    setImageError(false);
    setRetryCount((prev) => prev + 1);
    // 从缓存中移除失败的图片，允许重新加载
    loadedImagesCache.delete(wallpaper.file_path);
  }, [wallpaper.file_path]);

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
        title={t("clickToViewDetails")}
      >
        {imageError ? (
          // 图片加载失败 - 仅显示错误状态，重试按钮在底部
          <div className="wallpaper-image-placeholder">
            <p className="placeholder-error-text">{t("imageLoadError")}</p>
            <p className="placeholder-hint-text">
              {t("imageLoadErrorHint")}
            </p>
          </div>
        ) : (
          <>
            {imageLoading && (
              <div className="wallpaper-image-placeholder">
                <div className="spinner"></div>
                <p className="placeholder-loading-text">{t("loading")}</p>
              </div>
            )}
            <img
              key={`${wallpaper.file_path}-${retryCount}`}
              src={imageUrl}
              alt={title}
              className="wallpaper-image"
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
          onClick={imageError ? handleManualRetry : handleSetWallpaper}
          className="btn btn-primary"
          disabled={imageLoading}
          title={
            imageLoading
              ? t("loading")
              : imageError
                ? t("retry")
                : t("setWallpaper")
          }
        >
          {imageLoading ? t("loading") : imageError ? t("retry") : t("setWallpaper")}
        </button>
      </div>
    </div>
  );
});
