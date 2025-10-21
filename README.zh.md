# Bing Wallpaper Now

[English](README.md) | [中文](README.zh.md)

一个基于 Tauri 的跨平台桌面应用，每天自动获取并设置 Bing 精美壁纸。

![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![React](https://img.shields.io/badge/React-18-blue)
![TypeScript](https://img.shields.io/badge/TypeScript-5-blue)
![Rust](https://img.shields.io/badge/Rust-2024-orange)
![License](https://img.shields.io/badge/license-Anti--996-blue)

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

这会同时启动 Vite 开发服务器和 Tauri 应用窗口。

### 构建应用

```bash
pnpm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录。

## 📦 下载安装

从 [GitHub Releases](https://github.com/qiyuey/bing-wallpaper-now/releases) 下载预编译的安装包。

### macOS 安装说明

如果看到"应用已损坏，无法打开"或需要运行 `xattr` 命令，这是未签名开源应用的正常情况。

**快速解决：**
```bash
xattr -cr "/Applications/Bing Wallpaper Now.app"
```

详细解决方案和说明请参考 [macOS 安装指南](docs/MACOS_INSTALL.md)。

## 📁 项目结构

```bash
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
└── scripts/                      # 开发脚本
```

## ⚙️ 应用设置

- **自动更新** - 定期自动获取新壁纸，并自动应用最新一张
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
- **语言**: Rust（Edition 2024）
- **核心依赖**:
  - `reqwest` - HTTP 客户端
  - `serde` / `serde_json` - 序列化/反序列化
  - `chrono` - 日期时间处理
  - `anyhow` - 错误处理
  - `wallpaper` - 跨平台壁纸设置
  - `objc2` / `objc2-foundation` / `objc2-app-kit` - macOS 原生 API 绑定

### Tauri 插件

- `@tauri-apps/plugin-opener` - 打开文件/链接
- `@tauri-apps/plugin-dialog` - 原生对话框
- `@tauri-apps/plugin-store` - 配置持久化
- `@tauri-apps/plugin-autostart` - 开机启动
- `@tauri-apps/plugin-notification` - 系统通知

## 🗺️ 发展路线图

### 🎯 下一版本 (v0.2.0)

**多平台优化**
- [ ] Windows 多显示器支持增强
- [ ] Linux 桌面环境兼容（GNOME、KDE、XFCE 等）
- [ ] 不同平台的壁纸设置模式选择（填充、适应、拉伸等）

**界面优化**
- [ ] 深色模式 / 浅色模式切换
- [ ] 主题自定义（颜色方案）
- [ ] 网格布局自定义（卡片大小、列数）

**通知与反馈**
- [ ] 新壁纸下载完成通知
- [ ] 壁纸设置成功通知
- [ ] 更新进度显示（下载中、处理中）

**性能优化**
- [ ] 图片懒加载优化
- [ ] 缩略图生成与缓存
- [ ] 增量更新（只下载新壁纸）
- [ ] 内存占用优化

### 🌟 未来计划

**国际化**
- [ ] 多语言支持（英文、中文、日文等）
- [ ] 时区处理优化
- [ ] 区域化壁纸获取（不同地区的 Bing）

**AI 功能**
- [ ] AI 壁纸描述生成
- [ ] 壁纸颜色分析与主题色提取

**高级功能**
- [ ] 壁纸大图预览（点击卡片全屏查看）
- [ ] 壁纸收藏功能（标记喜欢的壁纸，防止被自动清理）
- [ ] 壁纸搜索/筛选（按日期、标题、关键词）
- [ ] 定时自动切换壁纸（每小时/每天/自定义）

## 🤝 参与贡献

我们欢迎任何形式的贡献！无论是问题反馈、功能建议、文档改进还是代码贡献。

### 开发流程

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码质量

提交 PR 前，请确保：

```bash
# 运行所有提交前检查
make pre-commit

# 或者分别运行各项检查
pnpm run lint          # ESLint 检查
pnpm run format:check  # Prettier 格式检查
pnpm run typecheck     # TypeScript 类型检查
pnpm run test:frontend # 前端测试
cargo fmt              # Rust 代码格式化
cargo clippy           # Rust 代码检查
cargo test             # Rust 测试
```

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

本项目同时支持[反 996 许可证](https://github.com/996icu/996.ICU/blob/master/LICENSE)。使用本软件即表示您同意遵守劳动法律法规，不强制员工无偿加班。

**附加条款：**
- 本软件仅供学习、研究和合法的个人使用
- 用户必须遵守当地劳动法律法规
- 禁止使用本软件剥削劳动者或侵犯劳动权益

**什么是反 996？**

反 996 许可证是开发者创建的软件许可证，旨在抗议科技公司中普遍存在的"996工作制"（早9点到晚9点，每周6天）。通过支持此许可证，我们倡导：

- ⏰ 合理的工作时间
- 🏖️ 工作与生活的平衡
- 💪 开发者的身心健康
- 🌟 可持续的软件开发

MIT © [Bing Wallpaper Now 贡献者](https://github.com/qiyuey/bing-wallpaper-now/graphs/contributors)

## 🙏 致谢

- [Bing](https://www.bing.com) - 提供精美的每日壁纸
- [Tauri](https://tauri.app) - 轻量级跨平台应用框架
- [React](https://react.dev) - 用户界面库
- [反 996 许可证](https://github.com/996icu/996.ICU) - 倡导开发者福祉

## 📞 联系与支持

- **问题反馈**: [GitHub Issues](https://github.com/qiyuey/bing-wallpaper-now/issues)
- **讨论交流**: [GitHub Discussions](https://github.com/qiyuey/bing-wallpaper-now/discussions)
- **贡献代码**: 随时欢迎 Pull Request！

---

**用 ❤️ 由开源社区制作**

**工作生活平衡很重要，向 996 说不！💪**
