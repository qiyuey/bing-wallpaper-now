# Changelog

All notable changes to Bing Wallpaper Now will be documented in this file.

## 0.3.7

### Added

- 🌍 **多语言索引支持**
  - 索引文件（`index.json`）现在支持按语言分组存储壁纸元数据
  - 支持同时保存多个语言的壁纸信息（如中文和英文）
  - 切换语言时，可以重新加载对应语言的壁纸数据
  - 每个语言维护独立的索引，互不干扰

### Changed

- 🔧 **索引格式升级**
  - 索引版本从 v1 升级到 v2，支持多语言存储结构
  - 索引文件结构改为按语言分组：`wallpapers_by_language`（外层 key 为语言代码，内层 key 为 `start_date`）
  - 优化索引加载和保存逻辑，提升多语言支持的性能

- 🛠️ **代码质量改进**
  - 改进文件删除错误处理，确保索引与文件系统的一致性
  - 修复清理操作中的壁纸选择逻辑，使用 BTreeMap 确保一致性
  - 版本不匹配时自动保存旧索引备份，防止数据丢失

### Fixed

- 🐛 **修复文件删除错误处理**
  - 修复删除文件失败时索引仍被更新的问题
  - 现在只更新成功删除文件的索引，失败的文件会记录警告日志
  - 改进错误处理逻辑，确保索引与文件系统保持一致

- 🐛 **修复清理操作的确定性**
  - 修复 `get_all_wallpapers_unique()` 方法的不确定性行为
  - 使用 BTreeMap 按语言代码排序，确保多语言环境下的一致性
  - 优先选择字典序靠前的语言，避免随机选择

- 🐛 **修复版本不匹配处理**
  - 版本不匹配时自动保存旧索引备份到 `.backup` 文件
  - 提升日志级别为 error，更明确地标识问题
  - 防止数据丢失，便于后续恢复

## 0.3.6

### Added

- 🌍 **多语言支持（国际化）**
  - 新增中文（简体）和英文两种语言支持
  - 支持自动检测系统语言，或手动选择语言
  - 新增语言设置选项，支持"自动"、"中文"、"English"三种模式
  - 所有界面文本完整翻译，包括设置面板、错误提示、操作按钮等
  - 动态标语支持多语言显示，根据语言设置自动切换

- 🛠️ **代码审查工具**
  - 新增 `.cursor/commands/review.md` 代码审查命令
  - 支持架构质量检查（高内聚、低耦合、模块化）
  - 自动检测潜在 Bug 和重构机会
  - 提供快速改进建议和行动选项

- 📝 **代码质量改进**
  - 新增 `.gitattributes` 统一行尾符配置（LF），确保跨平台一致性
  - 优化 Makefile 命令，改进错误处理和提示信息
  - 新增工具函数模块（`utils.rs`），统一语言检测逻辑

- 📋 **索引文件格式优化**
  - 将索引文件从 MessagePack 格式改为 JSON 格式，提升可读性和调试便利性
  - 索引文件按时间排序，便于查看最新壁纸

### Changed

- 🔧 **后端架构优化**
  - 重构 `run_update_cycle_internal` 函数，拆分为更小的模块化函数
  - 提取 `apply_latest_wallpaper_if_needed`、`fetch_bing_images_with_retry`、`download_wallpapers_concurrently` 等函数
  - 改进错误处理，使用 `unwrap_or_else` 替代 `expect`，避免 panic
  - 优化日期计算逻辑，添加 fallback 机制处理边界情况
  - 优化索引管理器，使用 `tokio::sync::Mutex` 替代 `std::sync::Mutex`，避免阻塞异步运行时

- 🌐 **语言检测优化**
  - 统一语言检测逻辑到 `utils.rs` 模块
  - 改进系统时间回退检测，防止缓存逻辑异常
  - 优化 Bing API 市场代码获取逻辑

- 🎨 **UI 样式优化**
  - 增强 liquid glass（液态玻璃）效果，提升视觉质感
  - 优化壁纸卡片、按钮、状态徽章的玻璃态效果
  - 改进"已是最新"徽章样式，与整体设计风格统一
  - 移除"关于"页面中的技术栈信息，简化界面

- ⚙️ **设置功能增强**
  - 保留壁纸数量支持 0（不限制），但至少保留 8 张
  - 默认保留数量从 10000 改为 0（不限制）
  - 优化设置页面尺寸，确保在任何窗口大小下都能完整显示

- 🪟 **窗口尺寸优化**
  - 调整窗口最小尺寸为 380x580，确保至少能完整显示一个壁纸卡片
  - 优化标题布局，防止窗口过窄时标题换行

- 📍 **最近更新时间显示**
  - 恢复最近更新时间显示功能
  - 更新时间显示在标题下方，充分利用左侧空白区域
  - 优化更新时间的获取逻辑，优先从内存状态读取，必要时从索引文件读取

- 📦 **依赖更新**
  - 更新多个前端和后端依赖到最新版本
  - 添加 `num_cpus` 依赖用于并发下载优化
  - 移除 `rmp-serde` 依赖（不再使用 MessagePack）

### Fixed

- 🐛 **修复跨天更新问题**
  - 修复新一天开始时无法获取最新壁纸的问题
  - 改进运行时状态缓存逻辑，确保跨天时正确检查新壁纸
  - 修复手动刷新时强制更新的逻辑

- 🐛 **修复索引与文件同步问题**
  - 修复索引文件与实际文件不同步的问题
  - 改进文件存在性检查，确保缺失的文件会被重新下载
  - 优化索引加载逻辑，移除不必要的向后兼容代码

- 🐛 **修复 React Hook 依赖问题**
  - 修复 `useSettings` 中 `fetchSettings` 缺少依赖项警告
  - 修复 `App.tsx` 中 `handleOpenFolder` 缺少 `t` 依赖项

- 🐛 **修复 Rust 测试代码**
  - 修复测试中缺少 `language` 字段的问题
  - 改进错误处理，使用更安全的 unwrap 替代
  - 修复 Clippy 警告（`ptr_arg`、`manual_clamp`、`collapsible_if`）

- 🐛 **修复 ESLint 错误**
  - 修复 `useBingWallpapers.ts` 中 `NodeJS.Timeout` 类型问题
  - 修复 `useDynamicTagline.ts` 中 case 块词法声明问题
  - 修复 `translations.ts` 中 `navigator` 未定义问题

- 🐛 **修复前端测试**
  - 修复所有测试中的 `I18nProvider` 上下文问题
  - 创建 `renderWithI18n` 测试工具函数
  - 更新测试用例以匹配新的翻译文本

### Removed

- 🗑️ **移除"已是最新"功能**
  - 移除前端"已是最新"状态检查和显示
  - 简化更新逻辑，统一由后端处理

### Quality

- ✅ **代码质量提升**
  - 改进日期计算的错误处理，避免潜在 panic
  - 优化窗口隐藏的错误处理
  - 添加更完善的注释和文档
  - 移除未使用的代码和依赖

## 0.3.5

### Changed

- 🎨 **标题样式全面优化**
  - 增强"Bing Wallpaper"标题的对比度和可见性，优化颜色渐变和阴影效果
  - 提升字体粗细至 900，优化字体渲染和字母间距
  - 优化"Now"部分的发光效果，浅色和深色模式统一质感
  - 柔化标题边缘发光效果，使过渡更自然

- 🌈 **背景渐变优化**
  - 优化浅色模式背景渐变：粉色占比提升，过渡更平滑
  - 优化深色模式背景渐变：增强右下角蓝色显示，过渡更明显
  - 统一浅色和深色模式的过渡节奏，保持一致的视觉体验

- 📝 **文档更新**
  - 重组 AGENTS.md，优化项目结构和开发指引
  - 更新 README 和 README.zh，提升文档清晰度和一致性
  - 移除过时的文档文件

## 0.3.4

### Changed

- ⚙️ **设置面板改为即时保存体验**
  - 勾选开机自启动、自动更新、壁纸数量等选项立即保存，无需再点击“保存”按钮
  - 主题切换后即时写入配置并同步 UI，左上角新增版本号标签，便于快速确认当前版本
  - 调整设置面板布局与间距，目录路径支持省略号展示，避免长路径撑破弹窗
- 🎨 **桌面卡片间距和行高微调**：优化虚拟列表的行高与网格间距，让卡片对齐更紧凑、滚动更连贯

### Fixed

- 🌗 **主题应用逻辑修复**：ThemeContext 现在在保存设置后会立刻刷新页面主题，确保深浅色模式瞬时生效

### Quality

- 🛠️ **`make check` 自动修复能力增强**
  - Rust `cargo fmt`、ESLint、Prettier、MarkdownLint 失败时会尝试自动修复并重跑校验
  - 命令行输出新增提示，帮助快速定位仍需人工处理的检查项

### Tests

- ✅ **前端测试全面适配新行为**
  - Settings、App、hooks 测试改用 `waitFor` / `act` 处理异步状态
  - `src/test/setup.ts` 模拟 Tauri API，提升测试隔离性与稳定性

## 0.3.3

### Fixed

- 🐛 **修复 Tauri 事件监听器清理问题** (React StrictMode 兼容性)
  - 解决了 React StrictMode 双重挂载导致的 `TypeError: undefined is not an object (evaluating 'listeners[eventId].handlerId')` 错误
  - 创建安全的 unlisten 包装器，使其具有幂等性（可安全多次调用）
  - 改进所有事件监听器生命周期管理（useBingWallpapers.ts, App.tsx, ThemeContext.tsx）
  - 添加全局错误抑制机制处理 Tauri 内部边缘情况
  - 修复主题切换时的双重保存问题
  - 使用 useRef 模式提供稳定的处理器引用
  - 所有改进都遵循 Tauri v2 + React StrictMode 的社区最佳实践

- 🎨 **优化桌面窗口调整布局行为**
  - 添加窄窗口响应式断点（<1024px）：将卡片最小宽度从 380px 降至 320px，防止水平滚动
  - 在窄窗口下隐藏 "上次更新时间" 文本，防止头部按钮溢出视口
  - 添加缺失的 `.wallpaper-grid-empty-hint` 样式，创建视觉层级（小字号 + 降低透明度）
  - 修复空状态/加载状态容器布局：从横向改为纵向 flex 布局
  - 修复极窄窗口（≤750px）卡片重叠问题，强制单列布局

- 📱 **完整的响应式断点层级**（桌面端优化）
  - ≤750px: 1 张卡片/行（防止重叠）
  - 751-1024px: 2 张卡片/行
  - 1025-1919px: 3 张卡片/行（默认桌面）
  - ≥1920px: 4 张卡片/行（4K）

### Changed

- 🧹 **移除移动端和平板端响应式代码**
  - 应用专为桌面平台设计（macOS, Windows, Linux）
  - 删除不必要的移动端（<768px）和平板端（<1200px）媒体查询
  - 简化 WallpaperGrid.tsx 响应式逻辑，专注桌面体验
  - 保留 4K 断点（min-width: 1920px）支持大屏显示器
  - 代码更简洁，减少 22 行冗余代码

- 🔧 **简化版本管理命令**
  - 重命名命令：`snapshot-patch` → `patch`，`snapshot-minor` → `minor`，`snapshot-major` → `major`
  - 移除 release 后的自动 snapshot，开发者手动控制版本递增
  - 更清晰的版本管理工作流
  - 保持向后兼容（旧命令仍可用）

- 🔄 **替换 rollback 为 retag 命令**
  - 移除危险的 rollback 功能（删除标签、重置提交、强制推送）
  - 新增 `make retag` 命令：安全地重新推送版本标签
  - 用于重新触发 CI/CD 构建（如构建失败需重试）
  - 仅对发布版本有效，带有完整验证

### Technical

- 使用最新 Tauri 2.9.1（最新稳定版）
- 创建 `src/utils/eventListener.ts` 工具模块
- 所有测试通过（104 个测试）
- TypeScript 类型检查通过
- 简化的 Makefile 和版本管理脚本

## 0.3.2

### Changed

- 🔧 **简化 CI/CD 构建流程**
  - 移除 GitHub Actions release workflow 中的 Rust cache 配置
  - 移除 sccache 编译缓存设置，简化构建流程
  - 移除构建缓存统计摘要部分
  - 提升 CI 构建的稳定性和可靠性

### Fixed

- 🐛 **修复 Rust 编译警告**
  - 修复 `wallpaper_manager.rs` 中未使用变量的编译警告
  - 改进代码质量，消除潜在的警告信息

### Technical

- 优化 GitHub Actions 工作流配置
- 减少 CI 构建的复杂度和依赖项
- 提高跨平台构建的一致性

## 0.3.1

### Added

- 📝 **新增 CLAUDE.md 开发指南文档**
  - 为 Claude Code (claude.ai/code) 提供项目结构和开发指引
  - 包含架构说明、开发命令、测试策略等详细信息
  - 优化 AI 辅助开发体验

- 🏃 **运行时状态持久化模块** (`runtime_state.rs`)
  - 新增独立的运行时状态管理，与用户设置分离
  - 存储在隐藏文件 `.runtime.json` 中
  - 支持最后更新时间和检查时间的持久化

- 📋 **Release Body 模板**
  - 新增 `.github/release_body_template.md`
  - 标准化 Release 说明格式

### Changed

- 🚀 **下载管理器优化** (`download_manager.rs`)
  - 改进并发下载控制逻辑
  - 优化错误处理和重试机制
  - 提升下载稳定性

- 📦 **索引管理器增强** (`index_manager.rs`)
  - 优化壁纸索引的读写性能
  - 改进缓存机制
  - 增强错误处理

- 🎨 **前端组件优化**
  - `WallpaperCard.tsx`: 改进图片加载状态管理
  - `WallpaperGrid.tsx`: 优化虚拟列表渲染性能
  - `ThemeContext.tsx`: 增强主题切换逻辑

- 🔧 **构建配置更新**
  - 更新 `tauri.conf.json` 窗口配置
  - 优化 GitHub Actions 工作流 (`release.yml`)
  - 更新依赖版本（Cargo.toml, package.json）

### Fixed

- 🐛 **修复壁纸管理器问题** (`wallpaper_manager.rs`)
  - 解决多显示器设置壁纸的兼容性问题
  - 修复 macOS 下的壁纸设置逻辑

- 🧪 **测试环境修复**
  - 修复 `WallpaperGrid.test.tsx` 测试用例
  - 更新 `setup.ts` 测试配置
  - 确保所有测试通过

### Technical

- 更新 `models.rs` 数据模型定义
- 优化 `settings_store.rs` 设置存储逻辑
- 改进 `storage.rs` 文件存储管理
- 主程序 (`lib.rs`) 架构调整和优化

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
