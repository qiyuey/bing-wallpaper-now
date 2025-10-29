# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Bing Wallpaper Now is a cross-platform desktop application that automatically fetches and sets Bing daily wallpapers. It's built with Tauri 2.0 (Rust backend) and React 18 (TypeScript frontend).

## Development Commands

### Quick Start

```bash
# Install dependencies (uses pnpm by default, falls back to npm)
make install

# Start development mode with hot reload
make dev
# or directly:
pnpm run tauri dev
```

### Code Quality

```bash
# Run all checks before committing (format, lint, types, tests, markdown)
make check

# Individual checks:
pnpm run lint          # ESLint for TypeScript/React
pnpm run lint:fix      # Auto-fix linting issues
pnpm run lint:md       # Markdown linting
pnpm run lint:md:fix   # Auto-fix markdown issues
pnpm run format        # Format with Prettier
pnpm run format:check  # Check Prettier formatting
pnpm run typecheck     # TypeScript type checking
pnpm run test          # Run all tests (Rust + Frontend)
pnpm run test:rust     # Rust tests only
pnpm run test:frontend # Frontend tests only

# Rust-specific:
cargo fmt              # Format Rust code
cargo clippy           # Rust linting
cargo test             # Run Rust tests
cargo test <test_name> # Run specific Rust test
```

### Build & Release

```bash
# Build production app
pnpm run tauri build

# Version management workflow:
make patch    # Create patch dev version (0.1.0 -> 0.1.1-0)
make minor    # Create minor dev version (0.1.0 -> 0.2.0-0)
make major    # Create major dev version (0.1.0 -> 1.0.0-0)
make release  # Release version, create tag, and push
make retag    # Re-push current tag (re-trigger CI/CD)
```

## Architecture

### Backend (Rust/Tauri)

The Rust backend (`src-tauri/src/`) follows a modular architecture:

- **Core Modules**:
  - `lib.rs`: Central hub defining all Tauri commands, `AppState` structure, and application lifecycle
  - `main.rs`: Application entry point and window setup
  - `models.rs`: Data structures (Wallpaper, Settings, LocalWallpaper, etc.)
  - `runtime_state.rs`: Application runtime state management with watch channels for reactive updates

- **Manager Modules**:
  - `wallpaper_manager.rs`: Desktop wallpaper setting logic, platform-specific implementations
  - `download_manager.rs`: Concurrent image downloads with controlled parallelism (max 3 parallel)
  - `storage.rs`: File storage and wallpaper metadata persistence using MessagePack serialization
  - `settings_store.rs`: User settings persistence and validation using tauri-plugin-store
  - `index_manager.rs`: Wallpaper indexing and retrieval with smart caching

- **API Integration**:
  - `bing_api.rs`: Bing wallpaper API client with retry mechanism and exponential backoff

- **Platform-Specific**:
  - `macos_app.rs`: macOS-specific functionality using objc2 bindings for multi-monitor support

**Key Patterns**:

- All Tauri commands are async and return `Result<T, String>` for error handling
- Shared state via `Arc<Mutex<T>>` in `AppState` structure
- Event emission for frontend updates (e.g., `wallpaper-updated`, `settings-changed`)
- Watch channels (`tokio::sync::watch`) for reactive settings propagation
- Background tasks spawn with `tauri::async_runtime::spawn`
- Path validation to prevent setting arbitrary system files as wallpapers

### Frontend (React/TypeScript)

The React frontend (`src/`) uses functional components and hooks:

- **Core Components**:
  - `App.tsx`: Main application container
  - `components/WallpaperGrid.tsx`: Wallpaper gallery display
  - `components/WallpaperCard.tsx`: Individual wallpaper cards

- **State Management**:
  - `hooks/useBingWallpapers.ts`: Wallpaper fetching and state
  - `hooks/useSettings.ts`: Settings management
  - `contexts/ThemeContext.tsx`: Global theme state

- **Backend Communication**:
  - Uses Tauri's `invoke` API for command execution
  - Event listeners for real-time updates

### Data Flow

1. **Frontend → Backend**: Via Tauri commands using `invoke()` API
2. **Backend → Frontend**: Via events (`emit()`) and command return values
3. **State Updates**: Backend emits events (e.g., `wallpaper-updated`), frontend listens and updates UI reactively
4. **Settings Propagation**: Settings changes broadcast via `watch` channels to all background tasks
5. **Concurrent Operations**: Download manager handles parallel downloads (max 3 concurrent)
6. **Persistence**:
   - Settings: JSON via `tauri-plugin-store`
   - Wallpaper metadata: MessagePack binary format for efficiency
   - Images: Downloaded to configured directory (default: `$HOME/Pictures/Bing Wallpaper Now`)

### Background Update System

The app implements a smart background update cycle:

- Auto-update task spawns on app start if enabled in settings
- Checks Bing API based on last update time (stored in `AppState`)
- Downloads new wallpapers automatically without blocking UI
- Applies latest wallpaper if auto-update is enabled
- Respects retention count setting for automatic cleanup
- All update operations are cancellable via `AbortHandle`

## Platform-Specific Considerations

### macOS

- **Multi-monitor support**: Sets wallpaper across all connected displays simultaneously via `NSWorkspace`
- **Native APIs**: Uses `objc2`, `objc2-foundation`, and `objc2-app-kit` crates for Objective-C bindings
- **Fullscreen handling**: Detects and handles fullscreen apps to ensure wallpaper applies correctly
- **Spaces support**: Automatically restores wallpaper when switching Spaces or exiting fullscreen
- **Implementation**: See `macos_app.rs` for platform-specific code

### Windows

- Direct Windows API integration for wallpaper setting via `wallpaper` crate
- MSI installer configuration in `tauri.conf.json`
- Portable `.exe` also available

### Linux

- Desktop environment detection for wallpaper setting
- Supports GNOME, KDE, XFCE, and other common DEs
- Multiple package formats: `.deb` (Debian/Ubuntu), `.rpm` (Fedora/RedHat), `.AppImage` (Universal)

## Testing Strategy

- **Frontend Tests**: Located alongside components (`.test.tsx` files)
  - Uses Vitest and React Testing Library
  - Focus on component behavior and user interactions

- **Rust Tests**: In-module tests in Rust files
  - Run with `cargo test`
  - Cover core logic and error handling

## Key Configuration Files

- `src-tauri/tauri.conf.json`: Tauri app configuration, build settings
- `package.json`: Frontend dependencies and scripts
- `src-tauri/Cargo.toml`: Rust dependencies
- `Makefile`: Development workflow automation
- `.github/workflows/`: CI/CD pipelines for building and releasing

## Common Development Tasks

### Adding a New Tauri Command

1. Define the async function in `src-tauri/src/lib.rs` with the `#[tauri::command]` macro:

```rust
#[tauri::command]
async fn your_command(
    arg: String,
    state: tauri::State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<ReturnType, String> {
    // Implementation
    // Access shared state: state.settings.lock().await
    // Emit events: app.emit("event-name", payload)?;
    Ok(result)
}
```

2. Register the command in the `.invoke_handler()` call in `lib.rs`

3. Call from frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke<ReturnType>('your_command', { arg: 'value' });
```

### Modifying Storage System

The storage system uses MessagePack for efficient binary serialization:

- **Wallpaper metadata**: `storage.rs` handles read/write operations
- **Data structures**: Modify `LocalWallpaper` in `models.rs` (must be Serde-compatible)
- **Settings**: Stored separately via `tauri-plugin-store` in JSON format
- **File operations**: All file paths are validated to prevent security issues

### Working with Background Tasks

Background tasks use Tokio's async runtime:

- **Spawn tasks**: Use `tauri::async_runtime::spawn()` for background work
- **Settings reactivity**: Listen to `settings_rx` watch channel for settings changes
- **Cancellation**: Store `JoinHandle` in `AppState` to abort tasks when needed
- **Update cycle**: See `run_update_cycle()` and `force_update()` in `lib.rs`

### Debugging

```bash
# Run with Rust logging enabled
RUST_LOG=debug pnpm run tauri dev

# Check specific modules
RUST_LOG=bing_wallpaper_now_lib::wallpaper_manager=debug pnpm run tauri dev

# Run tests with output
cargo test -- --nocapture
```

## Performance Considerations

- **Frontend**: Uses `react-window` for virtualized lists to handle large wallpaper collections
- **Downloads**: Max 3 concurrent downloads to balance speed and resource usage
- **Caching**: Local cache prioritized over remote API calls
- **Lazy loading**: Wallpaper metadata loaded on-demand
- **MessagePack**: Binary serialization faster than JSON for metadata storage
- **Event-driven**: Reactive updates avoid polling and unnecessary re-renders

## Release Workflow

The project uses automated version management and CI/CD:

1. **Development cycle**: After releasing a version (e.g., `0.1.0`), create a development version:
   ```bash
   make patch    # Creates 0.1.1-0 for patch development
   make minor    # Creates 0.2.0-0 for minor features
   make major    # Creates 1.0.0-0 for breaking changes
   ```

2. **Release process**: When ready to release:
   ```bash
   make release  # Runs checks, updates version, creates tag, pushes
   ```
   This triggers GitHub Actions to build and publish installers for all platforms.

3. **Re-trigger CI**: If a build fails or you need to rebuild:
   ```bash
   make retag    # Re-pushes the current version tag
   ```

**Important**: The release script automatically:
- Validates working directory is clean
- Runs all quality checks (`make check`)
- Updates version in `package.json`, `Cargo.toml`, and `tauri.conf.json`
- Creates a git commit and tag
- Pushes to remote repository
