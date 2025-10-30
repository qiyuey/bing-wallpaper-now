# Code Quality Check

Run code quality checks and automatically fix issues when possible.

## Process

1. **Run quality checks**: `make check`
   - This runs all 8 checks: Rust format, Clippy, Rust tests, TypeScript types, ESLint, Prettier, Frontend tests, Markdown lint

2. **If checks fail, analyze and fix**:

   **Auto-fixable issues** (run fixes and re-check):
   - **Rust formatting**: Run `cargo fmt --manifest-path src-tauri/Cargo.toml`
   - **ESLint**: Run `pnpm run lint:fix`
   - **Prettier**: Run `pnpm run format`
   - **Markdown lint**: Run `pnpm run lint:md:fix`

   **Manual fixes required** (provide guidance):
   - **Rust Clippy**: Review warnings and fix manually
     - Run `cargo clippy --manifest-path src-tauri/Cargo.toml` to see detailed messages
     - Fix issues indicated by Clippy
   - **Rust tests**: Fix failing tests
     - Run `cargo test --manifest-path src-tauri/Cargo.toml` to see failures
     - Fix test code or implementation
   - **TypeScript types**: Fix type errors
     - Run `pnpm run typecheck` to see detailed errors
     - Fix type issues in TypeScript files
   - **Frontend tests**: Fix failing tests
     - Run `pnpm run test:frontend` to see failures
     - Fix test code or implementation

3. **Re-run checks**: After fixes, run `make check` again

4. **Iterate**: Continue fixing until all checks pass

## Notes

- Auto-fix commands should be run repeatedly until they report no changes
- Some issues may require multiple rounds of fixes
- Test failures and type errors typically need manual code changes
- The script auto-fixes once, but you may need to fix multiple times

