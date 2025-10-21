# Bing Wallpaper Now

[English](README.md) | [中文](README.zh.md)

A cross-platform desktop application built with Tauri that automatically fetches and sets beautiful Bing daily wallpapers.

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![React](https://img.shields.io/badge/React-18-blue)
![TypeScript](https://img.shields.io/badge/TypeScript-5-blue)
![Rust](https://img.shields.io/badge/Rust-2024-orange)
![License](https://img.shields.io/badge/license-Anti--996-blue)

## ✨ Features

### Core Functionality

- 📸 **Daily Wallpapers** - Automatically fetch Bing's daily featured wallpapers (up to 8)
- 🖼️ **High Resolution** - Download in UHD (Ultra High Definition) resolution
- 🎨 **One-Click Setup** - Set as desktop wallpaper with a single click
- 📁 **Local Management** - Automatically save wallpapers locally with history browsing
- 🔄 **Background Updates** - Auto-download new wallpapers in the background without blocking UI
- 🗑️ **Smart Cleanup** - Automatically clean old wallpapers based on retention count

### macOS Exclusive Features

- 🖥️ **Multi-Monitor Support** - Set wallpaper for all displays simultaneously
- 🎯 **Fullscreen App Support** - Perfect handling of wallpaper setting in fullscreen scenarios
- 🔄 **Space Auto-Recovery** - Automatically restore wallpaper when switching Spaces or exiting fullscreen
- 🎪 **Native API** - Uses NSWorkspace API (objc2) for native experience

### User Experience

- 🚀 **Fast Response** - Prioritize loading local cache, fetch remote data in background
- 💾 **System Tray** - Minimize to tray without occupying taskbar space
- ⚙️ **Flexible Configuration** - Customize save directory, retention count, startup options
- 🌐 **Copyright Links** - Click images to visit detailed info pages
- 📂 **Quick Access** - One-click to open wallpaper save folder

## 🚀 Quick Start

### Prerequisites

- Node.js 22+ (LTS)
- Rust 1.80+ (Edition 2024)
- OS: macOS 10.15+ / Windows 10+ / Linux

### Install Dependencies

```bash
pnpm install
```

### Development Mode

```bash
pnpm run tauri dev
```

This will start both Vite dev server and Tauri application window.

### Build Application

```bash
pnpm run tauri build
```

Build artifacts are located in `src-tauri/target/release/bundle/` directory.

## 📦 Download

Download pre-built binaries from [GitHub Releases](https://github.com/qiyuey/bing-wallpaper-now/releases).

### macOS Installation Note

If you see "App is damaged and can't be opened" or need to run `xattr` command, this is normal for unsigned open-source apps. 

**Quick fix:**
```bash
xattr -cr "/Applications/Bing Wallpaper Now.app"
```

See [macOS Installation Guide](docs/MACOS_INSTALL.md) for detailed solutions and explanation.

## 📁 Project Structure

```bash
bing-wallpaper-now/
├── src/                          # Frontend code
│   ├── components/               # React components
│   │   ├── WallpaperCard.tsx    # Wallpaper card component
│   │   ├── WallpaperGrid.tsx    # Wallpaper grid layout
│   │   └── Settings.tsx         # Settings dialog
│   ├── hooks/                    # React Hooks
│   │   └── useBingWallpapers.ts # Core wallpaper management logic
│   ├── types/                    # TypeScript type definitions
│   ├── App.tsx                   # Main app component
│   └── main.tsx                  # Application entry
├── src-tauri/                    # Tauri backend
│   ├── src/
│   │   ├── bing_api.rs          # Bing API integration
│   │   ├── wallpaper_manager.rs # Wallpaper management (with macOS optimizations)
│   │   ├── download_manager.rs  # Image download manager
│   │   ├── storage.rs           # File storage management
│   │   ├── models.rs            # Data models
│   │   └── lib.rs               # Tauri app entry
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri app configuration
└── scripts/                      # Development scripts
```

## ⚙️ Application Settings

- **Auto Update** - Periodically fetch new wallpapers and automatically apply the latest one
- **Save Directory** - Customize wallpaper save location
- **Retention Count** - Set maximum number of wallpapers to keep (minimum 8)
- **Launch at Startup** - Start application with system

## 🖥️ System Tray

- **Left Click** - Show/hide main window (300ms debounce)
- **Right Click Menu** - Show window, exit application
- **Window Close** - Minimize to tray instead of exit

## 🛠️ Tech Stack

### Frontend

- **Framework**: React 18 + TypeScript
- **Build Tool**: Vite
- **State Management**: React Hooks
- **Styling**: CSS

### Backend

- **Framework**: Tauri 2.0
- **Language**: Rust (Edition 2024)
- **Core Dependencies**:
  - `reqwest` - HTTP client
  - `serde` / `serde_json` - Serialization/deserialization
  - `chrono` - Date/time handling
  - `anyhow` - Error handling
  - `wallpaper` - Cross-platform wallpaper setting
  - `objc2` / `objc2-foundation` / `objc2-app-kit` - macOS native API bindings

### Tauri Plugins

- `@tauri-apps/plugin-opener` - Open files/links
- `@tauri-apps/plugin-dialog` - Native dialogs
- `@tauri-apps/plugin-store` - Settings persistence
- `@tauri-apps/plugin-autostart` - Launch at startup
- `@tauri-apps/plugin-notification` - System notifications

## 🗺️ Roadmap

### 🎯 Next Release (v0.2.0)

**Multi-Platform Optimization**
- [ ] Windows multi-monitor support enhancement
- [ ] Linux desktop environment compatibility (GNOME, KDE, XFCE, etc.)
- [ ] Wallpaper fit modes for different platforms (fill, fit, stretch, etc.)

**UI Improvements**
- [ ] Dark mode / Light mode toggle
- [ ] Theme customization (color schemes)
- [ ] Grid layout customization (card size, column count)

**Notifications & Feedback**
- [ ] New wallpaper downloaded notification
- [ ] Wallpaper set success notification
- [ ] Update progress display (downloading, processing)

**Performance Optimization**
- [ ] Image lazy loading optimization
- [ ] Thumbnail generation and caching
- [ ] Incremental updates (only download new wallpapers)
- [ ] Memory usage optimization

### 🌟 Future Plans

**Internationalization**
- [ ] Multi-language support (English, Chinese, Japanese, etc.)
- [ ] Timezone handling optimization
- [ ] Regionalized wallpaper fetching (different regions of Bing)

**AI Features**
- [ ] AI-powered wallpaper description generation
- [ ] Wallpaper color analysis and theme color extraction

**Advanced Features**
- [ ] Wallpaper preview mode (click card for fullscreen view)
- [ ] Favorite wallpapers (prevent auto-cleanup)
- [ ] Search/filter wallpapers (by date, title, keywords)
- [ ] Scheduled wallpaper rotation (hourly/daily/custom)

## 🤝 Contributing

We welcome all forms of contributions! Whether it's bug reports, feature requests, documentation improvements, or code contributions.

### Development Workflow

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Code Quality

Before submitting a PR, please ensure:

```bash
# Run all pre-commit checks
make pre-commit

# Or run checks individually
pnpm run lint          # ESLint check
pnpm run format:check  # Prettier check
pnpm run typecheck     # TypeScript check
pnpm run test:frontend # Frontend tests
cargo fmt              # Rust format
cargo clippy           # Rust lint
cargo test             # Rust tests
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

This project also supports the [Anti-996 License](https://github.com/996icu/996.ICU/blob/master/LICENSE). By using this software, you agree to comply with labor laws and regulations, and not to force employees to work overtime without reasonable compensation.

**Additional Terms:**
- This software is for learning, research, and legitimate personal use
- Users must comply with local labor laws and regulations
- Prohibited from using this software to exploit workers or violate labor rights

**What is Anti-996?**

The Anti-996 License is created by developers to protest the "996 working hour system" (9am-9pm, 6 days a week) prevalent in some tech companies. By supporting this license, we advocate for:

- ⏰ Reasonable working hours
- 🏖️ Work-life balance
- 💪 Developer well-being
- 🌟 Sustainable software development

MIT © [Bing Wallpaper Now Contributors](https://github.com/qiyuey/bing-wallpaper-now/graphs/contributors)

## 🙏 Acknowledgments

- [Bing](https://www.bing.com) - For providing beautiful daily wallpapers
- [Tauri](https://tauri.app) - Lightweight cross-platform app framework
- [React](https://react.dev) - UI library
- [Anti-996 License](https://github.com/996icu/996.ICU) - Promoting developer welfare

## 📞 Contact & Support

- **Issues**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **Discussions**: [GitHub Discussions](https://github.com/qiyuey/bing-wallpaper-now/discussions)
- **Pull Requests**: Always welcome!

---

**Made with ❤️ by open source community**

**Work-life balance matters. Say NO to 996! 💪**
