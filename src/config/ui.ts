// UI 常量配置
// 统一管理界面中的常用常量值

/**
 * 容器尺寸配置
 */
export const CONTAINER = {
  /** 默认容器高度 */
  DEFAULT_HEIGHT: 600 as number,
} as const;

/**
 * 间距配置（用于内联样式）
 */
export const INLINE_SPACING = {
  /** 标题间距 */
  TITLE_GAP: "0.25rem",
  /** 头部操作区域左边距 */
  HEADER_MARGIN_LEFT: "16px",
} as const;

/**
 * 文本内容配置
 */
export const TEXT = {
  /** 应用标题 */
  APP_TITLE: "Bing Wallpaper",
  /** 应用副标题 */
  APP_SUBTITLE: "Now",
  /** 应用标语 */
  APP_TAGLINE: "哪怕前路渺茫，也要让心中有光。",
  /** 无壁纸提示 */
  NO_WALLPAPERS: "暂无壁纸",
  /** 无壁纸提示副文本 */
  NO_WALLPAPERS_HINT: "点击上方刷新按钮获取最新壁纸",
  /** 设置失败提示 */
  SETTINGS_ERROR: "保存设置失败",
  /** 壁纸设置失败提示 */
  WALLPAPER_ERROR: "设置壁纸失败",
  /** 文件夹打开失败提示 */
  FOLDER_ERROR: "打开文件夹失败",
} as const;

/**
 * 事件名称配置
 */
export const EVENTS = {
  /** 打开设置 */
  OPEN_SETTINGS: "open-settings",
  /** 打开关于 */
  OPEN_ABOUT: "open-about",
  /** 打开文件夹 */
  OPEN_FOLDER: "open-folder",
  /** 检查更新结果 */
  CHECK_UPDATES_RESULT: "check-updates-result",
  /** 检查更新无更新 */
  CHECK_UPDATES_NO_UPDATE: "check-updates-no-update",
  /** mkt 状态变化（mismatch 边沿触发：false→true / true→false） */
  MKT_STATUS_CHANGED: "mkt-status-changed",
} as const;
