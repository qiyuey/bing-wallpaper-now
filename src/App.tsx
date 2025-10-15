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
    fetchBingImages,
    fetchLocalWallpapers,
    setDesktopWallpaper,
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

  // 刷新壁纸列表
  const handleRefresh = async () => {
    // 先刷新本地列表（快速响应）
    fetchLocalWallpapers();

    // 然后在后台获取并下载新壁纸（不阻塞）
    // fetchBingImages 内部会自动下载并刷新本地列表
    fetchBingImages();
  };

  // 打开下载目录
  const handleOpenFolder = async () => {
    try {
      const folderPath = await invoke<string>("get_wallpaper_directory");
      console.log("Opening folder:", folderPath);

      // 确保目录存在
      await invoke("ensure_wallpaper_directory_exists");

      // 打开目录
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
        <div className="header-actions">
          <button onClick={handleRefresh} className="btn btn-icon" title="刷新">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M21 2v6h-6M3 12a9 9 0 0 1 15-6.7L21 8M3 22v-6h6M21 12a9 9 0 0 1-15 6.7L3 16"/>
            </svg>
          </button>
          <button
            onClick={handleOpenFolder}
            className="btn btn-icon"
            title="打开下载目录"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
            </svg>
          </button>
          <button
            onClick={() => setShowSettings(true)}
            className="btn btn-icon"
            title="设置"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>
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
