# Bing Wallpaper Now

一个基于 Tauri 的跨平台桌面应用，每天自动获取并设置 Bing 精美壁纸。

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![React](https://img.shields.io/badge/React-18-blue)
![TypeScript](https://img.shields.io/badge/TypeScript-5-blue)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange)

## ✨ 特性

### 核心功能

- 📸 **每日壁纸** - 自动获取 Bing 每日精选壁纸（最多 8 张）
- 🖼️ **高清下载** - 支持 UHD 超高清分辨率下载
- 🎨 **一键设置** - 点击即可设置为桌面壁纸
- 📁 **本地管理** - 自动保存壁纸到本地，支持历史记录浏览
- 🔄 **后台更新** - 自动在后台下载新壁纸，不阻塞界面
- 🗑️ **智能清理** - 根据保留数量自动清理旧壁纸

### macOS 特色功能

- 🖥️ **多显示器支持** - 同时为所有显示器设置壁纸
- 🎯 **全屏应用支持** - 完美处理全屏应用场景下的壁纸设置
- 🔄 **Space 自动恢复** - 切换 Space 或退出全屏时自动恢复壁纸
- 🎪 **原生 API** - 使用 NSWorkspace API (objc2) 实现原生体验

### 用户体验

- 🚀 **快速响应** - 优先加载本地缓存，后台获取远程数据
- 💾 **系统托盘** - 最小化到托盘，不占用任务栏空间
- ⚙️ **灵活配置** - 自定义保存目录、保留数量、启动选项
- 🌐 **版权链接** - 点击图片可访问详细介绍页面
- 📂 **快速访问** - 一键打开壁纸保存文件夹

## 🚀 快速开始

### 环境要求

- Node.js 18+
- Rust 1.70+
- 操作系统：macOS 10.15+ / Windows 10+ / Linux

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri dev
```

这会同时启动 Vite 开发服务器和 Tauri 应用窗口。

### 构建应用

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 📁 项目结构

```
bing-wallpaper-now/
├── src/                          # 前端代码
│   ├── components/               # React 组件
│   │   ├── WallpaperCard.tsx    # 壁纸卡片组件
│   │   ├── WallpaperGrid.tsx    # 壁纸网格布局
│   │   └── Settings.tsx         # 设置对话框
│   ├── hooks/                    # React Hooks
│   │   └── useBingWallpapers.ts # 壁纸管理核心逻辑
│   ├── types/                    # TypeScript 类型定义
│   ├── App.tsx                   # 主应用组件
│   └── main.tsx                  # 应用入口
├── src-tauri/                    # Tauri 后端
│   ├── src/
│   │   ├── bing_api.rs          # Bing API 集成
│   │   ├── wallpaper_manager.rs # 壁纸设置管理（含 macOS 优化）
│   │   ├── download_manager.rs  # 图片下载管理
│   │   ├── storage.rs           # 文件存储管理
│   │   ├── models.rs            # 数据模型
│   │   └── lib.rs               # Tauri 应用入口
│   ├── Cargo.toml               # Rust 依赖配置
│   └── tauri.conf.json          # Tauri 应用配置
└── CLAUDE.md                     # 开发指南（给 AI 的项目说明）
```

## 🎯 核心功能实现

### 壁纸获取与下载

- 使用 Bing HPImageArchive API 获取壁纸元数据
- 自动构造 UHD 高清图片 URL
- 后台异步下载，不阻塞用户界面
- 自动跳过已下载的壁纸

### 壁纸设置（跨平台）

#### macOS 优化实现

- 使用 `objc2` 生态系统与 Objective-C API 交互
- 使用 `NSWorkspace` API 直接设置壁纸
- 遍历所有 `NSScreen` 为每个显示器设置
- 正确处理全屏应用场景
- 监听 `NSWorkspaceActiveSpaceDidChangeNotification` 通知
- 使用 `declare_class!` 宏创建动态观察者类
- 自动处理 Space 切换和全屏应用场景

#### 其他平台

- 使用 `wallpaper` crate 提供跨平台支持

### 数据持久化

每张壁纸保存为两个文件：

- `{startdate}.jpg` - 壁纸图片
- `{startdate}.json` - 元数据（标题、版权、日期等）

默认保存位置：

- macOS: `~/Pictures/Bing Wallpaper Now`
- Windows: `%USERPROFILE%\Pictures\Bing Wallpaper Now`
- Linux: `~/Pictures/Bing Wallpaper Now`

## ⚙️ 应用设置

- **自动更新** - 定期自动获取新壁纸，并自动应用最新一张
- **更新间隔** - 自定义更新频率（小时）
- **保存目录** - 自定义壁纸保存位置
- **保留数量** - 设置最多保留的壁纸数量（最少保留 8 张）
- **开机启动** - 应用随系统启动

## 🖥️ 系统托盘

- **左键点击** - 显示/隐藏主窗口（300ms 防抖）
- **右键菜单** - 显示窗口、退出应用
- **窗口关闭** - 最小化到托盘而非退出

## 🛠️ 技术栈

### 前端

- **框架**: React 18 + TypeScript
- **构建工具**: Vite
- **状态管理**: React Hooks
- **样式**: CSS

### 后端

- **框架**: Tauri 2.0
- **语言**: Rust
- **核心依赖**:
  - `reqwest` - HTTP 客户端
  - `serde` / `serde_json` - 序列化/反序列化
  - `chrono` - 日期时间处理
  - `anyhow` - 错误处理
  - `wallpaper` - 跨平台壁纸设置
  - `objc2` / `objc2-foundation` / `objc2-app-kit` - macOS 原生 API 绑定
  - `once_cell` - 延迟初始化

### Tauri 插件

- `@tauri-apps/plugin-opener` - 打开文件/链接
- `@tauri-apps/plugin-dialog` - 原生对话框
- `@tauri-apps/plugin-store` - 配置持久化
- `@tauri-apps/plugin-autostart` - 开机启动
- `@tauri-apps/plugin-notification` - 系统通知

## 🎨 用户界面

- 简洁现代的卡片式布局
- 响应式网格布局，自适应窗口大小
- 图片懒加载优化性能
- 加载状态和错误提示
- 平滑的交互动画

## 🐛 已知问题

无重大已知问题。如遇到问题请提交 Issue。

## 📝 开发说明

### 代码格式化

```bash
# Rust 代码格式化
cargo fmt --manifest-path src-tauri/Cargo.toml

# TypeScript 代码检查
npx tsc --noEmit
```

### 调试

- 前端：使用浏览器开发者工具（在应用中右键 -> Inspect Element）
- 后端：查看终端输出的 Rust 日志

### 贡献指南

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

MIT License

## 🙏 致谢

- [Bing](https://www.bing.com) - 提供精美的每日壁纸
- [Tauri](https://tauri.app) - 轻量级跨平台应用框架
- [React](https://react.dev) - 用户界面库

---

**注意**: 本应用仅用于个人学习和使用，壁纸版权归 Bing 及相应版权方所有。
