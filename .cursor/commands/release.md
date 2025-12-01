# 发布流程

**重要提示：执行此命令时，必须先修复所有问题和警告，并且不能是逃避问题，而是要真正解决，然后才能继续发布。在所有质量检查通过之前不要继续。**

按照以下步骤发布新版本：

1. **运行质量检查并修复所有问题**：
   - 运行 `make check` 执行所有质量检查（lint、格式检查、类型检查、Markdown lint 和测试）
   - **禁止使用 `#[allow()]` 属性**：
     - 在 Rust 代码中，严格禁止使用 `#[allow(...)]` 来抑制编译器警告或 clippy 警告
     - 如果遇到警告，必须通过修复代码来解决问题，而不是使用 `#[allow()]` 来逃避
     - 使用 `grep` 工具搜索代码库中的 `#[allow` 来检查是否存在违规使用
     - 如果发现任何 `#[allow()` 的使用，必须移除并修复根本问题
   - 如果检查失败，根据错误类型修复：
     - **Lint 错误**：运行 `pnpm run lint:fix` 自动修复，或使用 `search_replace`/`write` 手动修复
     - **类型错误**：运行 `pnpm run typecheck` 查看详细信息，使用 `search_replace`/`write` 修复
     - **格式问题**：运行 `pnpm run format` 自动修复
     - **Markdown lint 错误**：修复 Markdown 文件中的格式问题
     - **测试失败**：修复失败的测试用例
   - 使用 `read_lints` 工具检查所有 lint 错误
   - 重复运行 `make check` 并修复问题，直到所有检查完全通过
   - **在 `make check` 完全通过之前不要继续**

2. **查找上一个发布标签**：`git describe --tags --abbrev=0`
   - 保存标签名称供步骤 3 使用

3. **审查变更**：`git diff <previous-tag>..HEAD`
   - 将 `<previous-tag>` 替换为步骤 2 中的标签
   - 审查自上次发布以来的所有变更

4. **提交未提交的代码更改**：
   - 检查是否有未提交的更改：`git status`
   - 如果有未提交的更改：
     - 运行 `git add .` 添加所有更改
     - 根据变更内容自动生成 commit message（例如：`feat: add version check feature` 或 `fix: resolve update issue`）
     - 运行 `git commit -m "<generated-message>"` 提交代码更改
   - 如果没有未提交的更改，跳过此步骤

5. **更新 CHANGELOG.md**：
   - 添加新章节：`## x.y.z`（使用 package.json 中的版本号）
   - 编写面向用户的中文内容描述变更
   - 遵循先前条目的格式（Added、Changed、Fixed 等）
   - 避免对最终用户无意义的纯技术优化
   - 专注于用户可见的变更：新功能、bug 修复、改进和移除的功能

6. **提交 CHANGELOG**：
   - 运行 `git add CHANGELOG.md` 添加 CHANGELOG.md
   - 从 package.json 读取当前版本号（例如：`cat package.json | grep '"version"' | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'`）
   - 如果当前版本号不是 -0 的开发版，则运行 `make patch` 升级版本
   - 自动生成 commit message：`chore: release <version>`（例如：`chore: release 0.4.5`）
   - 运行 `git commit -m "chore: release <version>"` 提交 CHANGELOG

7. **发布**：运行 `make release`
   - 如果出现问题，中断执行，让用户处理
   - 需要严格运行命令，不要使用其他命令替代
