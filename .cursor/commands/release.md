# Release Process

**CRITICAL: When executing this command, you MUST fix all issues and warnings before proceeding with the release. Do not proceed until all quality checks pass.**

Follow these steps to release a new version:

1. **Run quality checks**: `make check`
   - This runs lint, format check, typecheck, and tests
   - **MUST fix all issues found and retry until all checks pass**
   - Do not proceed to next step if any check fails

2. **Fix all linting errors**:
   - Run `pnpm run lint` to check ESLint errors
   - Run `pnpm run lint:fix` to auto-fix ESLint issues if possible
   - Manually fix remaining ESLint errors using `search_replace` or `write` tools
   - Run `read_lints` tool to check all lint errors across the codebase
   - Continue fixing until `pnpm run lint` passes with no errors

3. **Fix all type errors**:
   - Run `pnpm run typecheck` to check TypeScript type errors
   - Fix all type errors using `search_replace` or `write` tools
   - Continue fixing until `pnpm run typecheck` passes with no errors

4. **Fix all formatting issues**:
   - Run `pnpm run format:check` to check formatting
   - Run `pnpm run format` to auto-fix formatting issues
   - Manually fix any remaining formatting issues
   - Continue fixing until `pnpm run format:check` passes

5. **Fix all Markdown linting issues**:
   - Run `pnpm run lint:md` to check Markdown files
   - Fix all Markdown linting errors
   - Continue fixing until `pnpm run lint:md` passes

6. **Ensure all tests pass**:
   - Run `pnpm test` to run all tests (Rust + frontend)
   - Fix any failing tests
   - Continue fixing until all tests pass

7. **Verify quality checks pass again**:
   - Run `make check` again to ensure everything passes
   - If any check fails, go back to the appropriate step above
   - Do not proceed until `make check` passes completely

8. **Find previous release tag**: `git describe --tags --abbrev=0`
   - Save the tag name for step 9

9. **Review changes**: `git diff <previous-tag>..HEAD`
   - Replace `<previous-tag>` with the tag from step 8
   - Review all changes since last release

10. **Update CHANGELOG.md**:
    - Add a new section: `## x.y.z` (use the version number from package.json)
    - Write user-facing Chinese content describing the changes
    - Follow the format of previous entries (Added, Changed, Fixed, etc.)
    - Avoid pure technical optimizations that are meaningless to end users
    - Focus on user-visible changes: new features, bug fixes, improvements, and removed features

11. **Commit CHANGELOG**: `git add CHANGELOG.md && git commit -m "docs: update changelog for x.y.z"`
    - Do NOT push yet

12. **Release**: `make release`
    - This will validate, update version, create tag, and push to remote
    - If any validation fails, fix the issues and retry
    - CI/CD will automatically build and publish after successful push

## Fix Workflow

When fixing issues found during quality checks:

1. **Identify the issue**: Use the error message to understand what needs fixing
2. **Locate the code**: Use `read_file` to read the file with the issue
3. **Fix the code**: Use `search_replace` or `write` tools to fix the issue
4. **Verify the fix**: Run the appropriate check command again
5. **Repeat**: Continue until all checks pass

## Priority Order for Fixes

1. **Critical issues** (must fix before release):
   - Panic risks (unwraps, indexing without bounds check, `block_on` in async)
   - Data loss potential
   - Security vulnerabilities
   - Breaking functionality
   - Memory leaks or resource leaks
   - Type errors that prevent compilation
   - Test failures

2. **High priority issues** (must fix before release):
   - Linting errors
   - Type safety issues (TypeScript `any` types)
   - Formatting inconsistencies
   - Markdown linting errors

3. **Medium priority issues** (should fix before release):
   - Code smells that affect maintainability
   - Performance issues

## Verification Checklist

Before proceeding to release, ensure:

- [ ] `make check` passes completely
- [ ] `pnpm run lint` passes with no errors
- [ ] `pnpm run typecheck` passes with no errors
- [ ] `pnpm run format:check` passes
- [ ] `pnpm run lint:md` passes
- [ ] `pnpm test` passes (all tests)
- [ ] No linting errors when running `read_lints` tool
- [ ] All Critical and High priority issues are fixed

**DO NOT proceed with release until all items above are checked.**
