// 多语言配置
// 支持的语言类型
export type Language = "auto" | "zh-CN" | "en-US";

/* eslint-env browser */

// 翻译内容
export const translations = {
  "zh-CN": {
    // App
    appTitle: "Bing Wallpaper",
    appSubtitle: "Now",
    appTagline: "世界之美 · 每日相遇",

    // 通用
    loading: "加载中...",
    error: "错误",
    success: "成功",
    cancel: "取消",
    save: "保存",
    close: "关闭",

    // 壁纸相关
    noWallpapers: "暂无壁纸",
    noWallpapersHint: "点击上方刷新按钮获取最新壁纸",
    wallpaperError: "设置壁纸失败",
    setWallpaper: "设置壁纸",
    retry: "重新加载",
    imageLoadError: "加载失败",
    imageLoadErrorHint: "图片可能还在下载中，请使用下方按钮重试",
    clickToViewDetails: "点击查看详情",

    // 文件夹相关
    folderError: "打开文件夹失败",
    openFolder: "打开目录",
    selectFolder: "选择文件夹",
    restoreDefault: "恢复默认目录",
    selectDirectory: "选择壁纸保存目录",

    // 设置相关
    settings: "设置",
    settingsTitle: "设置",
    launchAtStartup: "开机自启动",
    autoUpdate: "自动更新壁纸",
    theme: "主题",
    themeSystem: "跟随系统",
    themeLight: "浅色",
    themeDark: "深色",
    language: "语言",
    languageAuto: "自动",
    languageZhCN: "中文",
    languageEnUS: "English",
    saveDirectory: "保存目录",
    settingsLoading: "加载设置中...",
    settingsSaveError: "保存设置失败",
    settingsFolderSelectError: "选择文件夹失败",

    // 关于
    about: "关于",
    aboutTitle: "Bing Wallpaper Now",
    aboutVersion: "版本",
    aboutDescription:
      "每日自动获取并更新必应壁纸，支持高清壁纸下载和桌面壁纸设置。",
    aboutGitHub: "GitHub 仓库",
    aboutCopyright: "© 2025 Bing Wallpaper Now",

    // 状态
    lastUpdate: "上次更新",
    refresh: "更新",

    // 托盘菜单
    showWindow: "显示窗口",
    refreshWallpaper: "更新壁纸",
    openFolderMenu: "打开保存目录",
    settingsMenu: "打开设置",
    aboutMenu: "关于",
    quit: "退出",
  },
  "en-US": {
    // App
    appTitle: "Bing Wallpaper",
    appSubtitle: "Now",
    appTagline: "Beauty of the World · Daily Encounter",

    // 通用
    loading: "Loading...",
    error: "Error",
    success: "Success",
    cancel: "Cancel",
    save: "Save",
    close: "Close",

    // 壁纸相关
    noWallpapers: "No wallpapers",
    noWallpapersHint:
      "Click the refresh button above to get the latest wallpapers",
    wallpaperError: "Failed to set wallpaper",
    setWallpaper: "Set Wallpaper",
    retry: "Retry",
    imageLoadError: "Load Failed",
    imageLoadErrorHint:
      "Image may still be downloading, please use the button below to retry",
    clickToViewDetails: "Click to view details",

    // 文件夹相关
    folderError: "Failed to open folder",
    openFolder: "Open Folder",
    selectFolder: "Select Folder",
    restoreDefault: "Restore Default",
    selectDirectory: "Select Wallpaper Save Directory",

    // 设置相关
    settings: "Settings",
    settingsTitle: "Settings",
    launchAtStartup: "Launch at Startup",
    autoUpdate: "Auto Update Wallpaper",
    theme: "Theme",
    themeSystem: "System",
    themeLight: "Light",
    themeDark: "Dark",
    language: "Language",
    languageAuto: "Auto",
    languageZhCN: "中文",
    languageEnUS: "English",
    keepCount: "Keep Wallpaper Count",
    keepCountHint: "(0 means unlimited, at least keep 8)",
    saveDirectory: "Save Directory",
    settingsLoading: "Loading settings...",
    settingsSaveError: "Failed to save settings",
    settingsFolderSelectError: "Failed to select folder",

    // 关于
    about: "About",
    aboutTitle: "Bing Wallpaper Now",
    aboutVersion: "Version",
    aboutDescription:
      "Automatically fetch and update Bing wallpapers daily, with support for HD wallpaper downloads and desktop wallpaper settings.",
    aboutGitHub: "GitHub Repository",
    aboutCopyright: "© 2025 Bing Wallpaper Now",

    // 状态
    lastUpdate: "Last Update",
    refresh: "Refresh",

    // 托盘菜单
    showWindow: "Show Window",
    refreshWallpaper: "Refresh Wallpaper",
    openFolderMenu: "Open Save Directory",
    settingsMenu: "Open Settings",
    aboutMenu: "About",
    quit: "Quit",
  },
} as const;

// 翻译键值类型（在 translations 定义之后）
export type TranslationKey = keyof (typeof translations)["zh-CN"];

/**
 * 检测系统语言
 */
export function detectSystemLanguage(): "zh-CN" | "en-US" {
  // 优先使用标准的 navigator.language
  // 对于旧版 IE，使用类型守卫安全访问 userLanguage
  const systemLang =
    window.navigator.language ||
    ("userLanguage" in window.navigator
      ? (window.navigator as { userLanguage?: string }).userLanguage
      : undefined);

  if (systemLang && systemLang.startsWith("zh")) {
    return "zh-CN";
  }
  return "en-US";
}

/**
 * 获取实际使用的语言
 */
export function getActualLanguage(language: Language): "zh-CN" | "en-US" {
  if (language === "auto") {
    return detectSystemLanguage();
  }
  return language;
}
