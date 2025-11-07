# Bing Wallpaper Now

[English](README.md) | [中文](README.zh.md)

A cross-platform desktop app to automatically fetch and set Bing's daily beautiful wallpapers.

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qiyuey/bing-wallpaper-now/releases)
[![License](https://img.shields.io/badge/license-Anti--996-blue)](https://github.com/996icu/996.ICU)

## 📦 Download & Install

Get the latest version from [GitHub Releases](https://github.com/qiyuey/bing-wallpaper-now/releases):

- **Windows**: `.msi` installer or `.exe` portable
- **macOS**: `.dmg` disk image
- **Linux**: `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RedHat), or `.AppImage` (universal)

### macOS Installation Note

If you see "App is damaged or cannot be opened", run the following in Terminal (add `sudo` in front if needed):

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

## ✨ Features

### Core Features

- 📸 **Daily Wallpapers** - Automatically fetch up to 8 Bing daily wallpapers
- 🖼️ **High Resolution** - Download UHD (Ultra HD) wallpapers
- 🎨 **One Click Set** - Set as desktop wallpaper with a single click
- 📁 **Local Gallery** - Save all wallpapers locally, browse full history
- 🔄 **Background Fetch** - Downloads in the background, UI never blocked
- 🗑️ **Smart Cleanup** - Auto clean old wallpapers by retention count

### macOS Exclusive

- 🖥️ **Multiple Monitors** - Set wallpaper on all displays
- 🎯 **Fullscreen App Support** - Handles wallpapers in fullscreen usage
- 🔄 **Auto Restore on Space Switch** - Automatically restore wallpaper when switching Spaces or exiting fullscreen

### User Experience

- 🚀 **Fast Load** - Loads local cache first, fetches remote in the background
- 💾 **System Tray** - Minimize to tray, does not occupy taskbar
- ⚙️ **Configurable** - Custom save directory, retention policy, startup options
- 🎨 **Themes** - Light, dark, and system-follow modes

## 🎯 Usage

### First Launch

1. Download and install the app
2. Launch "Bing Wallpaper Now"
3. The app will fetch today’s Bing wallpaper
4. Browse the gallery in the main window

### Set Wallpaper

- **Click any wallpaper card** to immediately set it as your desktop wallpaper
- The wallpaper is applied instantly
- On macOS, it applies to all connected displays

### System Tray

The app runs in your system tray for quick access:

- **Left click** - Show/hide main window
- **Right click** - Menu (show window, quit)
- **Close window** - Minimize to tray (app keeps running)

### Settings

Click the "Settings" (star icon) to customize:

- **Auto Update** - Automatically fetch and set the latest wallpaper
- **Save Directory** - Choose where to store wallpapers
- **Retention Count** - How many wallpapers to keep (at least 8)

## ❓ FAQ

**Q: How often are wallpapers updated?**  
**A:** Bing releases new wallpapers daily. Enable "Auto Update" to fetch the latest automatically.

**Q: Where are my wallpapers saved?**  
**A:** By default, they're in a "Bing Wallpaper Now" folder in your system pictures directory, but you can change this in settings.

**Q: Can I use it offline?**  
**A:** Yes! Previously downloaded wallpapers are always available and can still be set offline.

**Q: How much storage does it use?**  
**A:** Each UHD wallpaper is roughly 1–3MB. Keeping 8 uses 8–24MB.

**Q: Can wallpapers be kept forever?**  
**A:** Currently, old wallpapers are auto-cleaned based on your retention setting (default up to 10000). Favorites feature planned in future.

## 📞 Support & Feedback

- **Report issues**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **Feature requests**: Always welcome!

## 🤝 Contributing

Want to help make Bing Wallpaper Now better? You're welcome!

Please read [AGENTS.md](AGENTS.md) for code style, workflow and tooling guidelines, then refer to the development docs below.

<details>
<summary><b>Developer Guide</b></summary>

### Requirements

- Node.js 24+ (LTS)
- Rust 1.80+ (Edition 2024)
- OS: macOS 10.15+ / Windows 10+ / Linux

### Install dependencies

```bash
pnpm install
```

### Development mode

```bash
pnpm run tauri dev
```

### Build app

```bash
pnpm run tauri build
```

Builds are in `src-tauri/target/release/bundle/`.

### Project Structure

```bash
bing-wallpaper-now/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/               # React components
│   ├── hooks/                    # React Hooks
│   └── types/                    # TypeScript type definitions
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

**Frontend**: React 19, TypeScript, Vite

**Backend**: Tauri 2.0, Rust (Edition 2024)

**Core crates**:

- `reqwest` - HTTP client
- `serde/serde_json` - Serialization
- `chrono` - Dates and time
- `wallpaper` - Cross-platform wallpaper setter
- `objc2` - macOS native APIs

### Workflow

1. Fork this repo
2. Create feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push your branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

### Quality Checks

Before submitting a PR, run:

```bash
make check  # Run all checks

# Or individually:
pnpm run lint          # ESLint
pnpm run format:check  # Prettier
pnpm run typecheck     # TypeScript
cargo fmt              # Rust formatting
cargo clippy           # Rust lints
cargo test             # Rust tests
```

</details>

## 📄 License

MIT License - see [LICENSE](LICENSE).

Project also supports the [Anti-996 License](https://github.com/996icu/996.ICU), advocating for:

- ⏰ Reasonable working hours
- 🏖️ Work-life balance
- 💪 Developer well-being

**Say NO to 996, prioritize your well-being! 💪**

## 🙏 Acknowledgments

- [Bing](https://www.bing.com) - Beautiful daily wallpapers
- [Claude Code](https://claude.com/code) - AI dev assistant for code generation
