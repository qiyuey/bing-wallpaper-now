import {
  useState,
  useEffect,
  useCallback,
  useRef,
  useMemo,
  type CSSProperties,
} from "react";
import { useBingWallpapers } from "./hooks/useBingWallpapers";
import { WallpaperGrid } from "./components/WallpaperGrid";
import { Settings } from "./components/Settings";
import { About } from "./components/About";
import { UpdateDialog } from "./components/UpdateDialog";
import { showSystemNotification } from "./utils/notification";
import { LocalWallpaper, getWallpaperFilePath } from "./types";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { version } from "../package.json";
import { getStandardIconProps } from "./config/icons";
import { useI18n } from "./i18n/I18nContext";
import { useUpdateCheck } from "./hooks/useUpdateCheck";
import { useTrayEvents } from "./hooks/useTrayEvents";
import { cn } from "./utils/cn";
import styles from "./App.module.css";
import btnStyles from "./styles/buttons.module.css";
import glassStyles from "./styles/liquid-glass.module.css";

function App() {
  const {
    localWallpapers,
    loading,
    error,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    forceUpdate,
    lastUpdateTime,
    effectiveMktLabel,
  } = useBingWallpapers();

  const { t } = useI18n();
  const [showSettings, setShowSettings] = useState(false);
  const [showAbout, setShowAbout] = useState(false);
  const [wallpaperDirectory, setWallpaperDirectory] = useState<string>("");
  const { updateInfo, setUpdateInfo } = useUpdateCheck();

  const ambientBgStyle = useMemo(() => {
    const first = localWallpapers[0];
    if (!first || !wallpaperDirectory) return undefined;
    const filePath = getWallpaperFilePath(wallpaperDirectory, first.end_date);
    if (!filePath) return undefined;
    return {
      "--ambient-bg": `url("${convertFileSrc(filePath)}")`,
    } as CSSProperties;
  }, [localWallpapers, wallpaperDirectory]);

  // 获取壁纸目录
  useEffect(() => {
    invoke<string>("get_wallpaper_directory")
      .then(setWallpaperDirectory)
      .catch((err) => console.error("Failed to get wallpaper directory:", err));
  }, []);

  // 打开下载目录
  const handleOpenFolder = useCallback(async () => {
    try {
      const folderPath = await invoke<string>("get_wallpaper_directory");
      await invoke("ensure_wallpaper_directory_exists");
      await openPath(folderPath);
    } catch (err) {
      console.error("Failed to open folder:", err);
      await showSystemNotification(
        t("folderError"),
        `${t("folderError")}: ${String(err)}`,
      );
    }
  }, [t]);

  useTrayEvents({
    onOpenSettings: () => setShowSettings(true),
    onOpenAbout: () => setShowAbout(true),
    onOpenFolder: handleOpenFolder,
  });

  // 键盘快捷键支持（使用 ref 避免频繁重新绑定事件监听器）
  const showSettingsRef = useRef(showSettings);
  const showAboutRef = useRef(showAbout);
  const updateInfoRef = useRef(updateInfo);

  useEffect(() => {
    showSettingsRef.current = showSettings;
    showAboutRef.current = showAbout;
    updateInfoRef.current = updateInfo;
  }, [showSettings, showAbout, updateInfo]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Esc 键关闭模态框
      if (e.key === "Escape") {
        if (showSettingsRef.current) {
          setShowSettings(false);
          e.preventDefault();
        } else if (showAboutRef.current) {
          setShowAbout(false);
          e.preventDefault();
        } else if (updateInfoRef.current) {
          setUpdateInfo(null);
          e.preventDefault();
        }
      }

      // Cmd/Ctrl + , 打开设置
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        setShowSettings(true);
        e.preventDefault();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [setUpdateInfo]);

  // 处理设置壁纸
  const handleSetWallpaper = async (wallpaper: LocalWallpaper) => {
    try {
      // 动态生成 file_path
      let filePath = getWallpaperFilePath(
        wallpaperDirectory,
        wallpaper.end_date,
      );

      // 如果路径为空，说明目录还未加载完成，重新获取目录
      if (!filePath || filePath.trim() === "") {
        const dir = await invoke<string>("get_wallpaper_directory");
        setWallpaperDirectory(dir);
        filePath = getWallpaperFilePath(dir, wallpaper.end_date);
        if (!filePath || filePath.trim() === "") {
          throw new Error(t("wallpaperDirectoryError"));
        }
      }

      // 异步设置，不阻塞 UI
      await setDesktopWallpaper(filePath);
      await showSystemNotification(t("setWallpaper"), t("wallpaperSetSuccess"));
    } catch (err) {
      console.error("Failed to set wallpaper:", err);
      await showSystemNotification(
        t("wallpaperError"),
        `${t("wallpaperError")}: ${String(err)}`,
      );
    }
  };

  // 刷新壁纸列表与触发后端更新
  const handleRefresh = async () => {
    await fetchLocalWallpapers();
    try {
      await forceUpdate(true);
    } catch (err) {
      console.error("Force update failed:", err);
      await showSystemNotification(t("wallpaperError"), String(err));
    }
  };

  // 语言切换时的刷新（强制更新）
  const handleLanguageChangeRefresh = async () => {
    try {
      // 先立即刷新一次列表，显示当前语言的现有数据
      await fetchLocalWallpapers(true);
      // 然后触发强制更新，下载新语言的壁纸数据
      await forceUpdate(true);
      // 更新完成后会通过 wallpaper-updated 事件自动刷新列表
    } catch (err) {
      console.error("Language change refresh failed:", err);
      await showSystemNotification(t("wallpaperError"), String(err));
    }
  };

  return (
    <div className={styles.app} style={ambientBgStyle}>
      <header
        className={cn(
          styles.appHeader,
          glassStyles.liquidGlass,
          glassStyles.nav,
        )}
      >
        <div className={styles.titleGroup}>
          <h1 className={styles.appTitle}>
            <span className={styles.appTitleMain}>{t("appTitle")}</span>
            <span className={styles.appTitleAccent}>{t("appSubtitle")}</span>
          </h1>
          {lastUpdateTime && (
            <div className={styles.lastUpdate}>
              {t("lastUpdate")}: {lastUpdateTime}
              {effectiveMktLabel && (
                <>
                  {" · "}
                  {t("currentMarket")}: {effectiveMktLabel}
                </>
              )}
            </div>
          )}
        </div>
        <div className={styles.headerActions}>
          <button
            onClick={handleRefresh}
            className={cn(btnStyles.btn, btnStyles.btnIcon)}
            aria-label={t("refresh")}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              {...getStandardIconProps()}
              fill="none"
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16" />
            </svg>
            <span className={btnStyles.btnTooltip}>{t("refresh")}</span>
          </button>
          <button
            onClick={handleOpenFolder}
            className={cn(btnStyles.btn, btnStyles.btnIcon)}
            aria-label={t("openFolder")}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              {...getStandardIconProps()}
              fill="none"
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
            <span className={btnStyles.btnTooltip}>{t("openFolder")}</span>
          </button>

          <button
            onClick={() => setShowSettings(true)}
            className={cn(btnStyles.btn, btnStyles.btnIcon)}
            aria-label={t("settings")}
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              {...getStandardIconProps()}
              fill="none"
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
            </svg>
            <span className={btnStyles.btnTooltip}>{t("settings")}</span>
          </button>
        </div>
      </header>

      {error && <div className={styles.errorMessage}>{error}</div>}

      <main className={styles.appMain}>
        <WallpaperGrid
          wallpapers={localWallpapers}
          onSetWallpaper={handleSetWallpaper}
          loading={loading}
          wallpaperDirectory={wallpaperDirectory}
        />
      </main>

      {showSettings && (
        <Settings
          onClose={() => setShowSettings(false)}
          version={version}
          onLanguageChange={handleLanguageChangeRefresh}
        />
      )}

      {showAbout && (
        <About onClose={() => setShowAbout(false)} version={version} />
      )}

      {updateInfo && (
        <UpdateDialog
          version={updateInfo.version}
          body={updateInfo.body}
          update={updateInfo.update}
          onClose={() => setUpdateInfo(null)}
          onIgnore={() => setUpdateInfo(null)}
        />
      )}
    </div>
  );
}

export default App;
