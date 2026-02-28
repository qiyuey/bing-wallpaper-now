// 多语言配置
// 支持的语言类型："auto" 表示跟随系统
export type Language = "auto" | "zh-CN" | "en-US";

// 翻译内容
export const translations = {
  "zh-CN": {
    // App
    appTitle: "Bing Wallpaper",
    appSubtitle: "Now",
    // 通用
    loading: "加载中...",
    error: "错误",
    success: "成功",
    cancel: "取消",
    save: "保存",
    close: "关闭",

    // 壁纸相关
    noWallpapers: "暂无壁纸",
    noWallpapersHint:
      "点击上方刷新按钮获取最新壁纸。如果您不是所在地区的 IP，可能无法获取对应语言的内容，请尝试切换语言。",
    wallpaperError: "设置壁纸失败",
    wallpaperSetSuccess: "壁纸设置成功",
    setWallpaper: "设置壁纸",
    retry: "重新加载",
    imageLoadError: "加载失败",
    imageLoadErrorHint: "图片可能还在下载中，请使用下方按钮重试",
    clickToViewDetails: "点击查看详情",

    // 文件夹相关
    folderError: "打开文件夹失败",
    wallpaperDirectoryError: "无法获取壁纸目录",
    openFolder: "打开目录",
    selectFolder: "选择文件夹",
    restoreDefault: "恢复默认目录",
    selectDirectory: "选择壁纸保存目录",

    // 设置相关
    settings: "设置",
    settingsTitle: "设置",
    launchAtStartup: "开机自启动",
    autoUpdate: "自动应用新壁纸",
    autoUpdateHint:
      "开启时：自动获取新壁纸，并在检测到更新的壁纸时自动应用该壁纸\n关闭时：只有手动点击设置壁纸才会设置，但是仍然会自动获取新壁纸",
    theme: "主题",
    themeSystem: "跟随系统",
    themeLight: "浅色",
    themeDark: "深色",
    language: "语言",
    languageAuto: "自动",
    languageZhCN: "中文",
    languageEnUS: "English",
    market: "壁纸市场",
    marketHint: "决定获取哪个地区的壁纸，与界面语言独立",
    marketRegionAsiaPacific: "亚太",
    marketRegionEurope: "欧洲",
    marketRegionAmericas: "美洲",
    marketRegionAfrica: "非洲",
    marketMismatchWarning:
      "注意：Bing 实际返回了 {actualMkt} 的壁纸，与您选择的 {requestedMkt} 不同。这通常是因为您所在地区的 Bing 不支持该市场。",
    saveDirectory: "保存目录",
    settingsLoading: "加载设置中...",
    settingsSaveError: "保存设置失败",
    settingsFolderSelectError: "选择文件夹失败",

    // 导入
    importData: "数据导入",
    importDataHint: "从其他 Bing Wallpaper Now 的壁纸目录导入历史数据",
    importSelectDirectory: "选择目录并导入",
    importInProgress: "导入中...",
    importSuccess:
      "新增 {new} 条壁纸信息，更新 {updated} 条，复制 {images} 张图片",
    importMetadataSkipped: "{count} 条元数据已跳过",
    importImagesFailed: "{count} 张图片复制失败",
    warningSeparator: "，",
    transferNotDirectory: "所选路径不是有效目录",
    importNoData: "所选目录中没有可导入的数据",
    importAlreadyUpToDate: "所有数据已是最新，无需导入",
    importError: "导入失败",
    importSameDirectory: "不能从当前壁纸目录导入",

    // 导出
    exportData: "数据导出",
    exportDataHint: "将壁纸信息和图片导出到指定目录，可用于备份或迁移",
    exportSelectDirectory: "选择目录并导出",
    exportInProgress: "导出中...",
    exportSuccess:
      "新增 {new} 条壁纸信息，更新 {updated} 条，复制 {images} 张图片",
    exportMetadataSkipped: "{count} 条元数据已跳过",
    exportImagesFailed: "{count} 张图片复制失败",
    exportAlreadyUpToDate: "目标目录数据已是最新，无需导出",
    exportError: "导出失败",
    exportSameDirectory: "不能导出到当前壁纸目录",
    exportNoData: "当前没有可导出的数据",

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
    currentMarket: "市场",
    refresh: "更新",

    // 版本检查
    checkForUpdates: "检查更新",
    checkingForUpdates: "检查中...",
    updateAvailable: "有新版本可用",
    updateAvailableMessage: "发现新版本 {version}，是否前往下载？",
    updateAvailableHint: "发现新版本 {version}，点击前往下载",
    downloadUpdate: "前往下载",
    goToUpdate: "前往更新",
    viewRelease: "详情",
    downloadAndInstall: "安装",
    downloading: "下载中...",
    downloadProgress: "下载中 {percent}%",
    downloadFailed: "下载失败",
    downloadComplete: "下载完成，正在重启...",
    cancelDownload: "取消下载",
    ignoreThisVersion: "忽略",
    noUpdateAvailable: "已是最新版本",
    updateCheckError: "检查更新失败",
    updateCheckFailed: "无法检查更新，请稍后重试",

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
      "Click the refresh button above to get the latest wallpapers. If you are not using an IP from the target region, you may not be able to fetch content in the corresponding language. Please try switching languages.",
    wallpaperError: "Failed to set wallpaper",
    wallpaperSetSuccess: "Wallpaper set successfully",
    setWallpaper: "Set Wallpaper",
    retry: "Retry",
    imageLoadError: "Load Failed",
    imageLoadErrorHint:
      "Image may still be downloading, please use the button below to retry",
    clickToViewDetails: "Click to view details",

    // 文件夹相关
    folderError: "Failed to open folder",
    wallpaperDirectoryError: "Failed to get wallpaper directory",
    openFolder: "Open Folder",
    selectFolder: "Select Folder",
    restoreDefault: "Restore Default",
    selectDirectory: "Select Wallpaper Save Directory",

    // 设置相关
    settings: "Settings",
    settingsTitle: "Settings",
    launchAtStartup: "Launch at Startup",
    autoUpdate: "Auto Apply New Wallpaper",
    autoUpdateHint:
      "When enabled: Automatically fetch new wallpapers and apply them when detected\nWhen disabled: Only set wallpaper when manually clicked, but still automatically fetch new wallpapers",
    theme: "Theme",
    themeSystem: "System",
    themeLight: "Light",
    themeDark: "Dark",
    language: "Language",
    languageAuto: "Auto",
    languageZhCN: "中文",
    languageEnUS: "English",
    market: "Wallpaper Market",
    marketHint:
      "Determines which region's wallpapers to fetch, independent of UI language",
    marketRegionAsiaPacific: "Asia Pacific",
    marketRegionEurope: "Europe",
    marketRegionAmericas: "Americas",
    marketRegionAfrica: "Africa",
    marketMismatchWarning:
      "Note: Bing returned wallpapers for {actualMkt} instead of your selected {requestedMkt}. This usually happens when Bing in your region does not support the selected market.",
    saveDirectory: "Save Directory",
    settingsLoading: "Loading settings...",
    settingsSaveError: "Failed to save settings",
    settingsFolderSelectError: "Failed to select folder",

    // 导入
    importData: "Data Import",
    importDataHint:
      "Import historical data from another Bing Wallpaper Now wallpaper directory",
    importSelectDirectory: "Select Directory & Import",
    importInProgress: "Importing...",
    importSuccess:
      "Added {new} entries, updated {updated}, copied {images} images",
    importMetadataSkipped: "{count} metadata entries were skipped",
    importImagesFailed: "{count} images failed to copy",
    warningSeparator: ", ",
    transferNotDirectory: "The selected path is not a valid directory",
    importNoData: "No importable data found in the selected directory",
    importAlreadyUpToDate: "All data is already up to date, nothing to import",
    importError: "Import failed",
    importSameDirectory: "Cannot import from the current wallpaper directory",

    // 导出
    exportData: "Data Export",
    exportDataHint:
      "Export current wallpaper data to a specified directory for backup or migration",
    exportSelectDirectory: "Select Directory & Export",
    exportInProgress: "Exporting...",
    exportSuccess:
      "Added {new} entries, updated {updated}, copied {images} images",
    exportMetadataSkipped: "{count} metadata entries were skipped",
    exportImagesFailed: "{count} images failed to copy",
    exportAlreadyUpToDate:
      "Target directory is already up to date, nothing to export",
    exportError: "Export failed",
    exportSameDirectory: "Cannot export to the current wallpaper directory",
    exportNoData: "No data available to export",

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
    currentMarket: "Market",
    refresh: "Refresh",

    // 版本检查
    checkForUpdates: "Check for Updates",
    checkingForUpdates: "Checking...",
    updateAvailable: "Update Available",
    updateAvailableMessage:
      "New version {version} is available, would you like to download it?",
    updateAvailableHint:
      "New version {version} is available, click to download",
    downloadUpdate: "Download",
    goToUpdate: "Go to Update",
    viewRelease: "View",
    downloadAndInstall: "Install",
    downloading: "Downloading...",
    downloadProgress: "Downloading {percent}%",
    downloadFailed: "Download Failed",
    downloadComplete: "Download complete, restarting...",
    cancelDownload: "Cancel Download",
    ignoreThisVersion: "Ignore",
    noUpdateAvailable: "Already up to date",
    updateCheckError: "Update Check Failed",
    updateCheckFailed: "Unable to check for updates, please try again later",

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
