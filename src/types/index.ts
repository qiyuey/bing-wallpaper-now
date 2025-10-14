/**
 * 必应图片条目
 */
export interface BingImageEntry {
  url: string;
  urlbase: string;
  copyright: string;
  copyrightlink: string;
  title: string;
  startdate: string;
  enddate: string;
  hsh: string;
}

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
  update_interval_hours: number;
  save_directory: string | null;
  keep_image_count: number;
  launch_at_startup: boolean;
}
