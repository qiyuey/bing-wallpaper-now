# Release Process

Follow these steps to release a new version:

1. **Run quality checks**: `make check`
   - Fix any issues found and retry until all checks pass

2. **Find previous release tag**: `git describe --tags --abbrev=0`
   - Save the tag name for step 3

3. **Review changes**: `git diff <previous-tag>..HEAD`
   - Replace `<previous-tag>` with the tag from step 2
   - Review all changes since last release

4. **Update CHANGELOG.md**:
   - Add a new section: `## x.y.z` (use the version number from package.json)
   - Write user-facing Chinese content describing the changes
   - Follow the format of previous entries (Added, Changed, Fixed, etc.)

5. **Commit CHANGELOG**: `git add CHANGELOG.md && git commit -m "docs: update changelog for x.y.z"`
   - Do NOT push yet

6. **Release**: `make release`
   - This will validate, update version, create tag, and push to remote
   - Fix any issues and retry as needed
   - CI/CD will automatically build and publish