/**
 * 本地壁纸信息
 */
export interface LocalWallpaper {
  id: string;
  title: string;
  copyright: string;
  copyright_link: string;
  start_date: string;
  end_date: string;
  file_path: string;
  download_time: string;
}

/**
 * 应用设置
 */
export interface AppSettings {
  auto_update: boolean;
  save_directory: string | null;
  keep_image_count: number;
  launch_at_startup: boolean;
  theme: string; // "light" | "dark" | "system" - 必需字段，与 Rust 端保持一致
  language: string; // "auto" | "zh-CN" | "en-US" - 必需字段，Rust 端有默认值 "auto"
}
