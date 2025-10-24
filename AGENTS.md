# AGENTS.md - Bing Wallpaper Now

> AI Agent Context Document - Version 1.0  
> 为 AI 开发工具（如 Claude Code、GitHub Copilot、Cursor 等）提供项目上下文

## 项目概述

**项目名称**: Bing Wallpaper Now  
**项目描述**: 一个跨平台桌面应用，自动获取并设置 Bing 每日精美壁纸  
**项目阶段**: 生产就绪 (v0.2.0-0)  
**代码仓库**: <https://github.com/qiyuey/bing-wallpaper-now>

### 核心功能

1. 自动获取 Bing 每日壁纸（最多8张）
2. 高分辨率 UHD 壁纸下载
3. 一键设置桌面壁纸
4. 系统托盘集成
5. 自动更新和后台下载
6. 智能清理旧壁纸
7. macOS 多显示器支持

## 技术栈

### 前端技术

- **框架**: React 18.3.1
- **语言**: TypeScript 5.9.0
- **构建工具**: Vite 7.1.11
- **包管理**: pnpm 10.19.0
- **UI框架**: 自定义 CSS，无第三方 UI 库

### 后端技术

- **框架**: Tauri 2.0
- **语言**: Rust (Edition 2024)
- **异步运行时**: Tokio 1.x (full features)
- **HTTP客户端**: Reqwest 0.12
- **序列化**: Serde + Serde JSON

### Tauri 插件

- `tauri-plugin-opener`: 打开文件/文件夹
- `tauri-plugin-store`: 持久化存储
- `tauri-plugin-autostart`: 开机自启
- `tauri-plugin-notification`: 系统通知
- `tauri-plugin-dialog`: 对话框
- `tauri-plugin-single-instance`: 单实例
- `tauri-plugin-log`: 日志系统

### 平台特定依赖

- **macOS**: objc2, objc2-foundation, objc2-app-kit
- **跨平台壁纸**: wallpaper crate
- **图像处理**: image crate

## 项目结构

```plaintext
bing-wallpaper-now/
├── src/                          # 前端源码 (React + TypeScript)
│   ├── App.tsx                  # 主应用组件
│   ├── App.css                  # 全局样式
│   ├── main.tsx                 # 应用入口
│   ├── components/              # React 组件
│   │   ├── Settings.tsx         # 设置对话框
│   │   └── WallpaperCard.tsx    # 壁纸卡片组件
│   ├── hooks/                   # 自定义 Hooks
│   │   └── useSettings.ts       # 设置管理 Hook
│   └── types/                   # TypeScript 类型定义
│       └── wallpaper.ts         # 壁纸相关类型
├── src-tauri/                    # 后端源码 (Rust + Tauri)
│   ├── src/
│   │   ├── lib.rs               # 库入口，注册命令
│   │   ├── main.rs              # 应用入口，系统托盘
│   │   ├── bing_api.rs          # Bing API 集成
│   │   ├── wallpaper_manager.rs # 壁纸管理核心逻辑
│   │   ├── download_manager.rs  # 并发下载管理
│   │   └── storage.rs           # 文件存储和清理
│   ├── Cargo.toml               # Rust 依赖配置
│   ├── tauri.conf.json          # Tauri 配置
│   └── capabilities/            # 权限配置
│       └── default.json         # 默认权限集
├── public/                       # 静态资源
│   └── icon.png                 # 应用图标
├── scripts/                      # 构建脚本
│   └── bump-version.sh          # 版本管理脚本
├── .github/                      # GitHub 配置
│   └── workflows/               # CI/CD 工作流
│       ├── ci.yml               # 持续集成
│       └── release.yml          # 发布流程
└── docs/                        # 文档
    └── ARCHITECTURE.md          # 架构文档
```

## 核心模块说明

### 前端模块

#### App.tsx

- 主应用组件，管理全局状态
- 处理壁纸获取、设置、下载
- 管理设置对话框和系统托盘交互

#### components/WallpaperCard.tsx

- 展示单个壁纸卡片
- 处理点击设置壁纸
- 显示壁纸信息（标题、日期、版权）

#### hooks/useSettings.ts

- 管理应用设置状态
- 与后端存储同步
- 处理设置变更

### 后端模块

#### bing_api.rs

- 获取 Bing 壁纸 API 数据
- 解析壁纸信息
- 构建高分辨率图片 URL

#### wallpaper_manager.rs

- 核心业务逻辑
- 管理壁纸生命周期
- 协调各模块工作
- 状态管理（AppState）

#### download_manager.rs

- 并发下载壁纸
- 连接池管理
- 错误重试机制
- 进度跟踪

#### storage.rs

- 文件系统操作
- 壁纸保存和清理
- 元数据管理
- 缓存策略

## API 接口 (Tauri Commands)

### 壁纸管理

- `fetch_wallpapers()` - 获取壁纸列表
- `set_wallpaper(url: String)` - 设置桌面壁纸
- `download_wallpaper(wallpaper: Wallpaper)` - 下载单个壁纸
- `get_local_wallpapers()` - 获取本地壁纸列表
- `refresh_wallpapers()` - 刷新壁纸（获取+下载）

### 设置管理

- `load_settings()` - 加载设置
- `save_settings(settings: Settings)` - 保存设置
- `get_wallpaper_directory()` - 获取当前壁纸目录
- `get_default_wallpaper_directory()` - 获取默认壁纸目录
- `set_wallpaper_directory(path: String)` - 设置壁纸目录

### 系统功能

- `open_wallpaper_folder()` - 打开壁纸文件夹
- `enable_autostart()` - 启用开机自启
- `disable_autostart()` - 禁用开机自启
- `is_autostart_enabled()` - 检查开机自启状态

## 数据模型

### Wallpaper

```rust
struct Wallpaper {
    startdate: String,      // 开始日期 (YYYYMMDD)
    fullstartdate: String,  // 完整开始日期时间
    enddate: String,        // 结束日期
    url: String,            // 图片 URL
    urlbase: String,        // 基础 URL
    copyright: String,      // 版权信息
    copyrightlink: String,  // 版权链接
    title: String,          // 标题
    quiz: String,           // 问答链接
    wp: bool,               // 是否为壁纸
    hsh: String,            // 哈希值
    drk: i32,               // 深色模式值
    top: i32,               // 顶部值
    bot: i32,               // 底部值
    hs: Vec<HotSpot>,       // 热点信息
}
```

### Settings

```rust
struct Settings {
    auto_update: bool,      // 自动更新
    wallpaper_directory: Option<String>,  // 壁纸目录
    retention_count: usize, // 保留数量 (最小8)
    launch_at_startup: bool,// 开机自启
}
```

## 编码规范

### TypeScript/React

- 使用函数组件和 Hooks
- 使用 TypeScript 严格模式
- 组件命名：PascalCase
- Hook 命名：use 前缀
- 避免 any 类型

### Rust

- 遵循 Rust 官方编码规范
- 使用 `anyhow::Result` 处理错误
- 使用 `log` crate 记录日志
- 异步函数使用 `async/await`
- 合理使用 `Arc<Mutex<T>>` 管理共享状态

### 文件命名

- TypeScript: camelCase.ts/tsx
- Rust: snake_case.rs
- 组件: PascalCase.tsx

## 构建和运行

### 开发环境

```bash
# 安装依赖
pnpm install

# 开发模式
pnpm run tauri dev

# 类型检查
pnpm run typecheck

# 代码格式化
pnpm run format
```

### 生产构建

```bash
# 构建应用
pnpm run tauri build

# 输出位置
# src-tauri/target/release/bundle/
```

### 测试

```bash
# Rust 测试
pnpm run test:rust

# 前端测试
pnpm run test:frontend

# 全部测试
pnpm test
```

## 平台特定注意事项

### macOS

- 需要 macOS 10.15+
- 使用 NSWorkspace API 设置壁纸
- 支持多显示器
- 处理全屏应用和 Space 切换
- 需要代码签名（使用免费签名）

### Windows

- 需要 Windows 10+
- 使用 Windows API 设置壁纸
- MSI 安装包支持中文

### Linux

- 支持 GNOME、KDE、XFCE 等桌面环境
- 使用 wallpaper crate 跨桌面环境设置

## 权限配置

应用需要以下权限（配置在 capabilities/default.json）：

### 文件系统

- `fs:read` - 读取文件
- `fs:write` - 写入文件
- `fs:scope` - 访问图片目录

### 插件权限

- `opener:allow-open-path` - 打开文件夹
- `dialog:allow-*` - 对话框操作
- `store:allow-*` - 持久化存储
- `autostart:allow-*` - 开机自启
- `notification:default` - 系统通知

### 网络权限

- `http:default` - HTTP 请求
- 允许访问 Bing API 域名

## 常见任务指南

### 添加新功能

1. 在 `src-tauri/src/lib.rs` 添加 Tauri 命令
2. 在对应模块实现功能逻辑
3. 在前端添加调用接口
4. 更新类型定义
5. 添加相应测试

### 修改 UI

1. 组件在 `src/components/`
2. 样式在 `src/App.css` 或组件内
3. 使用 React Hooks 管理状态
4. 保持响应式设计

### 调试问题

1. Rust 日志：使用 `log::debug!`
2. 前端日志：使用 `console.log`
3. 开发者工具：F12 或右键检查
4. Tauri 日志：查看终端输出

## 版本管理

- 使用语义化版本 (SemVer)
- 版本同步：package.json、Cargo.toml、tauri.conf.json
- 使用 `scripts/bump-version.sh` 脚本管理版本
- Git 标签格式：`v0.1.0`

## CI/CD 流程

### 持续集成 (ci.yml)

- 触发：Push 到 main 或 PR
- 步骤：依赖安装 → 类型检查 → 测试 → 构建

### 发布流程 (release.yml)

- 触发：推送版本标签 (v*)
- 平台：Windows、macOS、Linux
- 产物：安装包上传到 GitHub Releases
- 自动生成 Changelog

## 性能优化要点

### 前端

- 使用 React.memo 优化渲染
- 懒加载图片
- 虚拟列表（未来）

### 后端

- 并发下载，连接池复用
- 异步 I/O，避免阻塞
- 智能缓存策略
- 流式处理大文件

## 安全考虑

- CSP 策略配置
- 资源访问范围限制
- HTTPS 强制使用
- 输入验证和清理
- 避免路径遍历攻击

## 开发建议

1. **保持简洁**: 功能专注，避免过度工程
2. **用户体验优先**: 响应快速，操作流畅
3. **跨平台兼容**: 测试不同平台表现
4. **错误处理**: 优雅降级，用户友好提示
5. **文档完善**: 代码注释，README 更新

## 相关资源

- [Tauri 文档](https://tauri.app/v2/guides/)
- [React 文档](https://react.dev/)
- [Rust 文档](https://doc.rust-lang.org/)
- [Bing 壁纸 API](https://www.bing.com/HPImageArchive.aspx)

---

*此文档为 AI 开发工具提供项目上下文，请在重大变更后更新*  
*最后更新: 2025-10-24*
