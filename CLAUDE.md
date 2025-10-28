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
# Run all checks before committing (format, lint, types, tests)
make check

# Individual checks:
pnpm run lint          # ESLint for TypeScript/React
pnpm run lint:fix      # Auto-fix linting issues
pnpm run format        # Format with Prettier
pnpm run typecheck     # TypeScript type checking
pnpm run test          # Run all tests (Rust + Frontend)
pnpm run test:rust     # Rust tests only
pnpm run test:frontend # Frontend tests only
```

### Build & Release

```bash
# Build production app
pnpm run tauri build

# Version management (using Makefile):
make snapshot-patch   # Create patch dev version (0.1.0 -> 0.1.1-0)
make snapshot-minor   # Create minor dev version (0.1.0 -> 0.2.0-0)
make snapshot-major   # Create major dev version (0.1.0 -> 1.0.0-0)
make release         # Release version, tag and push
```

## Architecture

### Backend (Rust/Tauri)

The Rust backend (`src-tauri/src/`) follows a modular architecture:

- **Core Modules**:
  - `lib.rs`: Central hub for Tauri commands and application setup
  - `models.rs`: Data structures (Wallpaper, Settings, etc.)
  - `runtime_state.rs`: Application runtime state management

- **Manager Modules**:
  - `wallpaper_manager.rs`: Desktop wallpaper setting logic, platform-specific implementations
  - `download_manager.rs`: Concurrent image downloads with controlled parallelism
  - `storage.rs`: File storage and wallpaper metadata persistence
  - `settings_store.rs`: User settings persistence and validation
  - `index_manager.rs`: Wallpaper indexing and retrieval

- **API Integration**:
  - `bing_api.rs`: Bing wallpaper API client with retry mechanism

**Key Patterns**:

- All Tauri commands are async and return `Result<T, String>`
- Shared state via `Arc<Mutex<T>>` in `AppState`
- Event emission for frontend updates
- Exponential backoff for network operations

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

1. **Frontend → Backend**: Via Tauri commands (`invoke`)
2. **Backend → Frontend**: Via events and command returns
3. **State Updates**: Backend emits events, frontend listens and updates UI
4. **Concurrent Operations**: Download manager handles parallel downloads
5. **Persistence**: Settings and wallpaper metadata stored locally

## Platform-Specific Considerations

### macOS

- Multi-monitor support implementation in `wallpaper_manager.rs`
- Uses `objc2` bindings for native macOS APIs
- Handles fullscreen apps and Spaces switching

### Windows

- Direct Windows API integration for wallpaper setting
- MSI installer configuration in `tauri.conf.json`

### Linux

- Uses desktop environment detection for wallpaper setting
- Supports GNOME, KDE, and other common DEs

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

1. Add the async function in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
async fn your_command(state: tauri::State<'_, AppState>) -> Result<ReturnType, String> {
    // Implementation
}
```

1. Register in the command handler list in `lib.rs`

1. Call from frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';
const result = await invoke('your_command', { args });
```

### Modifying Wallpaper Storage

The storage system uses MessagePack for efficient binary serialization. Key files:

- `storage.rs`: Core storage logic
- `models.rs`: `Wallpaper` struct definition
- Settings are stored separately via `tauri-plugin-store`

### Working with Background Updates

The app uses a smart update cycle:

- Checks for updates based on last update time
- Runs in background without blocking UI
- Configurable auto-update interval
- See `run_update_cycle` and `force_update` commands in `lib.rs`

## Performance Considerations

- Frontend uses `react-window` for virtualized lists
- Rust backend uses concurrent downloads (max 3 parallel)
- Local cache prioritized over remote fetching
- Lazy loading of wallpaper metadata
