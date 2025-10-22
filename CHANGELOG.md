# Changelog

All notable changes to Bing Wallpaper Now will be documented in this file.

## [0.1.4]

### Added

- 🏗️ **ARM64 架构支持**: 新增 Windows ARM64 和 Linux ARM64 构建
  - Windows ARM64: 支持 Surface Pro X 等 ARM 设备  
  - Linux ARM64: 支持树莓派 4/5 及 ARM 服务器
  - 现支持 6 个平台：Windows (x64/ARM64), macOS (x64/ARM64), Linux (x64/ARM64)

### Changed

- 🎨 **图标全面优化**: 重新设计应用图标，提升各平台显示效果
  - 最大化图标尺寸，填满 32x32 画布实现 100% 覆盖率
  - 采用圆角矩形设计，山景轮廓更加清晰
  - 移除不需要的移动平台图标（iOS/Android 共 30 个文件）
  
- ⚙️ **壁纸管理增强**: 
  - 默认保存数量从 8 张提升至 999 张
  - 设置界面支持最大值调整至 999（原为 200）
  
- 🔧 **CI/CD 流程重构**: 全面优化构建和发布流程
  - 实现统一的 bundle job，所有平台在同一任务中构建
  - 添加独立的 create-release job，解决并发创建 Release 的竞态条件
  - 优化依赖安装，移除不必要的包
  - 集成 sccache 加速 Rust 编译
  - 使用 Swatinem/rust-cache@v2 替代通用缓存方案
  
- 📝 **Release 说明改进**: 
  - Changelog 自动提取并置于 Release 说明顶部
  - 为所有安装包添加直接下载链接
  - 统一使用 "Apple Silicon" 术语

### Fixed

- 🐛 **Linux ARM64 交叉编译修复**: 彻底解决 Ubuntu apt 源配置问题
  - 完全重写 sources.list，正确分离 amd64 和 arm64 架构
  - 移除 Azure/Microsoft 特定源，避免干扰
  - 添加完整的交叉编译环境变量配置
  
- 🔧 **Changelog 提取修复**: 修复 awk/sed 命令无法正确提取版本更新内容的问题

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

[0.1.4]:
https://github.com/qiyuey/bing-wallpaper-now/compare/0.1.3...0.1.4
[0.1.3]: https://github.com/qiyuey/bing-wallpaper-now/compare/0.1.2...0.1.3
[0.1.2]: https://github.com/qiyuey/bing-wallpaper-now/compare/0.1.1...0.1.2
[0.1.1]: https://github.com/qiyuey/bing-wallpaper-now/compare/0.1.0...0.1.1
[0.1.0]: https://github.com/qiyuey/bing-wallpaper-now/releases/tag/0.1.0
