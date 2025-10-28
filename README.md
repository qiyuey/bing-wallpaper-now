# Bing Wallpaper Now

[English](README.md) | [中文](README.zh.md)

A cross-platform desktop application that automatically fetches and sets beautiful Bing daily wallpapers.

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qiyuey/bing-wallpaper-now/releases)
[![License](https://img.shields.io/badge/license-Anti--996-blue)](https://github.com/996icu/996.ICU)

## 📦 Download & Install

Download the latest version from [GitHub Releases](https://github.com/qiyuey/bing-wallpaper-now/releases):

- **Windows**: `.msi` installer or `.exe` portable version
- **macOS**: `.dmg` disk image
- **Linux**: `.deb` (Debian/Ubuntu) or `.rpm` (Fedora/RedHat) or `.AppImage` (Universal)

### macOS Installation Note

The app is code-signed, but uses free signing (not from an Apple Developer account). On first launch:

Run this command in Terminal:

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

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

### User Experience

- 🚀 **Fast Response** - Prioritize loading local cache, fetch remote data in background
- 💾 **System Tray** - Minimize to tray without occupying taskbar space
- ⚙️ **Flexible Configuration** - Customize save directory, retention count, startup options
- 🎨 **Theme Support** - Light, dark, and system theme modes
- 🌐 **Copyright Links** - Click images to visit detailed info pages
- 📂 **Quick Access** - One-click to open wallpaper save folder

## 🎯 How to Use

### First Launch

1. Download and install the application
2. Launch "Bing Wallpaper Now"
3. The app will automatically fetch today's Bing wallpapers
4. Browse the wallpaper gallery in the main window

### Setting Wallpapers

- **Click any wallpaper card** to set it as your desktop background
- The wallpaper will be applied immediately
- On macOS, it applies to all connected displays

### System Tray

The app lives in your system tray for quick access:

- **Left Click** - Show/hide main window
- **Right Click** - Access menu (Show window, Exit)
- **Close Window** - Minimizes to tray (app keeps running)

### Settings

Click the "Settings" button (star icon) to customize:

- **Auto Update** - Automatically fetch new wallpapers and apply the latest one
- **Save Directory** - Choose where to store downloaded wallpapers
- **Retention Count** - Set how many wallpapers to keep (minimum 8)
- **Launch at Startup** - Start the app when your system boots

### Other Features

- **Copyright Info** - Click the copyright link on any wallpaper to learn more about it
- **Open Folder** - Click "Open Wallpaper Folder" to view all saved wallpapers
- **History** - Browse and set any previously downloaded wallpaper

## ❓ FAQ

**Q: How often does it update wallpapers?**  
**A:** Bing releases new wallpapers daily. Enable "Auto Update" in settings to fetch them automatically.

**Q: Where are wallpapers saved?**  
**A:** By default, they're saved in your system's pictures directory under "Bing Wallpaper Now". You can change this in Settings.

**Q: Does it work offline?**  
**A:** Yes! Previously downloaded wallpapers are available offline and can be set anytime.

**Q: How much storage does it use?**  
**A:** Each UHD wallpaper is about 1-3MB. With the minimum retention of 8 wallpapers, it uses approximately 8-24MB.

**Q: Can I keep wallpapers forever?**  
**A:** Currently, wallpapers are auto-cleaned based on retention count. A favorites feature is planned for future releases.

## 🗺️ Roadmap

### Planned Features

- 🔔 **System Notifications** - Get notified when new wallpapers are available
- 🌍 **Multi-language Support** - Internationalization (i18n)
- ⭐ **Wallpaper Favorites** - Save and quickly access your favorite wallpapers
- ✨ **UI/UX Enhancements** - Modern design with smooth animations and better layouts

## 📞 Support & Feedback

- **Report Issues**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **Feature Requests**: Always welcome!

## 🤝 Contributing

Want to help improve Bing Wallpaper Now? Contributions are welcome!

Check out our development documentation below if you're interested in contributing code.

<details>
<summary><b>Development Guide (for Contributors)</b></summary>

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

### Build Application

```bash
pnpm run tauri build
```

Build artifacts are located in `src-tauri/target/release/bundle/` directory.

### Project Structure

```bash
bing-wallpaper-now/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/               # React components
│   ├── hooks/                    # React Hooks
│   └── types/                    # TypeScript types
├── src-tauri/                    # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── bing_api.rs          # Bing API integration
│   │   ├── wallpaper_manager.rs # Wallpaper management
│   │   ├── download_manager.rs  # Image downloader
│   │   └── storage.rs           # File storage
│   └── Cargo.toml               # Rust dependencies
└── scripts/                      # Build scripts
```

### Tech Stack

**Frontend**: React 18, TypeScript, Vite

**Backend**: Tauri 2.0, Rust (Edition 2024)

**Key Libraries**:

- `reqwest` - HTTP client
- `serde/serde_json` - Serialization
- `chrono` - Date/time handling
- `wallpaper` - Cross-platform wallpaper setting
- `objc2` - macOS native API bindings

### Development Workflow

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Code Quality

Before submitting a PR:

```bash
make pre-commit  # Run all checks

# Or individually:
pnpm run lint          # ESLint
pnpm run format:check  # Prettier
pnpm run typecheck     # TypeScript
cargo fmt              # Rust format
cargo clippy           # Rust lint
cargo test             # Rust tests
```

</details>

## 📄 License

MIT License - see [LICENSE](LICENSE) file for details.

This project also supports the [Anti-996 License](https://github.com/996icu/996.ICU). We advocate for:

- ⏰ Reasonable working hours
- 🏖️ Work-life balance
- 💪 Developer well-being

**Work-life balance matters. Say NO to 996! 💪**

## 🙏 Acknowledgments

- [Bing](https://www.bing.com) - For providing beautiful daily wallpapers
- [Tauri](https://tauri.app) - Lightweight cross-platform app framework
- Open source community - For continuous support and contributions

---

Made with ❤️ by the open source community
