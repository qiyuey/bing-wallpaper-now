# Changelog

All notable changes to Bing Wallpaper Now will be documented in this file.

## [0.1.3]

### Fixed

- 🐛 **修复 CI/CD 构建失败问题**:
  - 修复 Windows 构建中 PowerShell 语法错误，强制使用 bash shell 执行 changelog 提取脚本
  - 修复 macOS 构建中 artifact 名称冲突问题，使用 platform 名称替代 runner.os 以确保唯一性

### Changed

- 📝 **代码质量改进**: 添加 markdown 格式检查和配置优化

## [0.1.2]

### Added

- 📋 **新增 CHANGELOG.md 文件**: 集中管理版本变更历史，Release Notes 自动从此文件提取
- 🎯 **新增 macOS Intel (x64) 构建支持**: 修复了 GitHub Actions macos-latest 迁移到 Apple Silicon 后缺失 Intel 版本的问题

### Changed

- 🌏 **Windows MSI 安装包语言改为中文**: 将 WiX 语言从 en-US 改为 zh-CN，更贴合中文用户
- 📦 **Release 说明增强**: 为所有平台安装包添加直接下载链接，用户无需滚动到页面底部

### Fixed

- 🛠️ **修复 macOS Intel (x64) dmg 缺失**: 在 CI 构建矩阵中显式添加 macos-13 (Intel) 和 macos-latest (Apple Silicon)

### Documentation

- 📝 **优化 macOS xattr 命令文档**: 将 `xattr -cr` 改为更精准的 `xattr -rd com.apple.quarantine`，只移除隔离属性

## [0.1.1]

### Added

- 📝 **测试覆盖增强**: 为数据模型（AppSettings, LocalWallpaper）添加了全面的单元测试
- ⚙️ **Maven 风格版本管理**: 引入 SNAPSHOT 版本机制，自动化版本号管理

### Changed

- 🔧 **CI/CD 性能优化**:
  - 引入 sccache 加速 Rust 编译
  - 优化 Cargo 缓存策略（debug/release 分离）
  - 迁移到 pnpm 包管理器，优化依赖管理

### Fixed

- 🐛 **修复 Windows 自动启动参数类型错误**: `vec!["--hidden"]` 类型从 `String` 改为 `&str`
- 🐛 **修复 Clippy 警告**: 使用 Rust 2024 let-chain 语法重构嵌套 if 语句
- 🐛 **修复 Windows 设置 UI 问题**: 解决设置对话框交互问题

## [0.1.0]

### Added

- 🎉 Initial release
- 📸 自动获取必应每日壁纸（最多 8 张）
- 🖼️ 支持 UHD 超高清下载
- 🎨 一键设置壁纸
- 📁 本地壁纸管理与历史记录
- 🔄 后台自动更新
- 💾 系统托盘集成
- ⚙️ 灵活配置（保存目录、保留数量、开机启动）

### macOS Exclusive

- 多显示器支持
- 全屏应用处理
- Space 自动恢复
- 原生 NSWorkspace API

[0.1.3]: https://github.com/qiyuey/bing-wallpaper-now/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/qiyuey/bing-wallpaper-now/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/qiyuey/bing-wallpaper-now/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/qiyuey/bing-wallpaper-now/releases/tag/v0.1.0
