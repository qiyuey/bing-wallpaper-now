# AGENTS.md

> Repository guidelines for AI coding agents working on Bing Wallpaper Now

## Setup commands

- Install deps: `pnpm install`
- Start dev server: `pnpm run tauri dev` (or `make dev`)
- Run tests: `pnpm test`
- Run quality checks: `make check`

## Prerequisites

- **Node.js**: 25+
- **Rust**: 1.80+ (Edition 2024)
- **pnpm**: 10.19.0 (specified in `packageManager` field)

## Project Overview

Bing Wallpaper Now is a cross-platform desktop application that automatically
fetches and sets Bing daily wallpapers. Built with Tauri 2.0, it combines a
React/TypeScript frontend with a Rust backend.

The app distinguishes between:

- UI language (`language` / `resolved_language`) for localization
- Bing wallpaper market (`mkt`) for content source selection

**Tech Stack:**

- Frontend: React 19, TypeScript, Vite
- Backend: Rust (Edition 2024), Tauri 2.0
- Testing: Vitest (frontend), Cargo test (backend)

### Common Commands

```bash
# Development
pnpm run dev                # Vite dev server only
pnpm run tauri dev          # Full Tauri app with hot reload
make dev MV=0.0.1           # Mock old version to test update flow

# Building
pnpm run build              # Build frontend (TypeScript compile + Vite build)
pnpm run tauri build        # Build production app for current platform
make package BUNDLES=dmg    # Build specific bundle format

# Type checking
pnpm run typecheck          # TypeScript type checking (tsc --noEmit)

# Linting & Formatting
pnpm run lint               # ESLint check
pnpm run lint:fix           # ESLint auto-fix
pnpm run lint:md            # Markdown linting
pnpm run lint:md:fix        # Markdown auto-fix
pnpm run format             # Prettier format code
pnpm run format:check       # Prettier check formatting

# Testing
pnpm test                   # Run all tests (Rust + frontend)
pnpm run test:frontend      # Vitest (React/TypeScript tests)
pnpm run test:rust          # Cargo test (Rust tests)

# Quality checks (runs before commit)
make check                  # Run all quality checks
make check NO_FIX=1         # Strict mode: check only, no auto-fix
make fix                    # Auto-fix all formatting and lint issues

# Version management
make patch                  # Bump patch version (1.0.0-0 -> 1.0.1-0)
make minor                  # Bump minor version (1.0.0-0 -> 1.1.0-0)
make major                  # Bump major version (1.0.0-0 -> 2.0.0-0)
make release                # Release current dev version and tag
make retag                  # Re-push current tag to trigger CI rebuild

# Cleanup
make clean                  # Clean build artifacts
make clean-all              # Deep clean (including Rust target)
```

## Project Structure

```text
bing-wallpaper-now/
├── src/                              # Frontend (React + TypeScript)
│   ├── App.tsx                      # Main app component
│   ├── App.css                      # App styles
│   ├── main.tsx                     # App entry point
│   ├── components/                  # React components
│   │   ├── Settings.tsx             # Settings dialog
│   │   ├── WallpaperCard.tsx        # Wallpaper display card
│   │   ├── WallpaperGrid.tsx        # Virtual-scrolled wallpaper grid
│   │   ├── UpdateDialog.tsx         # App update dialog
│   │   └── About.tsx               # About dialog
│   ├── hooks/                       # Custom React hooks
│   │   ├── useBingWallpapers.ts     # Wallpaper data fetching
│   │   ├── useSettings.ts          # Settings management
│   │   ├── useTrayEvents.ts        # System tray event integration
│   │   ├── useUpdateCheck.ts       # App update checking
│   │   └── useScreenOrientations.ts # Screen orientation detection
│   ├── config/                      # Frontend configuration
│   │   ├── icons.ts                # Icon props helpers
│   │   ├── layout.ts              # Layout breakpoints & sizing
│   │   └── ui.ts                   # UI spacing constants
│   ├── contexts/                    # React contexts
│   │   └── ThemeContext.tsx         # Theme (light/dark/system) provider
│   ├── i18n/                        # Internationalization
│   │   ├── I18nContext.tsx          # I18n context provider
│   │   └── translations.ts         # zh-CN / en-US translation strings
│   ├── utils/                       # Utility functions
│   │   ├── eventListener.ts        # Tauri event listener helpers
│   │   ├── notification.ts         # System notification helpers
│   │   └── transferHelpers.ts      # Import/export data helpers
│   ├── types/                       # TypeScript type definitions
│   │   └── index.ts
│   └── test/                        # Test utilities
│       ├── setup.ts                # Vitest setup
│       └── test-utils.tsx          # Render helpers for tests
├── src-tauri/                       # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── lib.rs                  # Tauri app setup & command registration
│   │   ├── main.rs                 # Binary entry point
│   │   ├── bing_api.rs             # Bing API integration
│   │   ├── index_manager.rs        # Local metadata index management
│   │   ├── wallpaper_manager.rs    # Wallpaper setting logic (macOS/Win/Linux)
│   │   ├── download_manager.rs     # Image download & caching
│   │   ├── settings_store.rs       # Persistent app settings store
│   │   ├── runtime_state.rs        # Runtime state persistence
│   │   ├── storage.rs              # File storage management
│   │   ├── update_cycle.rs         # Wallpaper update cycle orchestration
│   │   ├── auto_update.rs          # Background auto-update scheduler
│   │   ├── version_check.rs        # App version check helpers
│   │   ├── transfer.rs             # Import/export data transfer
│   │   ├── tray.rs                 # System tray menu & events
│   │   ├── utils.rs                # Language/mkt helper utilities
│   │   ├── commands/               # Tauri command handlers
│   │   │   ├── mod.rs
│   │   │   ├── wallpaper.rs        # Wallpaper commands
│   │   │   ├── settings.rs         # Settings commands
│   │   │   ├── storage.rs          # Storage/transfer commands
│   │   │   ├── mkt.rs              # Market status commands
│   │   │   └── window.rs           # Window management commands
│   │   └── models/                 # Shared data models
│   │       ├── mod.rs
│   │       ├── bing.rs             # Bing API response models
│   │       ├── settings.rs         # App settings model
│   │       ├── wallpaper.rs        # Wallpaper metadata model
│   │       ├── index.rs            # Index file models
│   │       └── runtime.rs          # Runtime state model
│   ├── Cargo.toml                   # Rust dependencies
│   ├── tauri.conf.json              # Tauri configuration
│   └── Info.plist                   # macOS bundle config (LSUIElement etc.)
├── .github/workflows/               # CI/CD
│   ├── ci.yml                       # PR/push checks + multi-platform cache warming
│   └── release.yml                  # Tag-triggered release builds
├── .cursor/skills/                  # AI agent skills
│   ├── bump-version/               # Version bump workflow
│   ├── release/                    # Release workflow
│   ├── review/                     # Code review workflow
│   └── update-deps/               # Dependency update workflow
├── scripts/                          # Build & utility scripts
│   ├── check-quality.sh             # Code quality checks
│   ├── manage-version.sh            # Version management
│   ├── validate-changelog.sh        # Changelog validation
│   ├── precheck.sh                  # Pre-build checks
│   ├── generate-icons.mjs           # App icon generation
│   └── lib/                         # Shared shell utilities
│       ├── version.sh
│       ├── ui.sh
│       ├── project.sh
│       ├── git.sh
│       └── validators.sh
├── Makefile                          # Development commands
└── package.json                      # Frontend dependencies & scripts
```

## Code style

- **TypeScript**: Strict mode enabled, follows Prettier defaults for quotes and semicolons
- **TypeScript/React naming**:
  - Components: PascalCase (`WallpaperCard.tsx`)
  - Hooks: camelCase with "use" prefix (`useBingWallpapers.ts`)
  - Files: Match component/hook name
- **React**: Functional components only, use hooks for state management, React 19+ (no need to import React in JSX files)
- **ESLint rules**: Unused vars warn (except with `_` prefix), `any` type warn,
  console warn (allow `console.warn` and `console.error`), React Hooks enforce
  rules-of-hooks (error), exhaustive-deps (warn)
- **Rust**: Edition 2024, use `cargo fmt --manifest-path src-tauri/Cargo.toml` for formatting, use `cargo clippy` for linting
- **Rust naming**: Files snake_case (`bing_api.rs`), functions snake_case, types PascalCase, constants SCREAMING_SNAKE_CASE
- **Rust patterns**: Use `anyhow::Result` for error handling, Tokio runtime
  (features = ["full"]) for async, add doc comments (`///`) for all public
  functions
- **Rust conditional compilation**: When `#[cfg(debug_assertions)]` blocks
  introduce mutability that only exists in debug mode, use `#[allow(unused_mut)]`
  to suppress the release-mode warning. This is the standard pattern for
  variables whose assignment is behind conditional compilation.
- **File organization**: Frontend source files in `src/`, backend source files
  in `src-tauri/src/`, tests colocated with source (`.test.ts`, `.test.tsx` for
  frontend), type definitions in `src/types/`

## Testing instructions

- Run `make check` to run all quality checks (format, lint, types, tests) before committing
- Run `pnpm test` to run all tests (Rust + frontend)
- Run `pnpm run test:frontend` for Vitest (React/TypeScript tests)
- Run `pnpm run test:rust` for Cargo test (Rust tests)
- Fix any test or type errors until the whole suite passes
- After moving files or changing imports, run `pnpm run lint` and
  `pnpm run typecheck` to ensure ESLint and TypeScript rules still pass
- Add or update tests for the code you change, even if nobody asked
- Frontend tests: `src/**/*.{test,spec}.{ts,tsx}`, coverage thresholds: Lines
  70%, Functions 40%, Branches 60%, Statements 70%
- Backend tests: Same files as implementation (`#[cfg(test)]` modules), run with `-- --nocapture` to see println! output

## PR instructions

- Always run `make check` before committing
- Run `pnpm run lint` and `pnpm run typecheck` before submitting PR
- Keep commits focused and atomic
- Write clear commit messages
- Add tests for new features
- Update documentation if needed

## Tauri-Specific Notes

### Plugin Permissions

The app uses several Tauri plugins with specific permissions configured in `src-tauri/capabilities/default.json`:

- **opener**: `default`, `allow-open-path` (scoped to `$PICTURE/**` and `$HOME/Pictures/**`)
- **dialog**: `default`, `allow-message`, `allow-open`, `allow-save`
- **store**: `default`, `allow-get`, `allow-set`
- **autostart**: `default`, `allow-enable`, `allow-disable`, `allow-is-enabled`
- **notification**: `default`
- **updater**: `default`
- **process**: `allow-restart`

When adding new plugin functionality, ensure proper permissions are configured.

### Updater Configuration

The app supports in-app auto-update via Tauri's updater plugin:

- `tauri.conf.json` → `bundle.createUpdaterArtifacts: true` enables `.sig` file generation during builds
- `tauri.conf.json` → `plugins.updater.pubkey` contains the minisign public key
- `tauri.conf.json` → `plugins.updater.endpoints` points to the `latest.json` on GitHub Releases
- Signing requires `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (configured as GitHub Secrets)
- The release workflow generates `latest.json` from uploaded `.sig` assets

### Commands

Rust functions exposed to frontend via `#[tauri::command]` macro:

- `get_local_wallpapers()` - Read local wallpaper metadata list
- `set_desktop_wallpaper(file_path: String)` - Set desktop wallpaper (with on-demand download if needed)
- `force_update()` - Trigger one update cycle immediately
- `get_settings()` / `update_settings(new_settings)` - Read/update app settings
- `get_wallpaper_directory()` - Get current wallpaper save directory
- `get_default_wallpaper_directory()` - Get default save directory
- `get_last_update_time()` / `get_update_in_progress()` - Query update status
- `add_ignored_update_version(version)` / `is_version_ignored(version)` - Manage ignored update versions
- See `src-tauri/src/lib.rs` for complete list

### Critical Event Chains (Manual Regression Checklist)

Cross-module event chains that are not fully covered by unit tests.
Verify these paths after structural refactoring or event-related changes:

1. **Tray → Manual Update Check → Show Window → Update Dialog**
   - `tray.rs` "check_updates" menu click
   - → `window.show()` + `window.set_focus()` + `emit("tray-check-updates")`
   - → frontend `useUpdateCheck` hook receives event
   - → `@tauri-apps/plugin-updater` `check()` + `is_version_ignored()`
   - → `UpdateDialog` renders (or system notification if no update)

2. **Tray → Force Update → Wallpaper Refresh**
   - `tray.rs` "refresh" menu click
   - → `update_cycle::run_update_cycle_internal(app, true)`
   - → download images → set wallpaper
   - → `emit("wallpaper-updated")` → frontend `useBingWallpapers` re-fetches

3. **Auto Update Cycle (Background)**
   - `auto_update::start_auto_update_task()` timer fires
   - → `update_cycle::run_update_cycle()` (same as above, silent)

4. **Tray → Open Settings / About / Folder**
   - `tray.rs` menu click → `emit("open-settings"/"open-about"/"open-folder")`
   - → frontend `useTrayEvents` hook dispatches callbacks
   - → respective panel/dialog shown or folder opened

5. **Market Mismatch Detection**
   - `update_cycle` detects actual mkt ≠ requested mkt
   - → `emit("mkt-status-changed")` → Settings.tsx re-fetches `get_market_status`
   - → warning banner shown if `is_mismatch`

6. **Startup Auto Update Check (Frontend)**
   - `useUpdateCheck` hook: 60s `setTimeout` → `@tauri-apps/plugin-updater` `check()`
   - → `invoke("is_version_ignored")` → silently set or skip `UpdateDialog`
   - (**Does NOT** go through tray event path; no window show/focus)
   - Update download/install uses plugin's `downloadAndInstall()` + `relaunch()`

### Settings & mkt mismatch behavior

- `AppSettings.language`: user preference (`auto` / `zh-CN` / `en-US`)
- `AppSettings.resolved_language`: resolved UI language for rendering
- `AppSettings.mkt`: Bing market code (independent from UI language)
- Backend emits `mkt-mismatch` when Bing actual market differs from requested `mkt`
- Runtime state persists `last_actual_mkt` to keep read/write index keys consistent across restarts

### Platform-Specific Code

- **macOS**: Uses objc2 bindings for native NSWorkspace, NSScreen APIs
- Handles multi-monitor wallpaper setting
- Supports Space switching and fullscreen app scenarios
- Dock 图标隐藏采用双重保障：`src-tauri/Info.plist` 的 `LSUIElement=true`
  （系统级声明）+ 启动时一次性 `setActivationPolicy(Accessory)` 运行时调用。
  单独依赖 Info.plist 不足以在所有场景下阻止 Dock 运行状态点出现。
  注意：仅在 setup 中调用一次，不要在 focus/reopen 等事件中重复调用以避免竞态
- **Windows/Linux**: 通过 `tauri.conf.json` 的 `skipTaskbar: true` 隐藏任务栏图标

## Build & Release Process

### Development Workflow

1. After a release, create a new development version:

   ```bash
   make patch  # Creates version like 1.0.1-0
   ```

2. Develop features, commit changes regularly

3. Before committing, run quality checks:

   ```bash
   make check  # Runs lint, format check, typecheck, tests
   ```

4. When ready to release:

   ```bash
   make release  # Removes -0 suffix, creates git tag, pushes
   ```

### Version Format

- **Development**: `X.Y.Z-0` (e.g., `1.0.0-0`)
- **Release**: `X.Y.Z` (e.g., `1.0.0`)
- Version is synchronized across:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`
  - `src-tauri/tauri.conf.json`

### CI/CD

**CI workflow** (`.github/workflows/ci.yml`):

- Triggered on PR and push to `main`
- Jobs: frontend checks (lint, typecheck, test), Rust checks (fmt, clippy, test)
- Multi-platform release cache warming (push to `main` only):
  builds `cargo build --release` on all 6 platforms to populate
  Rust cache for subsequent release builds

**Release workflow** (`.github/workflows/release.yml`):

- Triggered when version tags (`[0-9]*.*.*`) are pushed
- Builds for 6 platforms: Windows (x64, ARM64), macOS (Apple Silicon, Intel), Linux (x64, ARM64)
- macOS builds are code-signed with Apple Developer certificate
- Updater artifacts (`.sig` files) are generated and uploaded alongside bundles
- `latest.json` is generated from release assets for the in-app updater
- macOS `.app.tar.gz` files are renamed to include architecture (`_aarch64` / `_x64`)

**Cache strategy**: CI saves Rust release-profile cache on `main`
(`shared-key: release`). Release workflow reads this cache
(`save-if: false`). Different tags cannot share cache with each other
(GitHub Actions ref-scoping), but all tags can read from `main`.

## Important Files

- **`package.json`**: Frontend dependencies, scripts, version
- **`src-tauri/Cargo.toml`**: Rust dependencies, version
- **`src-tauri/tauri.conf.json`**: Tauri app configuration (updater, bundle, window settings)
- **`src-tauri/Info.plist`**: macOS bundle configuration（构建时与 Tauri 生成的
  plist 合并，用于 `LSUIElement` 等系统级声明）
- **`src-tauri/capabilities/default.json`**: Tauri plugin permissions
- **`eslint.config.js`**: ESLint flat config (modern format)
- **`vitest.config.ts`**: Vitest test configuration
- **`Makefile`**: Convenient command shortcuts
- **`scripts/check-quality.sh`**: Pre-commit quality checks

## Common Issues & Solutions

### Build Issues

**Issue**: Node.js version mismatch

- **Solution**: Ensure Node.js 25+ is installed. Use `node --version` to check.

**Issue**: Rust compilation errors

- **Solution**: Update Rust: `rustup update`. Ensure 1.80+ with edition 2024 support.

**Issue**: Tauri dev fails on macOS

- **Solution**: Install Xcode Command Line Tools: `xcode-select --install`

**Issue**: macOS Dock 图标意外出现运行状态（小圆点）

- **Solution**: 采用双重保障：确保 `src-tauri/Info.plist` 中 `LSUIElement`
  为 `true`，同时在 setup 中一次性调用 `setActivationPolicy(Accessory)`。
  单独依赖 Info.plist 不足以在所有 macOS 场景下阻止 Dock 运行状态点。
  不要在 focus/reopen 等高频事件中重复调用 `setActivationPolicy`，
  那会引起竞态条件和 Dock 图标闪烁。
  如果修改 Rust 代码后行为不符合预期，先执行 `cargo clean` 再重新编译，
  避免增量编译缓存干扰。

**Issue**: Rust 增量编译导致行为不一致

- **Solution**: 运行 `cd src-tauri && cargo clean && cd ..`，然后重新编译。
  平台特定行为（如 macOS Dock、窗口管理）修改后建议清理编译缓存再验证。

**Issue**: `unused_mut` warning in release vs `mut` required in debug

- **Solution**: When `#[cfg(debug_assertions)]` blocks introduce variable
  mutation that only exists in debug mode, add `#[allow(unused_mut)]` on the
  `let mut` binding. This suppresses the release-mode warning while keeping
  debug-mode compilation valid.

### Development Tips

1. **Hot Reload**: Use `pnpm run tauri dev` for full app hot reload including Rust changes
2. **Faster Frontend Iteration**: Use `pnpm run dev` for Vite-only mode when working on UI
3. **Type Safety**: Run `pnpm run typecheck` frequently to catch TypeScript errors early
4. **Pre-commit**: Always run `make check` before committing to catch issues
5. **Debugging Rust**: Use `log::debug!()` and enable Tauri logs in settings
6. **测试更新流程**: 使用 `make dev MV=0.0.1` 模拟旧版本来触发更新检测，
   无需修改任何配置文件。底层通过环境变量 `DEV_OVERRIDE_VERSION` 覆盖编译期
   版本号，仅在 debug 构建中生效，release 构建中完全移除。常用场景：
   - `make dev MV=0.0.1` — 模拟旧版本，触发更新
   - `make dev MV=99.0.0` — 模拟新版本，验证无更新
   - `make dev` — 不设置，使用真实版本号
7. **遇到问题先搜索**: 遇到平台特性、Tauri API、系统行为等问题时，**先联网
   搜索**相关问题及已知解决方案，再动手修复。很多问题（如 macOS Dock 行为、
   窗口管理、系统权限等）在社区中已有成熟的解决方案，避免用运行时 hack 解决
   本应在配置层面处理的问题

## External APIs

### Bing Wallpaper API

Endpoint: `https://www.bing.com/HPImageArchive.aspx`

Query parameters:

- `format=js` - JSON response
- `idx=0` - Start index (0 = today, 1 = yesterday, etc.)
- `n=8` - Number of images (max 8)
- `mkt=en-US` - Market/locale

Response contains:

- `images[]` - Array of wallpaper objects
- `url` - Partial URL path (needs to be prefixed with `https://www.bing.com`)
- `copyright` - Image attribution
- `title` - Image title

Notes:

- `mkt` in request may be ignored by Bing in some regions
- Use actual returned market (`actual_mkt`, parsed from response link) for metadata indexing

## Security & Privacy

- No analytics or tracking
- No external servers except Bing API
- All wallpapers stored locally
- Settings stored locally via Tauri plugin-store
- Updater signing keys stored in GitHub Secrets (never committed to repo)
- Open source under Anti-996 License

## Contributing Guidelines

1. Fork and create a feature branch
2. Follow code style conventions
3. Add tests for new features
4. Run `make check` before submitting PR
5. Update documentation if needed
6. Keep commits focused and atomic
7. Write clear commit messages

## License

MIT License + Anti-996 License

- Advocates for reasonable working hours
- Work-life balance
- Developer well-being

## References

- [Tauri Documentation](https://tauri.app)
- [React Documentation](https://react.dev)
- [Vite Documentation](https://vitejs.dev)
- [Vitest Documentation](https://vitest.dev)
- [Rust Book](https://doc.rust-lang.org/book/)
