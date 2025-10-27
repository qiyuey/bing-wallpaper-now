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

### macOS 安装说明

应用已经过代码签名，但由于使用的是免费签名（非 Apple Developer 账号），首次打开时：

**方法 1（推荐）**：右键点击应用 -> 选择"打开" -> 点击"打开"按钮

**方法 2**：在终端运行以下命令：

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

**方法 3**：前往"系统设置" -> "隐私与安全性" -> 找到应用 -> 点击"仍要打开"

## ✨ 特性

### 核心功能

- 📸 **每日壁纸** - 自动获取 Bing 每日精选壁纸（最多 8 张）
- 🖼️ **高清分辨率** - 下载 UHD（超高清）分辨率壁纸
- 🎨 **一键设置** - 点击即可设置为桌面壁纸
- 📁 **本地管理** - 自动保存壁纸到本地，支持历史记录浏览
- 🔄 **后台更新** - 在后台自动下载新壁纸，不阻塞界面
- 🗑️ **智能清理** - 根据保留数量自动清理旧壁纸

### macOS 专属功能

- 🖥️ **多显示器支持** - 同时为所有显示器设置壁纸
- 🎯 **全屏应用支持** - 完美处理全屏应用场景下的壁纸设置
- 🔄 **Space 自动恢复** - 切换 Space 或退出全屏时自动恢复壁纸

### 用户体验

- 🚀 **快速响应** - 优先加载本地缓存，后台获取远程数据
- 💾 **系统托盘** - 最小化到托盘，不占用任务栏空间
- ⚙️ **灵活配置** - 自定义保存目录、保留数量、启动选项
- 🌐 **版权链接** - 点击图片可访问详细介绍页面
- 📂 **快速访问** - 一键打开壁纸保存文件夹

## 🎯 使用方法

### 首次启动

1. 下载并安装应用
2. 启动"Bing Wallpaper Now"
3. 应用会自动获取今天的 Bing 壁纸
4. 在主窗口浏览壁纸画廊

### 设置壁纸

- **点击任意壁纸卡片**即可将其设置为桌面背景
- 壁纸会立即应用
- 在 macOS 上，会应用到所有连接的显示器

### 系统托盘

应用会驻留在系统托盘中，方便快速访问：

- **左键点击** - 显示/隐藏主窗口
- **右键点击** - 访问菜单（显示窗口、退出）
- **关闭窗口** - 最小化到托盘（应用继续运行）

### 设置选项

点击"设置"按钮（五角星图标）进行自定义：

- **自动更新** - 自动获取新壁纸并应用最新一张
- **保存目录** - 选择壁纸保存位置
- **保留数量** - 设置保留的壁纸数量（最少 8 张）
- **开机启动** - 系统启动时自动启动应用

### 其他功能

- **版权信息** - 点击壁纸上的版权链接了解更多信息
- **打开文件夹** - 点击"打开壁纸文件夹"查看所有保存的壁纸
- **历史记录** - 浏览并设置任何之前下载的壁纸

## ❓ 常见问题

**问：多久更新一次壁纸？**  
**答：** Bing 每天发布新壁纸。在设置中启用"自动更新"即可自动获取新壁纸。

**问：壁纸保存在哪里？**  
**答：** 默认保存在系统图片目录下的"Bing Wallpaper Now"文件夹中。您可以在设置中更改保存位置。

**问：离线时可以使用吗？**  
**答：** 可以！之前下载的壁纸可以离线使用，随时可以设置。

**问：占用多少存储空间？**  
**答：** 每张 UHD 壁纸约 1-3MB。最少保留 8 张壁纸，大约占用 8-24MB。

**问：可以永久保留壁纸吗？**  
**答：** 目前壁纸会根据保留数量自动清理。未来版本计划添加收藏功能。

## 🗺️ 发展路线图

### UI/UX 界面美化（下一步重点）

我们计划借助专业 AI 设计工具来增强用户界面和体验：

- 🎨 **现代化界面设计** - 采用当代设计模式重新设计界面
- 🌈 **视觉优化** - 改进配色方案、字体排版和间距布局
- ✨ **流畅动画** - 添加细腻的过渡效果和微交互
- 🎯 **优化交互流程** - 优化用户操作流程和交互体验
- 📱 **响应式布局** - 更好地适配不同窗口尺寸
- 🖼️ **增强图片展示** - 改进壁纸预览和网格布局

### 功能优化

- 深色模式 / 浅色模式切换
- 新壁纸系统通知
- 多语言支持
- 壁纸收藏功能

## 📞 支持与反馈

- **问题反馈**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **功能建议**: 随时欢迎！

## 🤝 参与贡献

想要帮助改进 Bing Wallpaper Now？欢迎贡献！

如果您有兴趣贡献代码，请查看下面的开发文档。

<details>
<summary><b>开发指南（面向贡献者）</b></summary>

### 环境要求

- Node.js 22+（LTS 版本）
- Rust 1.80+（Edition 2024）
- 操作系统：macOS 10.15+ / Windows 10+ / Linux

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm run tauri dev
```

### 构建应用

```bash
pnpm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

### 项目结构

```bash
bing-wallpaper-now/
├── src/                          # 前端（React + TypeScript）
│   ├── components/               # React 组件
│   ├── hooks/                    # React Hooks
│   └── types/                    # TypeScript 类型定义
├── src-tauri/                    # 后端（Rust + Tauri）
│   ├── src/
│   │   ├── bing_api.rs          # Bing API 集成
│   │   ├── wallpaper_manager.rs # 壁纸管理
│   │   ├── download_manager.rs  # 图片下载器
│   │   └── storage.rs           # 文件存储
│   └── Cargo.toml               # Rust 依赖
└── scripts/                      # 构建脚本
```

### 技术栈

**前端**: React 18, TypeScript, Vite

**后端**: Tauri 2.0, Rust（Edition 2024）

**核心库**:

- `reqwest` - HTTP 客户端
- `serde/serde_json` - 序列化
- `chrono` - 日期时间处理
- `wallpaper` - 跨平台壁纸设置
- `objc2` - macOS 原生 API 绑定

### 开发流程

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码质量

提交 PR 前：

```bash
make pre-commit  # 运行所有检查

# 或者分别运行：
pnpm run lint          # ESLint
pnpm run format:check  # Prettier
pnpm run typecheck     # TypeScript
cargo fmt              # Rust 代码格式化
cargo clippy           # Rust 代码检查
cargo test             # Rust 测试
```

</details>

## 📄 许可证

MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

本项目同时支持[反 996 许可证](https://github.com/996icu/996.ICU)。我们倡导：

- ⏰ 合理的工作时间
- 🏖️ 工作与生活的平衡
- 💪 开发者的身心健康

**工作生活平衡很重要，向 996 说不！💪**

## 🙏 致谢

- [Bing](https://www.bing.com) - 提供精美的每日壁纸
- [Tauri](https://tauri.app) - 轻量级跨平台应用框架
- 开源社区 - 持续的支持与贡献

---

用 ❤️ 由开源社区制作
