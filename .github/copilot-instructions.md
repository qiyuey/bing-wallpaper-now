# Copilot Instructions

- **Workflow – Check**: `make check` is mandatory before commits/releases. Fix all issues, rerun until clean (mirrors `.vscode/copilot-instructions.md`).
- **Workflow – Commit**: Run `make check`, then commit using Conventional Commits (`feat:detail`, `fix:detail`, etc.) and push (`git push`).
- **Workflow – Release**: Run `make check`, diff from previous tag via `git describe --tags --abbrev=0`, update `CHANGELOG.md` with `## x.y.z`, commit (no push), then `make release`; fix issues and retry as needed.
