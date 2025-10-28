# Changelog

All notable changes to Bing Wallpaper Now will be documented in this file.

## 0.3.0

### Added

- 🎨 **主题切换功能**
  - 支持浅色/深色/跟随系统三种主题模式
  - 新增 ThemeContext 统一管理主题状态
  - 主题设置持久化，重启后保持用户选择
  - 自动响应系统主题变化（跟随系统模式下）
  - 全局样式适配，所有组件支持主题切换

- ℹ️ **关于页面**
  - 新增独立的关于对话框，展示应用信息
  - 显示应用版本号、功能介绍和技术栈
  - 包含项目链接和许可证信息
  - 从 package.json 自动导入版本号

- 💾 **设置存储优化**
  - 新增 settings_store.rs 模块，统一管理设置存储逻辑
  - 改进设置持久化机制，确保数据一致性
  - 优化设置加载性能

### Changed

- 🔧 **设置界面增强**
  - 设置对话框新增主题选择下拉框
  - 优化设置表单布局和交互
  - 改进版本号显示位置
  - 移除设置对话框底部的版本号，统一到关于页面

- 📦 **依赖更新**
  - 更新多个前端依赖到最新版本
  - 优化 ESLint 配置
  - 更新 Rust 依赖

### Fixed

- 🐛 **测试覆盖改进**
  - 新增 About 组件完整测试覆盖
  - 新增 ThemeContext 全面测试
  - 修复设置相关测试用例
  - 所有测试用例通过（104 个测试）

- 🎨 **样式优化**
  - 修复深色模式下的文字可读性
  - 优化主题切换过渡效果
  - 统一全局样式变量

## 0.2.1

### Added

- 🎨 **事件驱动的图片加载机制**
  - 后端下载完成时主动通知前端，替代轮询重试
  - 图片下载成功后立即触发前端自动加载
  - 减少不必要的加载尝试，提升用户体验

- ⚡ **并发图片下载优化**
  - 改为并发下载（最大并发数 4），大幅提升下载速度
  - 从顺序下载（16-24秒）优化到并发下载（7秒）
  - 保持元数据顺序一致性，页面显示按日期降序排列
  - 智能跳过已存在文件，避免重复下载

- 🚀 **首次启动体验优化**
  - 实现元数据优先加载：立即显示所有壁纸卡片（<1秒）
  - 图片后台下载，逐个显示，不阻塞 UI
  - 图片加载中显示友好的加载动画和提示
  - 等待后端下载通知而非盲目重试，避免误报"加载失败"

- 🔄 **智能图片加载策略**
  - 新增 `waitingForDownload` 状态，区分"等待下载"和"加载失败"
  - 只在确认下载完成后才标记加载失败
  - 手动重试按钮与"设置壁纸"按钮复用，界面更简洁

### Changed

- 📊 **GitHub Actions 缓存优化**
  - 优化 Rust 编译缓存策略
  - 调整 sccache 和 rust-cache 执行顺序
  - 添加 HTTP/2 和连接池优化配置

- 📝 **文档更新**
  - 合并性能优化计划到 README
  - 添加 UI/UX 增强路线图
  - 删除已完成的优化任务

### Fixed

- 🐛 **修复图片加载失败误报**
  - 移除过于激进的自动重试机制（3秒×3次）
  - 改为等待后端下载完成通知
  - 避免大图片（UHD 1-3MB）下载期间被误判为失败

- 🧪 **修复测试环境问题**
  - 全局 mock Tauri event API
  - 修复 WallpaperCard 测试中的事件监听器错误
  - 所有 77 个测试用例通过

## 0.2.0

### Added

- 🚀 **高效元数据存储系统** (Phase 2 性能优化)
  - 使用 MessagePack 格式的统一索引文件替代分散的 JSON 文件
  - 实现内存缓存机制，大幅减少磁盘 I/O 操作
  - 索引文件自动版本检查和管理
  - 壁纸列表加载速度显著提升

- 🔄 **并发图片下载** (Phase 1 性能优化)
  - 实现并发下载机制，同时下载多张壁纸（可配置并发数）
  - 全局 HTTP 客户端连接池复用，减少连接开销
  - 智能重试机制，支持指数退避策略
  - 流式下载减少内存占用
  - 下载速度提升 4 倍

- ⚡ **React 渲染优化** (Phase 1 性能优化)
  - 使用 React.memo 优化组件渲染
  - 实现 useMemo 缓存昂贵计算
  - 优化 useCallback 避免不必要的函数重建
  - 防止组件不必要的重渲染

- 🧪 **全面的测试覆盖**
  - 前端测试覆盖率从 84% 提升到 94%
  - 后端新增 12+ 个测试用例
  - 测试性能优化，运行时间从 20s+ 降至 1.5s
  - Settings 组件测试覆盖率从 50% 提升到 97%

- 📋 **开发者工具增强**
  - 新增 `AGENTS.md` 文档，记录 AI 代理协作历史
  - 增强 `make check` 流程，集成更多质量检查

### Changed

- 🏗️ **存储架构重构**
  - 从分散的 JSON 文件迁移到集中式 MessagePack 索引
  - 优化索引管理器，支持批量操作
  - 改进目录切换处理，支持多目录独立索引

### Fixed

- 🐛 **修复"暂无壁纸"问题**
  - 修复全局单例 IndexManager 导致的目录切换bug
  - 改用 HashMap 管理多目录的 IndexManager 实例
  - 确保设置中更改目录后能正确读取壁纸

### Removed

- 🗑️ **移除不必要代码**
  - 删除 JSON 迁移相关代码（应用无现有用户）
  - 清理测试中的网络依赖，提升测试速度
  - 删除性能优化计划文档，内容已合并到 README

### Performance

- 📈 **整体性能提升**
  - 壁纸下载速度提升 4 倍（并发 + 连接池）
  - 壁纸列表加载接近瞬时（内存缓存 + MessagePack）
  - 前端渲染性能优化，减少不必要的重渲染
  - 测试套件运行时间缩短 92%（从 20s 到 1.5s）

## 0.1.9

### Added

- ✨ **UI 版本号自动导入**: 从 package.json 自动导入版本号到设置界面，无需手动维护
- 📋 **CHANGELOG 验证**: 添加 pre-commit 检查，确保发布前 CHANGELOG 已更新
- 📝 **Release Notes 增强**:
  - 自动为所有平台安装包生成直接下载链接
  - 改进格式，更清晰的平台分类和说明

### Changed

- 🔄 **CI/CD 流程全面重构**: 分离 Release 和日常 CI 工作流
  - 将 `ci.yml` 重构为 `release.yml`，仅在 tag 推送时触发完整构建
  - 新增轻量级 `ci.yml` 用于 PR 和 main 分支的快速验证
  - 优化 Release 构建流程：前端单次构建 + 多平台并行 bundle
  - 使用 Tauri 2 默认签名，移除复杂的证书管理流程

- 🏗️ **脚本模块化重构**: 创建共享库架构，大幅提升代码复用性和可维护性
  - 新增 `scripts/lib/` 模块化库结构：
    - `ui.sh`: UI/输出工具（颜色、打印、格式化）
    - `git.sh`: Git 操作（状态检查、标签管理、提交）
    - `version.sh`: 语义化版本处理（解析、比较、增量）
    - `project.sh`: 项目配置（路径管理、工具检测）
    - `validators.sh`: 验证逻辑（CHANGELOG、代码质量）
  - 主脚本重命名优化：
    - `check-commit.sh` → `check-quality.sh`（更准确反映用途）
    - `version.sh` → `manage-version.sh`（避免与库文件命名冲突）
  - 简化 Makefile：移除冗余目标，保留核心命令（dev, check, snapshot-*, release）

### Fixed

- 🐛 **CI 构建修复**:
  - 修复 bundle artifact 路径，正确包含所有平台安装包
  - 修复 sccache 统计生成，完整显示缓存命中信息
  - 修复 Windows ARM64 的 pnpm store 路径检测
  - 仅上传可分发文件，过滤内部构建产物（如 .app.tar.gz）
- 🔧 **macOS 签名修复**:
  - 解决证书检测和 keychain 导入问题
  - 动态设置签名身份，适配不同构建环境
- 📝 **Release Notes 修复**: 下载链接使用点号而非空格，避免链接失效
- 🎨 **代码质量**: 修复 Clippy 警告，移除文档注释后的空行

### Performance

- 🚀 **构建优化**:
  - Release 构建：前端只构建一次，通过 artifact 共享给所有平台
  - 日常 CI：仅运行必要检查（lint + typecheck + Rust check），跳过完整构建
  - 预期节省 80%+ 的 CI 运行次数
  - 减少不必要的构建产物和上传流量

### Technical

- 代码复用：共享函数减少重复代码 900+ 行
- 易于维护：修改一次，所有脚本受益
- 清晰架构：统一的动词-名词命名模式
- 向后兼容：保持 Makefile 主要命令接口不变
- 简化签名：使用 Tauri 2 内置签名，无需管理证书

## 0.1.8

### Changed

- ⚡ **CI/CD 性能优化**: 改进构建缓存策略，加速 CI 构建流程
  - 新增 Vite 构建缓存（`node_modules/.vite`），支持增量构建
  - 优化缓存 key 策略：按 OS、架构和依赖锁文件哈希分组
  - 简化 CI summary 输出：优先显示 Rust Cache 状态
  - 仅在 Rust Cache 未命中时显示 sccache 统计信息
  - 减少冗余信息，聚焦关键指标（编译请求、缓存命中、命中率）

### Performance

- 📊 **缓存策略**:
  - Vite 缓存：跨构建共享，仅在依赖变更时失效
  - pnpm 缓存：GitHub Actions 原生支持
  - Rust Cache + sccache：多层缓存加速 Rust 编译
- 🚀 **预期提升**:
  - 首次构建：无变化（缓存未命中）
  - 后续构建：前端构建加速 30-70%（依赖未变时）

## 0.1.7

### Added

- 🍎 **macOS Dock 图标动态显示**: 根据窗口状态自动显示/隐藏 Dock 图标
  - 窗口关闭时隐藏 Dock 图标，仅保留菜单栏图标（类似原生 macOS 菜单栏应用）
  - 窗口显示时自动显示 Dock 图标
  - 使用 NSApplication activation policy 实现（Accessory/Regular 模式切换）
  - 新增 macos_app.rs 模块，封装 Objective-C 调用

- 🚀 **Release 流程完全自动化**: `make release` 命令现在完全无需人工确认
  - 自动验证工作目录干净（有未提交更改则报错退出）
  - 自动运行 pre-commit 检查（格式化、lint、测试）
  - 自动推送 release commit 和 tag 到远程
  - 自动创建下一个开发版本（如 0.1.7 → 0.1.8-0）
  - 自动推送开发版本到远程

- 🔄 **新增 `make rollback` 命令**: 快速回滚失败的发布
  - 删除本地和远程 tag
  - 回退到 HEAD~2（撤销 release + snapshot 提交）
  - 强制推送到远程
  - 需要输入 "yes" 确认以防误操作

### Changed

- 🎨 **macOS 菜单栏图标优化**: 改用单色模板图标，完美适配亮色/暗色模式
  - 使用白色前景 + alpha 通道，由 macOS 自动渲染为模板图标
  - 解决之前黑色图标在菜单栏显示为黑色方块的问题
  - 自动适应系统外观模式（亮色/暗色）

- ⬆️ **Rust 依赖升级**: 升级核心依赖到最新版本
  - tauri 2.9.0 → 2.9.1
  - tauri-build 2.5.0 → 2.5.1
  - tauri-plugin 2.5.0 → 2.5.1
  - wry 0.53.4 → 0.53.5
  - rustls 0.23.33 → 0.23.34
  - 以及其他间接依赖更新

### Fixed

- 🐛 **修复 Clippy 警告**:
  - 使用 let-chain 语法优化嵌套 if 语句
  - 使用 if-let 替代单一匹配的 match 语句

### Technical

- 添加 objc2-app-kit 和 objc2-foundation 依赖，用于 macOS 原生 API 调用
- 使用 MainThreadMarker 确保 Objective-C 调用的线程安全
- 平台特定编译：macOS 特性使用 #[cfg(target_os = "macos")]
- Release 脚本重构：移除所有人工确认提示，实现全自动化发布

## 0.1.6

### Changed

- ⬆️ **依赖升级**: 升级构建工具链和开发依赖到最新版本
  - Vite 6.4.1 → 7.1.11 (更快的构建和 HMR)
  - @vitejs/plugin-react 4.7.0 → 5.0.4
  - Vitest 3.2.4 → 4.0.1 (更好的测试性能)
  - @vitest/coverage-v8 3.2.4 → 4.0.1
  - @tauri-apps/cli 2.9.0 → 2.9.1
  - eslint-plugin-react-hooks 5.2.0 → 7.0.0
  - jsdom 25.0.1 → 27.0.1
  - 保持 React 18.3.1 稳定版本
  - 所有测试通过 (58/58)，代码覆盖率 83.33%

## 0.1.5

### Added

- 🔐 **Apple 代码签名支持**: 为 macOS 构建添加免费代码签名
  - 在 CI 中自动进行 ad-hoc 签名
  - 用户可以右键点击"打开"应用，无需使用终端命令
  - 添加完整的签名配置文档（SIGNING.md）

### Changed

- 📝 **改进 macOS 安装说明**: 更新 README 提供更清晰的安装指引
  - 说明应用已签名但使用免费签名
  - 提供三种打开应用的方法
  - 推荐使用右键点击方式打开

## 0.1.4

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

## 0.1.3

### Fixed

- 🐛 **修复 CI/CD 构建失败问题**:
  - 修复 Windows 构建中 PowerShell 语法错误，强制使用 bash shell 执行 changelog 提取脚本
  - 修复 macOS 构建中 artifact 名称冲突问题，使用 platform 名称替代 runner.os 以确保唯一性

### Changed

- 📝 **代码质量改进**: 添加 markdown 格式检查和配置优化

## 0.1.2

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

## 0.1.1

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

## 0.1.0

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
