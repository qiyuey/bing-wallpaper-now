import { useState, useEffect, useCallback } from "react";
import "./App.css";
import { useBingWallpapers } from "./hooks/useBingWallpapers";
import { WallpaperGrid } from "./components/WallpaperGrid";
import { Settings } from "./components/Settings";
import { About } from "./components/About";

import { LocalWallpaper } from "./types";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";
import { listen } from "@tauri-apps/api/event";
import { version } from "../package.json";
import { createSafeUnlisten } from "./utils/eventListener";
import { getStandardIconProps } from "./config/icons";
import { INLINE_SPACING, EVENTS } from "./config/ui";
import { useDynamicTagline } from "./hooks/useDynamicTagline";
import { useI18n } from "./i18n/I18nContext";

function App() {
  const {
    localWallpapers,
    loading,
    error,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    forceUpdate,
    lastUpdateTime,
  } = useBingWallpapers();

  const { t, actualLanguage } = useI18n();
  const dynamicTagline = useDynamicTagline("time-based", 60000, actualLanguage); // 使用基于时间段的动态标语，根据当前语言显示

  const [showSettings, setShowSettings] = useState(false);
  const [showAbout, setShowAbout] = useState(false);

  // 监听托盘发出的 open-settings 事件，触发前端设置面板显示
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen(EVENTS.OPEN_SETTINGS, () => {
          setShowSettings(true);
        });
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safeUnlisten;
        } else {
          safeUnlisten();
        }
      } catch (e) {
        console.error("Failed to bind open-settings event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, []);

  // 监听托盘发出的 open-about 事件，触发前端关于对话框显示
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen(EVENTS.OPEN_ABOUT, () => {
          setShowAbout(true);
        });
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safeUnlisten;
        } else {
          safeUnlisten();
        }
      } catch (e) {
        console.error("Failed to bind open-about event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, []);

  // 处理设置壁纸
  const handleSetWallpaper = async (wallpaper: LocalWallpaper) => {
    try {
      // 异步设置，不阻塞 UI
      await setDesktopWallpaper(wallpaper.file_path);
    } catch (err) {
      console.error("Failed to set wallpaper:", err);
      alert(`${t("wallpaperError")}: ${String(err)}`);
    }
  };

  // 刷新壁纸列表与触发后端更新
  const handleRefresh = async () => {
    await fetchLocalWallpapers();
    try {
      await forceUpdate(true);
    } catch (err) {
      console.warn("Force update failed:", err);
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
      console.warn("Language change refresh failed:", err);
    }
  };

  // 打开下载目录
  const handleOpenFolder = useCallback(async () => {
    try {
      const folderPath = await invoke<string>("get_wallpaper_directory");
      await invoke("ensure_wallpaper_directory_exists");
      await openPath(folderPath);
    } catch (err) {
      console.error("Failed to open folder:", err);
      alert(`${t("folderError")}: ${String(err)}`);
    }
  }, [t]);

  // 监听托盘发出的 open-folder 事件（复用打开目录逻辑）
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen(EVENTS.OPEN_FOLDER, () => {
          handleOpenFolder();
        });
        const safeUnlisten = createSafeUnlisten(unlistenFn);

        if (mounted) {
          unlisten = safeUnlisten;
        } else {
          safeUnlisten();
        }
      } catch (e) {
        console.error("Failed to bind open-folder event:", e);
      }
    })();

    return () => {
      mounted = false;
      unlisten?.();
    };
  }, [handleOpenFolder]);

  return (
    <div className="app">
      <header
        className="app-header"
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <div
          style={{
            display: "flex",
            flexDirection: "column",
            gap: INLINE_SPACING.TITLE_GAP,
            minWidth: 0,
            flexShrink: 1,
          }}
        >
          <h1 style={{ margin: 0 }} className="app-title">
            <span className="app-title-main">{t("appTitle")}</span>
            <span className="app-title-accent">{t("appSubtitle")}</span>
          </h1>
          <div
            style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}
          >
            <p className="app-tagline">{dynamicTagline}</p>
            {lastUpdateTime && (
              <div className="last-update">
                {t("lastUpdate")}: {lastUpdateTime}
              </div>
            )}
          </div>
        </div>
        <div
          className="header-actions"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            flex: "0 0 auto",
            marginLeft: INLINE_SPACING.HEADER_MARGIN_LEFT,
            gap: "0.625rem",
          }}
        >
          <button
            onClick={handleRefresh}
            className="btn btn-icon"
            title={t("refresh")}
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
          </button>
          <button
            onClick={handleOpenFolder}
            className="btn btn-icon"
            title={t("openFolder")}
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
          </button>

          <button
            onClick={() => setShowSettings(true)}
            className="btn btn-icon"
            title={t("settings")}
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
          </button>
        </div>
      </header>

      {error && <div className="error-message">{error}</div>}

      <main className="app-main">
        <WallpaperGrid
          wallpapers={localWallpapers}
          onSetWallpaper={handleSetWallpaper}
          loading={loading}
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
    </div>
  );
}

export default App;
