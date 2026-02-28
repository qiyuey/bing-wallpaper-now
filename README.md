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
- 🗑️ **Automatic Cleanup** - Automatically removes older cache to control disk usage

### macOS Exclusive

- 🖥️ **Multiple Monitors** - Set wallpaper on all displays
- 🎯 **Fullscreen App Support** - Handles wallpapers in fullscreen usage
- 🔄 **Auto Restore on Space Switch** - Automatically restore wallpaper when switching Spaces or exiting fullscreen

### User Experience

- 🚀 **Fast Load** - Loads local cache first, fetches remote in the background
- 💾 **System Tray** - Minimize to tray, does not occupy taskbar
- ⚙️ **Configurable** - Custom save directory, startup options, and market/language preferences
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

- **Auto Update** - Automatically fetch and apply the latest wallpaper
- **Language** - UI language (`Auto` / `zh-CN` / `en-US`)
- **Wallpaper Market** - Choose which Bing market to fetch wallpapers from (`mkt`)
- **Save Directory** - Choose where to store wallpapers
- **Theme** - Light / dark / follow system
- **Launch at Startup** - Start app automatically after login

> Note: In some regions, Bing may ignore your selected market and return another one
> (for example, forcing `zh-CN`). The app will detect this automatically, use the
> actual returned market for indexing, and show a warning in Settings.

## ❓ FAQ

**Q: How often are wallpapers updated?**  
**A:** Bing releases new wallpapers daily. Enable "Auto Update" to fetch the latest automatically.

**Q: Where are my wallpapers saved?**  
**A:** By default, they're in a "Bing Wallpaper Now" folder in your system
pictures directory, but you can change this in settings.

**Q: Can I use it offline?**  
**A:** Yes! Previously downloaded wallpapers are always available and can still be set offline.

**Q: How much storage does it use?**  
**A:** Each UHD wallpaper is roughly 1–3MB. Keeping 8 uses 8–24MB.

**Q: Can wallpapers be kept forever?**  
**A:** The app automatically cleans older cached wallpapers over time to keep
storage usage predictable. A "favorites/never delete" option is not available yet.

**Q: Why does the app show a market mismatch warning?**  
**A:** This means Bing returned wallpapers for a different market than the one
you selected. It usually happens due to regional restrictions. The app keeps
working and uses the actual returned market to avoid empty lists.

## 📞 Support & Feedback

- **Report issues**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **Feature requests**: Always welcome!

## 🤝 Contributing

Want to help make Bing Wallpaper Now better? You're welcome!

Please read [AGENTS.md](AGENTS.md) for development setup, code style, project structure, and workflow guidelines.

## 📄 License

MIT License - see [LICENSE](LICENSE).

Project also supports the [Anti-996 License](https://github.com/996icu/996.ICU), advocating for:

- ⏰ Reasonable working hours
- 🏖️ Work-life balance
- 💪 Developer well-being

**Say NO to 996, prioritize your well-being! 💪**

## 🙏 Acknowledgments

- [Bing](https://www.bing.com) - Beautiful daily wallpapers
- AI-assisted tools
