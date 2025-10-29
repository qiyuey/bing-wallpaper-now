# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bing Wallpaper Now is a cross-platform desktop application built with Tauri 2 that automatically fetches and sets Bing's daily wallpapers. The architecture follows a client-server pattern with a React frontend communicating with a Rust backend via Tauri's IPC system.

## Development Commands

### Setup

```bash
pnpm install                    # Install dependencies
```

### Development Workflow

```bash
pnpm tauri dev                  # Full app with hot reload (recommended)
make dev                        # Alternative command
pnpm dev                        # Frontend only (web UI, no backend)
```

### Code Quality (Run Before Commits)

```bash
make check                      # Run ALL quality checks (mandatory)
pnpm run typecheck              # TypeScript type checking
pnpm run lint                   # ESLint check
pnpm run lint:fix               # Auto-fix ESLint issues
pnpm run format                 # Format code with Prettier
pnpm run format:check           # Check formatting
pnpm run lint:md                # Check markdown files
pnpm run lint:md:fix            # Auto-fix markdown issues
cargo fmt --manifest-path src-tauri/Cargo.toml  # Format Rust code
cargo clippy --manifest-path src-tauri/Cargo.toml  # Lint Rust code
```

### Testing

```bash
pnpm test                       # Run both Rust and frontend tests
pnpm test:rust                  # Rust unit tests only
pnpm test:frontend              # Frontend tests only (Vitest + Testing Library)
pnpm test:frontend -- --coverage  # With coverage report
```

### Building

```bash
pnpm build                      # Build frontend (TypeScript + Vite)
pnpm tauri build                # Build complete app for distribution
```

### Version Management

```bash
make patch                      # Create patch dev version (0.1.0 -> 0.1.1-0)
make minor                      # Create minor dev version (0.1.0 -> 0.2.0-0)
make major                      # Create major dev version (0.1.0 -> 1.0.0-0)
make release                    # Release current version (removes -0 suffix), tag, and push
```

## Architecture

### Frontend (src/)

**React 19 + TypeScript + Vite** - The UI is a single-page application that displays wallpaper cards in a virtualized grid.

- **Components** (`src/components/`): Presentational React components
  - `WallpaperCard`: Individual wallpaper card with image preview
  - `WallpaperGrid`: Virtualized grid using react-window for performance
  - `Settings`: Settings panel with form inputs
  - `About`: About dialog

- **Hooks** (`src/hooks/`): Custom React hooks for state management
  - `useBingWallpapers`: Primary hook managing wallpaper state, fetch/refresh logic, and backend communication
  - `useSettings`: Settings persistence and synchronization

- **Contexts** (`src/contexts/`): React context providers
  - `ThemeContext`: Theme management (light/dark/system)

- **Types** (`src/types/`): Shared TypeScript interfaces
  - `LocalWallpaper`: Wallpaper metadata structure
  - `AppSettings`: Application settings structure

- **Event Communication**: Frontend listens to backend events via Tauri's event system:
  - `wallpaper-updated`: Refresh wallpaper list
  - `image-downloaded`: Single image download complete
  - `open-settings`: Tray menu triggered settings panel
  - `open-about`: Tray menu triggered about dialog
  - `open-folder`: Tray menu triggered folder open

### Backend (src-tauri/src/)

**Rust + Tauri 2** - The backend handles all system interactions, file management, and wallpaper operations.

- **Main Entry** (`lib.rs`): Application lifecycle, state management, and command registration
  - `AppState`: Global state with settings, wallpaper directory, update status
  - `run_update_cycle()`: Core update logic with retry mechanism and smart update checking
  - `start_auto_update_task()`: Background task with hourly polling and midnight alignment
  - System tray setup and event handling

- **Core Modules**:
  - `bing_api.rs`: Bing API integration (fetch image metadata)
  - `download_manager.rs`: Concurrent image downloader with retry logic
  - `wallpaper_manager.rs`: Cross-platform wallpaper setting
    - macOS: Multi-monitor support with Space recovery observer
    - Windows: Standard wallpaper API
    - Linux: Platform-specific wallpaper setting
  - `storage.rs`: File system operations (save/load wallpapers, cleanup)
  - `index_manager.rs`: Wallpaper metadata persistence (MessagePack format)
  - `settings_store.rs`: Settings persistence (Tauri plugin-store)
  - `runtime_state.rs`: Runtime state tracking (last update time, update checks)
  - `macos_app.rs`: macOS-specific app activation policy (Accessory mode for tray-only)

- **Tauri Commands** (IPC interface):
  - `get_local_wallpapers`: Fetch local wallpaper list (with redownload for missing files)
  - `set_desktop_wallpaper`: Set wallpaper (async, non-blocking)
  - `get_settings` / `update_settings`: Settings CRUD
  - `force_update`: Manual trigger for update cycle
  - `cleanup_wallpapers`: Remove old wallpapers based on retention count
  - `get_wallpaper_directory`: Get current save directory
  - `ensure_wallpaper_directory_exists`: Create directory if missing

### Update Strategy

The app uses a smart update system:

1. **Startup**: Loads persisted runtime state and checks if update needed
2. **Smart Check**: Skips update if already updated today AND local wallpapers exist
3. **Hourly Polling**: Checks for new wallpapers every hour
4. **Midnight Alignment**: Triggers at 00:00-00:05 local time with exponential backoff retry (up to 17 minutes)
5. **First Launch Optimization**: Immediately saves metadata so UI can show wallpapers while images download in background
6. **Concurrent Downloads**: Up to 4 parallel downloads using futures stream
7. **Auto-apply**: Automatically sets the latest wallpaper after successful update

### State Management Flow

```text
User Action (Frontend)
  → Tauri Command (IPC)
  → Rust Backend Handler
  → Update AppState
  → Emit Event (if needed)
  → Frontend Hook Listener
  → React State Update
  → UI Re-render
```

### Data Persistence

- **Settings**: Stored via tauri-plugin-store (JSON)
- **Wallpaper Metadata**: MessagePack format (`wallpaper-index.msgpack`)
- **Runtime State**: Persisted for smart update checks (`runtime-state.json`)
- **Images**: Saved as `YYYYMMDD.jpg` in configured directory

## Important Patterns

### Async/Non-blocking Operations

All wallpaper operations are async to avoid blocking the UI thread:

```rust
// Backend spawns async tasks
tauri::async_runtime::spawn(async move {
    wallpaper_manager::set_wallpaper(&path)
});
```

### Concurrent Safety

- `Arc<Mutex<T>>` for shared state across async tasks
- `watch` channel for broadcasting settings changes
- `update_in_progress` flag prevents concurrent update cycles

### macOS-specific Features

- **Multi-monitor**: Sets wallpaper for all connected displays
- **Space Recovery**: Observer detects Space switches and reapplies wallpaper
- **Accessory Mode**: App lives in tray only (no Dock icon)

### Windows-specific Optimizations

- High DPI tray icon: Uses 128x128 PNG for crisp display at 200% scaling
- Proper WebView2 cleanup on exit to prevent class registration errors

## Testing Strategy

- **Frontend**: Vitest + Testing Library (component and hook tests)
  - Tests colocated with components (`*.test.tsx`)
  - Setup file: `src/test/setup.ts`
  - Coverage thresholds: 70% lines, 40% functions, 60% branches

- **Rust**: Standard `cargo test` with unit tests
  - Tests in `#[cfg(test)]` modules
  - Focus on pure functions (validation, parsing)

## Key Configuration Files

- `package.json`: Frontend dependencies, scripts, version (must match Cargo.toml)
- `src-tauri/Cargo.toml`: Rust dependencies, version (must match package.json)
- `src-tauri/tauri.conf.json`: Tauri configuration, permissions, build settings
- `vitest.config.ts`: Test configuration and coverage thresholds
- `Makefile`: Development workflow automation

## Commit Workflow

1. Run `make check` - fixes all linting and verifies tests pass
2. Commit using Conventional Commits format: `type: description`
   - Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
3. Push to trigger CI/CD

## Release Workflow

1. Run `make check` to ensure quality
2. Review `git diff $(git describe --tags --abbrev=0)` for changes since last tag
3. Update `CHANGELOG.md` with `## x.y.z` section
4. Commit changes (do NOT push yet)
5. Run `make release` - removes `-0` suffix, creates git tag, pushes
6. GitHub Actions automatically builds and publishes release artifacts

## Plugin Permissions

The app requires these Tauri plugin permissions (configured in `tauri.conf.json`):

- `opener:allow-open-path` - Open wallpaper folder in file manager
- `dialog:allow-message`, `dialog:allow-open`, `dialog:allow-save` - Folder picker
- `store:allow-get`, `store:allow-set` - Settings persistence
- `autostart:allow-enable`, `autostart:allow-disable`, `autostart:allow-is-enabled` - Launch at startup
- `notification:default` - System notifications (planned feature)

## Anti-patterns to Avoid

- Don't use `git commit --amend` unless explicitly requested or fixing pre-commit hook issues
- Never skip hooks (`--no-verify`) unless user explicitly requests it
- Don't commit without running `make check` first
- Don't modify generated files in `src-tauri/gen/` or `target/`
- Avoid blocking operations in Tauri commands (always spawn async tasks)
- Don't create new files when editing existing ones would suffice
