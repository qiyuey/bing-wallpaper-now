# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bing Wallpaper Now is a Tauri-based desktop application that fetches and sets Bing daily wallpapers. The app is built with:
- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust (Tauri)
- **Architecture**: Cross-platform desktop app with native system integration

## Development Commands

### Setup
```bash
npm install
```

### Development
```bash
npm run dev
# or
npm run tauri dev
```
This starts both the Vite dev server and the Tauri development window.

### Building
```bash
npm run build              # Build frontend only
npm run tauri build        # Build complete app bundle
```

### Type Checking
```bash
npx tsc --noEmit
```

## Architecture

### Frontend Structure (`src/`)

- **App.tsx**: Main application component with header actions (refresh, open folder, settings)
- **hooks/useBingWallpapers.ts**: Core hook managing all wallpaper operations
  - Fetches Bing images via Tauri commands
  - Handles auto-download of fetched wallpapers in background
  - Manages local wallpaper cache
- **components/**: UI components (WallpaperCard, WallpaperGrid, Settings)
- **types/index.ts**: TypeScript type definitions matching Rust backend models

### Backend Structure (`src-tauri/src/`)

The Rust backend is organized into modules:

- **lib.rs**: Main Tauri entry point
  - Registers all Tauri commands
  - Manages global `AppState` (settings, wallpaper directory, tray click debounce)
  - Initializes plugins (opener, store, autostart, notification, dialog)
  - Sets up system tray icon with debounced click handling

- **bing_api.rs**: Bing API integration
  - Fetches wallpaper metadata from Bing's HPImageArchive API
  - Constructs high-resolution wallpaper URLs

- **wallpaper_manager.rs**: System wallpaper operations
  - Uses `wallpaper` crate for cross-platform wallpaper setting
  - Handles getting/setting desktop wallpaper

- **download_manager.rs**: Image downloading
  - Downloads wallpaper images from Bing

- **storage.rs**: File system operations
  - Manages wallpaper directory (default: `~/Pictures/Bing Wallpaper Now`)
  - Saves/loads wallpaper metadata as JSON
  - Handles cleanup of old wallpapers based on retention count

- **models.rs**: Data structures
  - `BingImageEntry`: Raw data from Bing API
  - `LocalWallpaper`: Local wallpaper with metadata
  - `AppSettings`: User preferences (auto-update, retention, launch at startup)

### Key Data Flow

1. **Fetch**: Frontend calls `fetch_bing_images` → Rust queries Bing API → Returns metadata
2. **Auto-download**: Frontend silently downloads all fetched images in background
3. **Set Wallpaper**: User clicks wallpaper → `download_wallpaper` (if needed) → `set_desktop_wallpaper` → System wallpaper updated
4. **Persistence**: Each downloaded wallpaper saved as `.jpg` + `.json` metadata file

### Tauri Commands (Frontend ↔ Backend)

All commands invoked via `invoke()` from `@tauri-apps/api/core`:

- `fetch_bing_images(count: u8)` - Get wallpaper list from Bing
- `download_wallpaper(image_entry: BingImageEntry)` - Download image to local storage
- `set_desktop_wallpaper(file_path: String)` - Set system wallpaper
- `get_local_wallpapers()` - List downloaded wallpapers
- `get_settings()` / `update_settings(settings: AppSettings)` - Settings management
- `cleanup_wallpapers()` - Remove old wallpapers per retention policy
- `get_current_wallpaper()` - Get current system wallpaper path
- `get_default_wallpaper_directory()` - Get default save location
- `ensure_wallpaper_directory_exists()` - Create wallpaper dir if missing

## Configuration

- **tauri.conf.json**: Tauri app configuration (window size, bundle settings, permissions)
- **Cargo.toml**: Rust dependencies and Tauri plugins
- **package.json**: Frontend dependencies and scripts

## Important Patterns

### Error Handling
- Rust functions return `Result<T, String>` for Tauri commands
- Frontend displays errors via state management in hooks

### State Management
- Rust: `AppState` managed via `Arc<Mutex<T>>` for thread-safe access
- Frontend: React hooks with local state

### File Naming
- Wallpapers: `{startdate}.jpg` (e.g., `20240315.jpg`)
- Metadata: `{startdate}.json` (e.g., `20240315.json`)

### Auto-download Strategy
- When fetching Bing images, all images are automatically downloaded in background
- Download happens silently without blocking UI
- If wallpaper already exists (checked via file path), download is skipped

## System Tray Functionality

### Tray Icon Behavior
- **Left Click**: Toggle window visibility (show/hide)
  - Uses 300ms debounce to prevent double-trigger issues
  - Window state tracked via `is_visible()` check
- **Right Click**: Show context menu with "显示窗口" and "退出" options
- **Window Close**: Hides window to tray instead of exiting application

### Debounce Implementation
- `AppState` includes `last_tray_click: Arc<Mutex<Option<Instant>>>` for tracking last click time
- Clicks within 300ms of previous click are ignored to prevent rapid toggle issues
- Ensures stable window show/hide behavior on all platforms

## File Management Features

### Opening Wallpaper Folder
- Uses `@tauri-apps/plugin-opener` to open wallpaper directory in system file explorer
- Automatically creates directory if it doesn't exist
- Works cross-platform (Finder on macOS, Explorer on Windows, file manager on Linux)

### Folder Selection in Settings
- Uses `@tauri-apps/plugin-dialog` for native folder picker
- Settings dialog includes "选择文件夹" button for wallpaper save directory
- Option to restore default directory with "恢复默认目录" button
- Selected path persisted in `AppSettings.save_directory`
