import { BingImageEntry } from "../types";
import { openUrl } from "@tauri-apps/plugin-opener";

interface WallpaperCardProps {
  image: BingImageEntry;
  onSetWallpaper: () => void;
}

export function WallpaperCard({
  image,
  onSetWallpaper,
}: WallpaperCardProps) {
  // 处理图片点击，打开版权链接
  const handleImageClick = async () => {
    if (image.copyrightlink) {
      try {
        await openUrl(image.copyrightlink);
      } catch (err) {
        console.error("Failed to open link:", err);
      }
    }
  };
  // 解析标题和副标题
  // 主标题：image.title（如"蓝与白的梦境"）
  // 副标题：copyright 中不包含括号的部分（如"伊亚镇，圣托里尼岛，希腊"）
  const parseTitleAndSubtitle = () => {
    const title = image.title;
    const copyright = image.copyright;

    // 从 copyright 中提取括号外的内容作为副标题
    const match = copyright.match(/^([^(]+?)(?:\s*\(([^)]+)\))?$/);
    const subtitle = match ? match[1].trim() : copyright;

    return {
      title,
      subtitle
    };
  };

  const { title, subtitle } = parseTitleAndSubtitle();

  return (
    <div className="wallpaper-card">
      <div
        className="wallpaper-image-container"
        onClick={handleImageClick}
        style={{ cursor: 'pointer' }}
        title="点击查看详情"
      >
        <img
          src={image.url}
          alt={title}
          className="wallpaper-image"
          loading="lazy"
        />
      </div>
      <div className="wallpaper-info">
        <h3 className="wallpaper-title">{title}</h3>
        {subtitle && <p className="wallpaper-subtitle">{subtitle}</p>}
      </div>
      <div className="wallpaper-actions">
        <button onClick={onSetWallpaper} className="btn btn-primary">
          设置壁纸
        </button>
      </div>
    </div>
  );
}
