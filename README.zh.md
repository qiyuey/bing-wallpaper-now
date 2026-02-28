# Bing Wallpaper Now

[English](README.md) | [中文](README.zh.md)

一个跨平台桌面应用，自动获取并设置 Bing 每日精美壁纸。

[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/qiyuey/bing-wallpaper-now/releases)
[![License](https://img.shields.io/badge/license-Anti--996-blue)](https://github.com/996icu/996.ICU)

## 📦 下载安装

从 [GitHub Releases](https://github.com/qiyuey/bing-wallpaper-now/releases) 下载最新版本：

- **Windows**: `.msi` 安装包或 `.exe` 便携版本
- **macOS**: `.dmg` 磁盘镜像
- **Linux**: `.deb` (Debian/Ubuntu) 或 `.rpm` (Fedora/RedHat) 或 `.AppImage` (通用版本)

### macOS 通过 Homebrew 安装

```bash
brew tap qiyuey/tap
brew install --cask bing-wallpaper-now
```

### macOS 安装说明

若出现"应用已损坏"或"无法打开"，在终端执行（需要管理员权限时可在前面加 sudo）：

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

## ✨ 为什么选择 Bing Wallpaper Now

### 轻量现代

- **Tauri 2.0 构建** — Rust 后端 + React 前端，非 Electron，内存占用低、启动快
- **6 平台原生支持** — macOS (Apple Silicon / Intel)、Windows (x64 / ARM64)、Linux (x64 / ARM64)
- **隐私优先** — 无遥测、无追踪，仅与 Bing API 通信，所有数据本地存储

### 竖屏壁纸

- **自动识别竖屏显示器**，为竖屏下载专属 1080×1920 壁纸
- 多显示器混合横竖屏场景下，每块屏幕自动匹配正确方向的壁纸

### macOS 深度优化

- **多显示器** — 通过 NSWorkspace 原生 API 同时为所有显示器设置壁纸
- **全屏应用** — 正确处理全屏 / Split View 场景，不丢壁纸
- **Space 自动恢复** — 切换 Space 或退出全屏时自动恢复壁纸并验证
- **纯托盘运行** — 不在 Dock 显示图标，不占任务栏

### 更多特性

- 📸 每日自动获取 Bing UHD 壁纸，一键设为桌面
- 🔄 应用内自动更新，带进度显示和签名校验
- 🎨 浅色 / 深色 / 跟随系统主题
- ⚙️ 自定义保存目录、壁纸市场、启动选项
- 🗑️ 自动清理旧缓存，控制磁盘占用

## ❓ 常见问题

**问：怎么设置壁纸？**
**答：** 点击任意壁纸卡片即可立即设为桌面。macOS 上会自动应用到所有显示器。

**问：关闭窗口后应用还在运行吗？**
**答：** 是的，关闭窗口只是最小化到系统托盘。点击托盘图标重新打开窗口，右键可退出。

**问：壁纸保存在哪里？**
**答：** 默认在系统图片目录下的"Bing Wallpaper Now"文件夹，可在设置中更改。

**问：占用多少存储空间？**
**答：** 每张 UHD 壁纸约 1-3MB，最少保留 8 张，约 8-24MB。应用会自动清理旧缓存。

**问：离线时可以使用吗？**
**答：** 可以，已下载的壁纸随时可以离线设置。

**问：为什么出现"市场不匹配"提示？**
**答：** Bing 在某些地区会忽略所选市场，返回其他地区的壁纸。应用会自动适配，不影响使用。

## 📞 支持与反馈

- **问题反馈**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **功能建议**: 随时欢迎！

## 🤝 参与贡献

想要帮助改进 Bing Wallpaper Now？欢迎贡献！

请阅读 [AGENTS.md](AGENTS.md) 了解开发环境搭建、编码规范、项目结构和工作流程。

## 📄 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

本项目同时支持[反 996 许可证](https://github.com/996icu/996.ICU)。我们倡导：

- ⏰ 合理的工作时间
- 🏖️ 工作与生活的平衡
- 💪 开发者的身心健康

**工作生活平衡很重要，向 996 说不！💪**

## 🙏 致谢

- [Bing](https://www.bing.com) - 提供精美的每日壁纸
- AI 辅助工具
