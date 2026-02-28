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

若出现“应用已损坏”或“无法打开”，在终端执行（需要管理员权限时可在前面加 sudo）：

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

## ✨ 特性

### 核心功能

- 📸 **每日壁纸** - 自动获取 Bing 每日精选壁纸（最多 8 张）
- 🖼️ **高清分辨率** - 下载 UHD（超高清）分辨率壁纸
- 🎨 **一键设置** - 点击即可设置为桌面壁纸
- 📁 **本地管理** - 自动保存壁纸到本地，支持历史记录浏览
- 🔄 **后台更新** - 在后台自动下载新壁纸，不阻塞界面
- 🗑️ **自动清理** - 自动清理较旧缓存，控制磁盘占用

### macOS 专属功能

- 🖥️ **多显示器支持** - 同时为所有显示器设置壁纸
- 🎯 **全屏应用支持** - 完美处理全屏应用场景下的壁纸设置
- 🔄 **Space 自动恢复** - 切换 Space 或退出全屏时自动恢复壁纸

### 用户体验

- 🚀 **快速响应** - 优先加载本地缓存，后台获取远程数据
- 💾 **系统托盘** - 最小化到托盘，不占用任务栏空间
- ⚙️ **灵活配置** - 自定义保存目录、启动选项、市场与语言偏好
- 🎨 **主题支持** - 浅色、深色和跟随系统主题模式

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

- **自动更新** - 自动获取并应用最新壁纸
- **语言** - 界面语言（`Auto` / `zh-CN` / `en-US`）
- **壁纸市场** - 选择要获取的 Bing 壁纸市场（`mkt`）
- **保存目录** - 选择壁纸保存位置
- **主题** - 浅色 / 深色 / 跟随系统
- **开机自启动** - 登录系统后自动启动应用

> 提示：在某些地区，Bing 可能忽略你选择的市场并返回其他市场
> （例如强制返回 `zh-CN`）。应用会自动检测并使用实际返回市场建立索引，
> 同时在设置页显示提示信息。

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
**答：** 应用会自动清理较旧缓存以控制磁盘占用，当前还不支持“收藏/永不删除”功能。

**问：为什么会出现“市场不匹配”提示？**  
**答：** 这表示 Bing 实际返回的壁纸市场与你选择的不一致，通常由地区限制导致。
应用会继续正常工作，并自动使用实际返回市场，避免出现壁纸列表为空。

## 📞 支持与反馈

- **问题反馈**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **功能建议**: 随时欢迎！

## 🤝 参与贡献

想要帮助改进 Bing Wallpaper Now？欢迎贡献！

请先阅读 [代码库规范](AGENTS.md) 了解编码标准、工作流程和工具要求，然后查看下面的开发文档。

<details>
<summary><b>开发指南（面向贡献者）</b></summary>

### 开发环境要求

- Node.js 24+ (LTS)
- pnpm 10+（必需）
- Rust 1.80+ (Edition 2024)
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

**前端**：React 19, TypeScript, Vite

**后端**：Tauri 2.0, Rust (Edition 2024)

**核心库**：

- `reqwest` - HTTP 客户端
- `serde/serde_json` - 序列化
- `chrono` - 日期时间处理
- `wallpaper` - 跨平台壁纸设置
- `objc2` - macOS 原生 API 绑定

### 开发工作流

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/AmazingFeature`）
3. 提交更改（`git commit -m 'Add some AmazingFeature'`）
4. 推送到分支（`git push origin feature/AmazingFeature`）
5. 提交 Pull Request

### 代码质量检查

提交 PR 前请运行：

```bash
make check  # 运行所有检查

# 或单独运行：
pnpm run lint          # ESLint
pnpm run format:check  # Prettier
pnpm run typecheck     # TypeScript
cargo fmt              # Rust 格式化
cargo clippy           # Rust 检查
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
- AI 辅助工具
