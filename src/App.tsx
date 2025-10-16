import { useState } from "react";
import "./App.css";
import { useBingWallpapers } from "./hooks/useBingWallpapers";
import { WallpaperGrid } from "./components/WallpaperGrid";
import { Settings } from "./components/Settings";

import { LocalWallpaper } from "./types";
import { invoke } from "@tauri-apps/api/core";
import { openPath } from "@tauri-apps/plugin-opener";

function App() {
  const {
    localWallpapers,
    loading,
    error,
    fetchLocalWallpapers,
    setDesktopWallpaper,
    forceUpdate,
    updating,
    lastUpdateTime,
  } = useBingWallpapers();

  const [showSettings, setShowSettings] = useState(false);

  // 处理设置壁纸
  const handleSetWallpaper = async (wallpaper: LocalWallpaper) => {
    try {
      console.log("Setting wallpaper...", wallpaper);
      await setDesktopWallpaper(wallpaper.file_path);
      alert("壁纸已设置成功!");
    } catch (err) {
      console.error("Failed to set wallpaper:", err);
      alert("设置壁纸失败: " + String(err));
    }
  };

  // 刷新壁纸列表与触发后端更新
  const handleRefresh = async () => {
    await fetchLocalWallpapers();
    try {
      await forceUpdate();
    } catch (err) {
      console.log("Force update failed:", err);
    }
  };

  // 打开下载目录
  const handleOpenFolder = async () => {
    try {
      const folderPath = await invoke<string>("get_wallpaper_directory");
      console.log("Opening folder:", folderPath);
      await invoke("ensure_wallpaper_directory_exists");
      await openPath(folderPath);
    } catch (err) {
      console.error("Failed to open folder:", err);
      alert("打开文件夹失败: " + String(err));
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>必应壁纸</h1>
        <div
          className="header-actions"
          style={{ display: "flex", alignItems: "center" }}
        >
          {updating && (
            <div
              className="updating-indicator"
              style={{
                fontSize: "12px",
                color: "#888",
                marginRight: "8px",
                display: "flex",
                alignItems: "center",
              }}
            >
              <svg
                width="14"
                height="14"
                viewBox="0 0 50 50"
                style={{
                  marginRight: "4px",
                  animation: "spin 1s linear infinite",
                }}
              >
                <circle
                  cx="25"
                  cy="25"
                  r="20"
                  stroke="#888"
                  strokeWidth="4"
                  fill="none"
                  strokeDasharray="100"
                  strokeDashoffset="60"
                />
              </svg>
              正在更新...
            </div>
          )}
          {!updating && lastUpdateTime && (
            <div
              className="last-update"
              style={{ fontSize: "12px", marginRight: "8px" }}
            >
              上次更新: {lastUpdateTime}
            </div>
          )}
          <button
            onClick={handleRefresh}
            className="btn btn-icon"
            title="刷新"
            disabled={updating}
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
              <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16" />
            </svg>
          </button>
          <button
            onClick={handleOpenFolder}
            className="btn btn-icon"
            title="打开下载目录"
            disabled={updating}
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
            disabled={updating}
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
              <circle cx="12" cy="12" r="3" />
              <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 8.4 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4 15.4a1.65 1.65 0 0 0-1.51-1H2a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 3.6 8.4a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 8.6 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09c0 .68.39 1.3 1 1.51.61.21 1.29.05 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06c-.38.53-.54 1.21-.33 1.82.21.61.83 1 1.51 1H21a2 2 0 0 1 0 4h-.09c-.68 0-1.3.39-1.51 1Z" />
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

      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </div>
  );
}

export default App;
