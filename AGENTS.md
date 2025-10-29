# Repository Guidelines

## Project Structure & Module Organization

Primary UI code lives under `src/`, with React components in `src/components`, cross-cutting state in `src/contexts`, hooks in `src/hooks`, and shared types/utilities in `src/types` and `src/utils`. Frontend tests sit alongside components (e.g., `src/App.test.tsx`) and helpers under `src/test`. The Tauri backend and wallpaper automation are in `src-tauri/src`, backed by `tauri.conf.json` for distribution settings. Static assets reside in `public/`, while reusable automation scripts live in `scripts/`, including versioning and quality gates.

## Build, Test, and Development Commands

Run `pnpm install` (or `npm install`) once to sync dependencies. Use `pnpm tauri dev` or `make dev` for the full desktop experience with hot reload; `pnpm dev` spins up the web UI only. Build production artifacts with `pnpm build`, and verify locally with `pnpm preview`. Quality checks are aggregated in `make check`, which wraps formatting, linting, type-checking, and both test suites.

## Coding Style & Naming Conventions

Code is TypeScript-first with React 19. Prettier enforces two-space indentation, trailing commas, and semicolons; run `pnpm format` or `pnpm format:check` before committing. ESLint (`eslint.config.js`) covers React hooks rules, import ordering, and TypeScript linting—address warnings via `pnpm lint` or `pnpm lint:fix`. Follow PascalCase for components, `use`-prefixed camelCase for hooks, and colocate component styles in CSS modules where applicable.

## Testing Guidelines

Frontend tests use Vitest plus Testing Library; keep specs in `*.test.tsx` files near the code or under `src/test`. Run `pnpm test:frontend` for UI logic, `pnpm test:rust` for backend commands, or `pnpm test` for both suites. Generate coverage via `pnpm test:frontend -- --coverage`, which writes to `coverage-frontend/`; maintain meaningful assertions instead of snapshot-only coverage.

## Commit & Pull Request Guidelines

The project uses Conventional Commits (`type: short imperative`, e.g., `fix: handle null wallpaper url`). Scope commits tightly and update documentation or configuration when behavior changes. Before opening a PR, run `make check` and include output for any failing step. Reference related issues, attach screenshots for UI-facing tweaks, and outline testing performed. For release work, coordinate with maintainers and reuse `scripts/manage-version.sh` to bump versions consistently.

## Environment & Configuration Tips

Target Node.js 22 LTS, Rust 1.80+, and pnpm 10 to match CI expectations. Update `src-tauri/tauri.conf.json` when packaging changes are needed; keep icons under `src-tauri/icons`. Reuse helpers in `scripts/lib/` instead of duplicating shell logic, and avoid editing generated files under `src-tauri/gen` or `target/`.
