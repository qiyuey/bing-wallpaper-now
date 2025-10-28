# Repository Guidelines

## Project Structure & Modules

- Frontend (React + Vite + TS): `src/` (components, hooks, contexts, utils, types). Tests colocated as `*.{test,spec}.{ts,tsx}`. Setup: `src/test/setup.ts`.
- Desktop app (Tauri + Rust): `src-tauri/` (`src/` for Rust crates, `tauri.conf.json`, `icons/`).
- Static assets: `public/`, `src/assets/`.
- Tooling/automation: `Makefile`, `scripts/`, configs (`*.config.*`, `vitest.config.ts`, `eslint.config.js`).

## Build, Test, and Development

- Install deps: `pnpm install` (or `npm install`).
- Web dev server: `pnpm dev`.
- Tauri app (hot reload): `pnpm tauri dev` or `make dev`.
- Build web: `pnpm build` (runs `tsc` then `vite build`).
- Tests: `pnpm test` (Rust + frontend), `pnpm test:frontend`, `pnpm test:rust`.
- Lint/format: `pnpm lint`, `pnpm lint:fix`, `pnpm format`, `pnpm format:check`, `pnpm lint:md`.
- Quality sweep: `make check` (format, lint, types, tests).

## Coding Style & Naming

- Formatting via Prettier; base rules in `.editorconfig` (2-space indent; Rust 4 spaces; LF; final newline).
- ESLint for TS/React (see `eslint.config.js`). Prefer:
  - Components: PascalCase files (e.g., `WallpaperCard.tsx`).
  - Hooks: `useX` naming (e.g., `useSettings.ts`).
  - Variables/functions: camelCase; types/interfaces: PascalCase.

## Testing Guidelines

- Framework: Vitest (+ jsdom, RTL). Config: `vitest.config.ts`.
- Test files: `src/**/*.{test,spec}.{ts,tsx}`; colocate near source.
- Coverage (soft thresholds): lines 70, funcs 40, branches 60, statements 70. Run `pnpm test` or `vitest run --coverage`.

## Commit & PR Guidelines

- Use Conventional Commits (seen in history): `feat:`, `fix:`, `docs:`, `refactor:`, `ci:`, `chore(release)`, `chore(version)`.
- PRs: include clear description, linked issues, and screenshots/GIFs for UI changes. Note platform (Windows/macOS/Linux) if relevant.
- Before opening PR: `make check` passes; update tests and relevant docs (`README.md`, `CHANGELOG.md`).

## Notes for Desktop (Tauri)

- Requires Rust 1.80+ and Node.js 22+. Use `pnpm tauri dev` to run the desktop app; `cargo test --manifest-path src-tauri/Cargo.toml` for Rust tests.
