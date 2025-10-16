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

## 🧪 CI/CD

本项目提供自动化持续集成与交付流程：

### GitHub Actions

已配置两个工作流（位于 `.github/workflows/`）：
- `ci.yml`：在 push / PR 时执行
  - Node & Rust 环境初始化与缓存
  - 前端 TypeScript 检查与构建
  - 前端 ESLint + Prettier（已整合进多平台 Quality Gates 而非单独 job）
  - Rust fmt + clippy（零警告策略）
  - Rust 单元测试（网络相关测试按需启用）
  - 多平台（Ubuntu / macOS / Windows）构建与产物上传
  - 覆盖率统计（Rust tarpaulin + 前端 Vitest，非阻断，continue-on-error）
  - 依赖/安全与许可证检查（cargo-deny）
- `release.yml`：当推送符合 `v*.*.*` 规则的标签时
  - 三平台打包 Tauri 应用
  - 汇总产物并创建 GitHub Release

Linux 环境需要的原生依赖（本地构建或 CI）：
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf
```

### 本地开发与验证

可使用内置脚本或 Makefile：
```bash
# 前端开发
npm run dev

# 本地 CI（类型检查 + 构建 + Rust 测试）
npm run ci
# 或:
make ci

# 打包 Tauri 应用
npm run tauri build
# 或:
make bundle
```

### 后续可拓展建议
- 接入 ESLint / Prettier / Rust clippy 进行代码质量检查
- 增加缓存命中率：针对 `Cargo.lock` 与 `package-lock.json`
- 增加发布签名（macOS / Windows）
- 增加测试覆盖率统计（例如 tarpaulin + 前端测试框架）
- 将快速重试与零点更新行为写入 Release Notes（自动生成）

**注意**: 本应用仅用于个人学习和使用，壁纸版权归 Bing 及相应版权方所有。

## 🛡️ 质量检查 (Quality Checks)

项目在本地与 CI 中都执行一套质量门槛（Quality Gates）：

### Rust
- 格式检查：`cargo fmt -- --check`
- 静态分析：`cargo clippy -D warnings`
- 单元测试：`cargo test --all-targets`
- 可选网络测试（Bing API）：默认忽略，通过设置环境变量 `BING_TEST=1` 再运行：
  ```bash
  BING_TEST=1 cargo test -- --ignored
  ```

### 前端 (TypeScript)
- 类型检查：`tsc --noEmit`
- 构建验证：`npm run build`
- （可选）ESLint：当前尚未启用，已在脚本中预留占位，可后续接入。

### 本地综合质量命令
使用 npm 脚本：
```bash
npm run fmt:check
npm run lint:rust
npm run typecheck
npm run test:rust
```
或使用 Makefile：
```bash
make check
```

### CI 中的执行顺序
1. Node & Rust 环境初始化与缓存
2. TypeScript 类型检查
3. Rust fmt & clippy（出现格式差异或警告则失败）
4. 前端构建
5. Rust 全目标测试
6. 可选网络测试（受 `BING_TEST` 控制）
7. cargo-deny 扫描（安全公告 / 许可证 / 重复 / 版本策略）
8. 产物上传（dist / 条件打包）

通过上述流程确保：
- 没有未格式化的 Rust 代码
- 没有静态分析警告
- 类型与构建可持续
- 许可证符合白名单策略
- 依赖不存在已披露的高危安全漏洞（RustSec）
- 网络不稳定不会拖慢默认 CI（测试被忽略）

### 前端 ESLint（已集成）

ESLint + Prettier 已内置于 `ci.yml` 的多平台 Quality Gates（与 fmt / clippy / typecheck 一起执行）。在本地可直接运行：

```bash
npm run lint:frontend          # 仅 ESLint
npm run format:check           # Prettier 检查
npm run lint                   # Rust + 前端汇总
```

若需自动修复：
```bash
npm run lint:frontend:fix
npm run format
```

配置位置：
- `.config/eslint/.eslintrc.cjs`
- `.config/prettier/.prettierrc` / `.prettierignore`

缓存：
CI 使用 `--cache --cache-location .cache/eslint/` 加速重复构建。

提升策略（后续可选）：
- 严格化 import 规则或添加自定义 alias resolver
- 引入项目级 TS Program（设置 parserOptions.project）提升类型规则精度

### cargo-deny 使用

配置文件位置：`src-tauri/deny.toml`  
（质量与安全相关配置已开始集中，相关 lint/格式化/覆盖率脚本逐步进入 `.config/` 目录）

本地运行：
```bash
cargo install --locked cargo-deny
cargo deny check
```

### 集中化质量与配置 (.config)

集中化目标：所有质量、脚本、格式化与度量相关内容不再散落在根目录，统一进入 `.config/` 及其子目录，便于：
- 结构清晰（根目录只保留核心源码与顶级说明）
- 渐进增强（新增工具时只需扩展子目录）
- CI/脚本复用（路径稳定、可直接引用）

当前子目录规划与状态：
- `.config/eslint/`：ESLint 主配置（已迁移 `.eslintrc.cjs`）
- `.config/prettier/`：Prettier 配置与忽略文件（已创建 `.prettierrc` / `.prettierignore`）
- `.config/scripts/`：复用脚本（质量检查、覆盖率、lint 聚合等）
- `.config/quality/`：质量基线与覆盖率策略说明（待补充文档）
- （可以新增：`.config/security/`、`.config/release/` 等）

当前已启用：
- `src-tauri/deny.toml`（安全与许可证策略）
- `.editorconfig`（跨语言基础格式约束）
- `.config/eslint/.eslintrc.cjs`（前端 lint 规则）
- `.config/prettier/.prettierrc` / `.prettierignore`（格式化策略）
- `.config/scripts/check-rust.sh`（Rust 基础质量门槛脚本）
- `.config/scripts/lint-frontend.sh`（前端 ESLint 运行脚本）
- `.config/scripts/coverage-rust.sh` / `.config/scripts/coverage-frontend.sh`（覆盖率采集脚本，占位实现）

后续扩展建议：
- 在 `.config/quality/README.md` 中补充：最低覆盖率阈值、允许的临时豁免策略
- 将 CI 步骤升级：新增 ESLint、Prettier 校验、覆盖率上传（Codecov 或 Badges）
- 增加 SBOM（例如使用 `cargo auditable` 或 CycloneDX）与发布签名策略

### Prettier 使用（前端格式化）

安装依赖：
```bash
npm i -D prettier eslint-config-prettier eslint-plugin-prettier
```

在 ESLint 中启用（示例，需修改 `.config/eslint/.eslintrc.cjs`）：
```js
extends: [
  "eslint:recommended",
  "plugin:react/recommended",
  "plugin:react-hooks/recommended",
  "plugin:@typescript-eslint/recommended",
  "plugin:import/recommended",
  "plugin:import/typescript",
  "plugin:prettier/recommended",
],
```

手动运行 Prettier：
```bash
npx prettier --config .config/prettier/.prettierrc --write "src/**/*.{ts,tsx,css,md}"
```

### 覆盖率统计

#### Rust 覆盖率 (tarpaulin)

安装：
```bash
cargo install cargo-tarpaulin
```

脚本（示例调用，脚本已放置）：
```bash
./.config/scripts/coverage-rust.sh
```

脚本可扩展为：
- 基线：行覆盖率 < 70% 即失败（可在脚本中添加解析并退出非零状态）
- 后续：区分单元测试与集成测试、生成 `coverage-report.xml` 供 CI 上传

#### 前端覆盖率 (Vitest + c8)

安装：
```bash
npm i -D vitest @vitest/coverage-v8
```

运行（占位脚本）：
```bash
./.config/scripts/coverage-frontend.sh
```

示例 `vitest.config.ts` 增补（待创建）：
```ts
export default {
  test: {
    coverage: {
      provider: "v8",
      reportsDirectory: "coverage-frontend",
      reporter: ["text", "lcov"],
      lines: 70,
      functions: 70,
      branches: 60,
      statements: 70,
    },
  },
};
```

### 前端 Lint 与格式化脚本

NPM 脚本（已更新）：
```bash
npm run lint:frontend
npm run lint          # 聚合 Rust + 前端
```

若需在 CI 启用 ESLint 与 Prettier，可在工作流添加步骤：
```yaml
- name: ESLint
  run: npm run lint:frontend

- name: Prettier (check only)
  run: npx prettier --config .config/prettier/.prettierrc --check "src/**/*.{ts,tsx,css,md}"
```

### 覆盖率集成到 CI（示例建议）

新增一个非阻断 job：
```yaml
jobs:
  coverage:
    runs-on: ubuntu-latest
    continue-on-error: true
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 18
      - uses: dtolnay/rust-toolchain@stable
      - run: npm ci
      - run: cargo install cargo-tarpaulin
      - run: ./.config/scripts/coverage-rust.sh
      - run: npm i -D vitest @vitest/coverage-v8
      - run: ./.config/scripts/coverage-frontend.sh
      # 可选：上传到 Codecov 或生成 badge
```

### 质量基线建议初版

| 维度 | 初始阈值 | 后续目标 |
| ---- | -------- | -------- |
| Rust 行覆盖率 | 70% | 80%+ |
| 前端行覆盖率 | 60% | 75%+ |
| Clippy 警告 | 0 | 0（保持） |
| TypeScript 编译错误 | 0 | 0 |
| 许可证违规 | 0 | 0 |
| RustSec 高危通告 | 0 | 0 |

如需我补充 `.config/quality/README.md` 具体清单或生成初始 Vitest 配置文件，继续告诉我即可。  

主要检查：
- 安全公告（advisories）：阻止已知漏洞库
- 许可证（licenses）：只允许 MIT / Apache-2.0 等
- 重复版本与潜在膨胀（duplicate）
- 不允许通配版本（wildcard-version）
- 可筛除未维护 / 含不安全标记库

CI 中已包含安装与运行步骤，可通过以下方式启用网络测试与升级策略：
```bash
# 仅在需要真实网络测试时手动开启
BING_TEST=1 cargo test -- --ignored
```

若需临时忽略特定安全公告，在 `deny.toml` 中添加：
```toml
[advisories]
ignore = ["RUSTSEC-YYYY-XXXX"] # 并在 PR / commit 说明理由
```

### 进一步建议
- 将 ESLint 与 Prettier 集成后添加格式校验
- 加入 `cargo clippy --all-targets --all-features`
- 添加覆盖率（Rust: tarpaulin；前端：vitest + c8）
- 对 Release 增加代码签名与 SBOM 输出

