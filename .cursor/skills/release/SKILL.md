---
name: release
description: 执行标准发布流程并确保发布可追溯。用于版本发布、打 tag、更新变更记录、发布前质量门禁。
allowed-tools: Read Write Shell Grep Glob AskQuestion
disable-model-invocation: true
---

# 发布流程

执行发布时，先保证质量，再执行发布。不要跳过问题，不要绕过检查。

## When to Use

- 用户请求"release / 发版 / 打 tag / 发布版本"

## 职责分工

AI 负责：确认发布范围、更新 `CHANGELOG.md`、按需更新 `AGENTS.md` / `README.md` / `README.zh.md`

`make release` 脚本自动处理：质量检查 → 版本文件更新 → release commit → Git tag → 推送

> **不要**在 `make release` 之前单独运行 `make check`，脚本内部已包含。

## Changelog 规范

- 仅基于"上一个 release tag → HEAD"范围内的提交/PR 写入，禁止编造或夸大
- 格式：`## X.Y.Z` 标题（无方括号、无日期），`### Added` / `### Changed` / `### Fixed` 等分类
- **只写用户可感知的变更**，省略内部重构、工具链、文档维护
- **用户视角描述**：写"修复了什么 / 现在可以做什么"，不写"重构了哪个模块"
- 每条一行，合并相关变更，措辞精炼直白
- 判断标准：普通用户听到这条能理解"对我有什么影响"吗？

## Instructions

1. **工作区预检**
   - 并行获取：当前版本、`git status`、最近 tag 到 HEAD 的提交列表
   - 未提交变更：`git add -A && git commit -m "chore: pre-release changes"` 并 `git push`
   - 非开发版本（无 `-0` 后缀）：分析提交内容，按推荐规则给出建议级别，以选项形式询问 patch/minor/major，运行 `make <level> YES=1`
   - 预检完成后，工作区应干净且版本为 `X.Y.Z-0`

2. **更新发布说明**
   - 先整理证据清单（提交/PR），再写入 `CHANGELOG.md`
   - 不得把原始 commit log 原样粘贴

3. **按需更新项目文档**
   - 仅当涉及新增/删除文件、功能变化、命令变化、配置变化时更新
   - `AGENTS.md`（中英混用）、`README.md`（英文）、`README.zh.md`（中文）
   - 不做无意义润色，只同步实际变更；无需更新则跳过

4. **确认发布内容（必须）**
   - 展示 CHANGELOG 新增内容及文档变更摘要
   - 以选项形式请求确认（确认发布 / 取消）
   - **必须等待用户确认后才能继续**

5. **提交发布文档**
   - 仅添加实际修改的文件，commit message：
     - 只改 CHANGELOG：`chore: add CHANGELOG entry for X.Y.Z`
     - 同时改文档：`docs: update docs and CHANGELOG for X.Y.Z`

6. **执行 `make release`**
   - 失败时：分析原因，若可修复则修复并提交，重试一次；否则停止并说明

7. **监控 CI 构建**
   - 先用 `gh run list` 获取 workflow run URL，以 **Markdown 超链接** 形式展示给用户（方便直接点击跳转）
   - 然后运行 `bash scripts/monitor-ci.sh <tag>`
   - 脚本会自动轮询并报告结果，构建完成时自动发送 macOS 系统通知
   - 若脚本报告失败：展示失败信息，询问用户是否排查修复
   - 修复后可 `make retag` 重新触发构建，再次运行监控脚本

## Retag

`make retag`：仅适用于已发布版本，强制推送 tag 到远端重新触发 CI。

## 版本升级推荐规则

本项目始终向后兼容，版本号不以"是否有破坏性变更"区分，而是以**功能规模**区分：

| 级别  | 适用场景                                   | 示例                             |
| ----- | ------------------------------------------ | -------------------------------- |
| patch | Bug 修复、小功能调整、文案/样式优化        | 修复托盘菜单显示异常、调整间距   |
| minor | 中等功能新增或改进，用户可明显感知         | 新增导出功能、支持新语言         |
| major | 大功能或体验重大变化，值得单独宣传         | 全新 UI 改版、新增多平台支持     |

在询问用户时，基于已收集的提交/变更内容，**明确标注推荐项**，例如：

- **patch**（推荐）— 补丁版本
- **minor** — 次版本
- **major** — 主版本

## 执行原则

- 质量检查由 `make release` 保证，不额外运行
- 遇到不确定或高风险操作，先停下并询问用户
- 不做破坏性 Git 操作
- 若 changelog 内容缺少证据，宁可少写，不可猜写
