# Code Quality Check

**CRITICAL: You MUST fix all issues found. Do NOT skip any fixes or report issues without fixing them.**

Run code quality checks and automatically fix ALL issues found. This command requires you to iteratively fix problems until ALL checks pass.

## Execution Requirements

⚠️ **MANDATORY**: You MUST:
1. Run `make check` to identify ALL issues
2. Fix ALL auto-fixable issues immediately (run fix commands)
3. Fix ALL manual issues (code changes)
4. Re-run `make check` after EVERY round of fixes
5. Continue iterating until ALL checks pass (exit code 0)
6. Report what was fixed in each iteration

❌ **FORBIDDEN**: You MUST NOT:
- Skip any fixable issues
- Report issues without fixing them
- Stop before all checks pass
- Skip re-running checks after fixes

## Process (Follow Strictly)

### Step 1: Initial Check
Run `make check` and capture the full output.

### Step 2: Fix Auto-Fixable Issues (MANDATORY)

**You MUST run these fix commands in order, and re-run until they report no changes:**

1. **Rust formatting**:
   ```bash
   cargo fmt --manifest-path src-tauri/Cargo.toml
   ```
   - Re-run until output shows "no changes" or no formatting errors

2. **ESLint**:
   ```bash
   pnpm run lint:fix
   ```
   - Re-run until output shows "no errors" or all auto-fixable issues resolved

3. **Prettier**:
   ```bash
   pnpm run format
   ```
   - Re-run until output shows "no changes" or all files formatted

4. **Markdown lint**:
   ```bash
   pnpm run lint:md:fix
   ```
   - Re-run until output shows "no errors" or all auto-fixable issues resolved

**After each fix command, verify it worked by checking if files changed or errors decreased.**

### Step 3: Fix Manual Issues (MANDATORY)

**You MUST fix ALL remaining issues found by running specific check commands:**

1. **Rust Clippy warnings**:
   ```bash
   cargo clippy --manifest-path src-tauri/Cargo.toml
   ```
   - Read ALL warnings
   - Fix ALL warnings in code
   - Do NOT skip any warnings

2. **Rust test failures**:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
   - Read ALL failing tests
   - Fix ALL test failures
   - Do NOT skip any tests

3. **TypeScript type errors**:
   ```bash
   pnpm run typecheck
   ```
   - Read ALL type errors
   - Fix ALL type errors
   - Do NOT skip any errors

4. **Frontend test failures**:
   ```bash
   pnpm run test:frontend
   ```
   - Read ALL failing tests
   - Fix ALL test failures
   - Do NOT skip any tests

### Step 4: Verify Fixes (MANDATORY)

**After fixing issues, you MUST re-run `make check`:**

```bash
make check
```

- If ANY check fails, go back to Step 2 or 3
- Continue iterating until exit code is 0 (all checks pass)

### Step 5: Final Verification (MANDATORY)

**Only when `make check` passes completely (exit code 0):**

1. Report what was fixed:
   - List all auto-fixable issues that were fixed
   - List all manual issues that were fixed
   - Note any issues that required multiple iterations

2. Confirm all checks pass:
   ```bash
   make check
   ```
   - Must show exit code 0
   - Must show all checks passing

## Iteration Rules

- **Minimum iterations**: You MUST run `make check` at least twice:
  1. Once before fixes (to identify issues)
  2. Once after fixes (to verify they're fixed)

- **Maximum iterations**: Continue until ALL checks pass. There is no maximum - fix until done.

- **Between iterations**: After each round of fixes, you MUST re-run `make check` before proceeding.

## Error Handling

- **If auto-fix commands fail**: Investigate why and fix the root cause
- **If manual fixes introduce new errors**: Fix those too
- **If fixes require multiple rounds**: That's normal - continue iterating

## Success Criteria

✅ **Command is successful ONLY when:**
- `make check` exits with code 0
- All 8 checks pass:
  - ✅ Rust format check
  - ✅ Rust Clippy check
  - ✅ Rust tests
  - ✅ TypeScript type check
  - ✅ ESLint check
  - ✅ Prettier check
  - ✅ Frontend tests
  - ✅ Markdown lint check

## Example Output Format

After completing fixes, report:

```
## Fix Summary

### Auto-Fixable Issues Fixed:
- [x] Rust formatting: Fixed 3 files
- [x] ESLint: Fixed 5 warnings
- [x] Prettier: Formatted 2 files
- [x] Markdown lint: Fixed 1 issue

### Manual Issues Fixed:
- [x] Rust Clippy: Fixed 2 warnings (unused imports)
- [x] TypeScript: Fixed 3 type errors (missing types)
- [x] Frontend tests: Fixed 1 failing test (mock update)

### Iterations:
- Iteration 1: Initial check found 11 issues
- Iteration 2: Fixed auto-fixable issues, 4 remaining
- Iteration 3: Fixed manual issues, all checks pass

### Final Status:
✅ All checks pass (exit code 0)
```

## Notes

- Auto-fix commands may need multiple runs (some fixes expose other issues)
- Type errors and test failures typically require code changes
- Do NOT stop until `make check` shows exit code 0
- Report ALL fixes made, even if they seem minor
