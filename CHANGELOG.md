# Changelog

All notable changes to Bing Wallpaper Now will be documented in this file.

## [0.1.1]

### Added
- 🎯 **新增 macOS Intel (x64) 支持**: 现在同时提供 Intel 和 Apple Silicon 两个版本
- 📝 **测试覆盖增强**: 为数据模型添加了全面的单元测试

### Changed
- 🌏 **Windows 安装包中文化**: MSI 安装程序语言改为简体中文 (zh-CN)
- 🔧 **CI/CD 优化**: 
  - 分离 macOS Intel 和 Apple Silicon 构建流程
  - 优化 Rust 编译缓存策略（debug/release 分离）
  - 引入 sccache 加速编译
  - 迁移到 pnpm 包管理器

### Fixed
- 修复 macOS Intel 版本 dmg 缺失问题
- 修复 autostart 参数类型错误
- 修复 Clippy collapsible_if 警告

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

[0.1.1]: https://github.com/qiyuey/bing-wallpaper-now/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/qiyuey/bing-wallpaper-now/releases/tag/v0.1.0
