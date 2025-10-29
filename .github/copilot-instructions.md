# Copilot Instructions

- **Check**: `make check` is mandatory before commits/releases. Fix all issues, rerun until clean (mirrors `.vscode/copilot-instructions.md`).
- **Commit**: Run `make check`, then commit using Conventional Commits (`feat:detail`, `fix:detail`, etc.) and push (`git push`).
- **Release**: Run `make check`, find the previous tag with `git describe --tags --abbrev=0`, inspect the diff (`git diff <previous-tag>..HEAD`, update `CHANGELOG.md` with `## x.y.z` & user-facing Chinese content, commit (no push), then `make release`; fix issues and retry as needed.
