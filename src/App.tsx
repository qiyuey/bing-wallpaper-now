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

function App() {
  const {
    localWallpapers,
    loading,
    error,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    forceUpdate,
    lastUpdateTime,
    isUpToDate,
  } = useBingWallpapers();

  const [showSettings, setShowSettings] = useState(false);
  const [showAbout, setShowAbout] = useState(false);

  // 监听托盘发出的 open-settings 事件，触发前端设置面板显示
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen("open-settings", () => {
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
        const unlistenFn = await listen("open-about", () => {
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
      alert("设置壁纸失败: " + String(err));
    }
  };

  // 壁纸壁纸列表与触发后端更新
  const handleRefresh = async () => {
    await fetchLocalWallpapers();
    try {
      await forceUpdate();
    } catch (err) {
      console.warn("Force update failed:", err);
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
      alert("打开文件夹失败: " + String(err));
    }
  }, []);

  // 监听托盘发出的 open-folder 事件（复用打开目录逻辑）
  useEffect(() => {
    let mounted = true;
    let unlisten: (() => void) | undefined;

    (async () => {
      try {
        const unlistenFn = await listen("open-folder", () => {
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
          style={{ display: "flex", flexDirection: "column", gap: "0.25rem" }}
        >
          <h1 style={{ margin: 0 }} className="app-title">
            <span className="app-title-main">Bing Wallpaper</span>
            <span className="app-title-accent">Now</span>
          </h1>
          <p className="app-tagline">世界之美 · 每日相遇</p>
        </div>
        <div
          className="header-actions"
          style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            flex: 1,
            marginLeft: "16px",
          }}
        >
          {isUpToDate && (
            <span className="status-badge status-success">已是最新</span>
          )}
          {lastUpdateTime && (
            <div className="last-update">上次更新: {lastUpdateTime}</div>
          )}
          <button onClick={handleRefresh} className="btn btn-icon" title="更新">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16" />
            </svg>
          </button>
          <button
            onClick={handleOpenFolder}
            className="btn btn-icon"
            title="打开下载目录"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
            </svg>
          </button>

          <button
            onClick={() => setShowSettings(true)}
            className="btn btn-icon"
            title="设置"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
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
        <Settings onClose={() => setShowSettings(false)} version={version} />
      )}

      {showAbout && (
        <About onClose={() => setShowAbout(false)} version={version} />
      )}
    </div>
  );
}

export default App;
