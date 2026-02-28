/**
 * 本地壁纸信息（后端格式：使用短字段名）
 */
export interface LocalWallpaperRaw {
  t: string; // title
  c: string; // copyright
  l: string; // copyright_link
  d: string; // end_date
  u?: string; // urlbase (可选)
}

/**
 * 本地壁纸信息（前端格式：使用完整字段名）
 */
export interface LocalWallpaper {
  title: string;
  copyright: string;
  copyright_link: string;
  end_date: string;
  urlbase?: string;
}

/**
 * 将后端返回的短字段名格式转换为前端使用的完整字段名格式
 */
export function normalizeWallpaper(raw: LocalWallpaperRaw): LocalWallpaper {
  return {
    title: raw.t,
    copyright: raw.c,
    copyright_link: raw.l,
    end_date: raw.d,
    urlbase: raw.u,
  };
}

/**
 * 批量转换壁纸数据格式
 */
export function normalizeWallpapers(
  raws: LocalWallpaperRaw[],
): LocalWallpaper[] {
  return raws.map(normalizeWallpaper);
}

/**
 * 获取壁纸的文件路径
 * @param wallpaperDirectory - 壁纸存储目录（完整路径）
 * @param endDate - 壁纸的结束日期（YYYYMMDD 格式）
 * @returns 壁纸文件的完整路径（使用正斜杠，适配 convertFileSrc）
 */
export function getWallpaperFilePath(
  wallpaperDirectory: string,
  endDate: string,
): string {
  // 如果目录为空，返回空字符串（让调用方处理）
  if (!wallpaperDirectory || wallpaperDirectory.trim() === "") {
    return "";
  }

  // 验证 endDate 格式（必须是 YYYYMMDD 格式的 8 位数字）
  if (!/^\d{8}$/.test(endDate)) {
    console.warn(`Invalid endDate format: ${endDate}, expected YYYYMMDD`);
    return "";
  }

  // 将反斜杠转换为正斜杠，确保跨平台兼容性
  // convertFileSrc 需要统一使用正斜杠，并且路径必须是绝对路径
  const normalizedDir = wallpaperDirectory.replace(/\\/g, "/");

  // 确保路径末尾没有多余的斜杠
  const cleanDir = normalizedDir.endsWith("/")
    ? normalizedDir.slice(0, -1)
    : normalizedDir;

  return `${cleanDir}/${endDate}.jpg`;
}

/**
 * 市场选项（单个市场）
 */
export interface MarketOption {
  /** 市场代码（如 "zh-CN"） */
  code: string;
  /** 显示名称（如 "中国大陆"） */
  label: string;
}

/**
 * 市场分组（按区域，由后端统一提供）
 */
export interface MarketGroup {
  /** 区域 ID（用于 i18n，如 "asia_pacific"） */
  region: string;
  /** 该区域下的市场列表 */
  markets: MarketOption[];
}

/**
 * Market 状态（由后端统一计算）
 */
export interface MarketStatus {
  /** 用户设置的 mkt */
  requested_mkt: string;
  /** 实际生效的 mkt（可能被 Bing 重定向） */
  effective_mkt: string;
  /** 是否存在 mismatch */
  is_mismatch: boolean;
}

/**
 * 应用设置
 */
export interface AppSettings {
  auto_update: boolean;
  save_directory: string | null;
  launch_at_startup: boolean;
  theme: string; // "light" | "dark" | "system" - 必需字段，与 Rust 端保持一致
  language: string; // "auto" | "zh-CN" | "en-US" - 用户的语言偏好（可以是 "auto"）
  resolved_language: string; // "zh-CN" | "en-US" - 后端解析后的实际语言，前端 i18n 应使用此字段
  mkt: string; // Bing API 市场代码（如 "zh-CN", "en-US", "ja-JP"），与 UI 语言独立
}
