# AGENTS.md

> Repository guidelines for AI coding agents working on Bing Wallpaper Now

## Project Overview

Bing Wallpaper Now is a cross-platform desktop application that automatically fetches and sets Bing daily wallpapers. Built with Tauri 2.0, it combines a React/TypeScript frontend with a Rust backend.

**Tech Stack:**
- Frontend: React 19, TypeScript, Vite
- Backend: Rust (Edition 2024), Tauri 2.0
- Testing: Vitest (frontend), Cargo test (backend)
- Package Manager: pnpm (version 10.19.0)

## Setup & Development

### Prerequisites

- **Node.js**: 24+ (LTS)
- **Rust**: 1.80+ (Edition 2024)
- **pnpm**: 10.19.0 (specified in `packageManager` field)

### Installation

```bash
# Install dependencies
pnpm install

# Start development mode (hot reload)
pnpm run tauri dev
# OR
make dev
```

### Common Commands

```bash
# Development
pnpm run dev                # Vite dev server only
pnpm run tauri dev          # Full Tauri app with hot reload

# Building
pnpm run build              # Build frontend (TypeScript compile + Vite build)
pnpm run tauri build        # Build production app for current platform

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
bash scripts/check-quality.sh  # Same as above

# Version management
make patch                  # Bump patch version (0.3.5 -> 0.3.6-0)
make minor                  # Bump minor version (0.3.5 -> 0.4.0-0)
make major                  # Bump major version (0.3.5 -> 1.0.0-0)
make release                # Release current dev version and tag
```

## Project Structure

```
bing-wallpaper-now/
├── src/                          # Frontend (React + TypeScript)
│   ├── components/               # React components
│   │   ├── App.tsx              # Main app component
│   │   ├── Settings.tsx         # Settings dialog
│   │   └── WallpaperCard.tsx    # Wallpaper display card
│   ├── hooks/                   # Custom React hooks
│   │   ├── useBingWallpapers.ts # Wallpaper data fetching
│   │   ├── useSettings.ts       # Settings management
│   │   └── useTray.ts           # System tray integration
│   ├── types/                   # TypeScript type definitions
│   └── main.tsx                 # App entry point
├── src-tauri/                   # Backend (Rust + Tauri)
│   ├── src/
│   │   ├── bing_api.rs         # Bing API integration
│   │   ├── wallpaper_manager.rs # Wallpaper setting logic
│   │   ├── download_manager.rs  # Image download & caching
│   │   ├── storage.rs          # File storage management
│   │   └── lib.rs              # Main Rust entry point
│   ├── Cargo.toml              # Rust dependencies
│   └── tauri.conf.json         # Tauri configuration
├── scripts/                     # Build & utility scripts
│   ├── check-quality.sh        # Code quality checks
│   └── manage-version.sh       # Version management
├── Makefile                     # Development commands
└── package.json                 # Frontend dependencies & scripts
```

## Code Style & Conventions

### TypeScript/React

- **TypeScript**: Strict mode enabled
- **Quotes**: No specific preference configured (follows Prettier defaults)
- **Semicolons**: Follow Prettier defaults (present)
- **Naming**:
  - Components: PascalCase (`WallpaperCard.tsx`)
  - Hooks: camelCase with "use" prefix (`useBingWallpapers.ts`)
  - Files: Match component/hook name
- **React**:
  - Functional components only
  - Use hooks for state management
  - React 19+ (no need to import React in JSX files)
- **ESLint Rules**:
  - Unused vars: warn (except with `_` prefix)
  - `any` type: warn
  - Console: warn (allow `console.warn` and `console.error`)
  - React Hooks: enforce rules-of-hooks (error), exhaustive-deps (warn)

### Rust

- **Edition**: 2024
- **Formatting**: Use `cargo fmt --manifest-path src-tauri/Cargo.toml`
- **Linting**: Use `cargo clippy` for warnings
- **Naming**:
  - Files: snake_case (`bing_api.rs`)
  - Functions: snake_case
  - Types: PascalCase
  - Constants: SCREAMING_SNAKE_CASE
- **Error Handling**: Use `anyhow::Result` for most functions
- **Async**: Use Tokio runtime (features = ["full"])
- **Documentation**: Add doc comments (`///`) for all public functions

### File Organization

- Frontend source files go in `src/`
- Backend source files go in `src-tauri/src/`
- Tests colocated with source (`.test.ts`, `.test.tsx` for frontend)
- Type definitions in `src/types/`

## Testing

### Frontend (Vitest)

```bash
pnpm run test:frontend          # Run tests
pnpm run test:frontend -- --coverage  # With coverage report
```

- **Framework**: Vitest with jsdom
- **Location**: `src/**/*.{test,spec}.{ts,tsx}`
- **Setup**: `src/test/setup.ts`
- **Coverage thresholds**:
  - Lines: 70%
  - Functions: 40%
  - Branches: 60%
  - Statements: 70%
- **Test utilities**: `@testing-library/react`, `@testing-library/jest-dom`

### Backend (Rust)

```bash
cargo test --manifest-path src-tauri/Cargo.toml
# OR
pnpm run test:rust
```

- Tests are in the same files as implementation (`#[cfg(test)]` modules)
- Run with `-- --nocapture` to see println! output

## Tauri-Specific Notes

### Plugin Permissions

The app uses several Tauri plugins with specific permissions configured in `src-tauri/capabilities/default.json`:

- **opener**: `allow-open-path` for opening wallpaper folder
- **dialog**: `allow-message`, `allow-open`, `allow-save` for file dialogs
- **store**: `allow-get`, `allow-set` for settings persistence
- **autostart**: `allow-enable`, `allow-disable`, `allow-is-enabled`
- **notification**: `default` for notifications

When adding new plugin functionality, ensure proper permissions are configured.

### Commands

Rust functions exposed to frontend via `#[tauri::command]` macro:

- `get_wallpapers()` - Fetch wallpapers from Bing API
- `set_wallpaper(path: String)` - Set desktop wallpaper
- `download_wallpaper(url: String)` - Download wallpaper to disk
- `get_wallpaper_directory()` - Get current wallpaper save directory
- `open_wallpaper_folder()` - Open folder in system file manager
- See `src-tauri/src/lib.rs` for complete list

### Platform-Specific Code

- **macOS**: Uses objc2 bindings for native NSWorkspace, NSScreen APIs
- Handles multi-monitor wallpaper setting
- Supports Space switching and fullscreen app scenarios

## Build & Release Process

### Development Workflow

1. After a release, create a new development version:
   ```bash
   make patch  # Creates version like 0.3.6-0
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

- **Development**: `X.Y.Z-0` (e.g., `0.3.5-0`)
- **Release**: `X.Y.Z` (e.g., `0.3.5`)
- Version is synchronized across:
  - `package.json`
  - `src-tauri/Cargo.toml`
  - `src-tauri/Cargo.lock`

### CI/CD

- GitHub Actions automatically builds and publishes releases when tags are pushed
- Builds for Windows (.msi, .exe), macOS (.dmg), Linux (.deb, .rpm, .AppImage)

## Important Files

- **`package.json`**: Frontend dependencies, scripts, version
- **`src-tauri/Cargo.toml`**: Rust dependencies, version
- **`src-tauri/tauri.conf.json`**: Tauri app configuration
- **`eslint.config.js`**: ESLint flat config (modern format)
- **`vitest.config.ts`**: Vitest test configuration
- **`Makefile`**: Convenient command shortcuts
- **`scripts/check-quality.sh`**: Pre-commit quality checks

## Common Issues & Solutions

### Build Issues

**Issue**: Node.js version mismatch
- **Solution**: Ensure Node.js 24+ is installed. Use `node --version` to check.

**Issue**: Rust compilation errors
- **Solution**: Update Rust: `rustup update`. Ensure 1.80+ with edition 2024 support.

**Issue**: Tauri dev fails on macOS
- **Solution**: Install Xcode Command Line Tools: `xcode-select --install`

### Development Tips

1. **Hot Reload**: Use `pnpm run tauri dev` for full app hot reload including Rust changes
2. **Faster Frontend Iteration**: Use `pnpm run dev` for Vite-only mode when working on UI
3. **Type Safety**: Run `pnpm run typecheck` frequently to catch TypeScript errors early
4. **Pre-commit**: Always run `make check` before committing to catch issues
5. **Debugging Rust**: Use `log::debug!()` and enable Tauri logs in settings

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

## Security & Privacy

- No analytics or tracking
- No external servers except Bing API
- All wallpapers stored locally
- Settings stored locally via Tauri plugin-store
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
