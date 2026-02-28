---
name: release
description: 执行标准发布流程并确保发布可追溯。用于版本发布、打 tag、更新变更记录、发布前质量门禁。
allowed-tools: Read Write Shell Grep Glob
disable-model-invocation: true
---

# 发布流程

执行发布时，先保证质量，再执行发布。不要跳过问题，不要绕过检查。

## When to Use

- 用户请求"release / 发版 / 打 tag / 发布版本"
- 需要执行正式发布前检查并确认发布范围
- 需要更新变更记录并走标准发布命令

## Invocation Mode

- 当前配置为 `disable-model-invocation: true`，仅在显式 `/release` 时生效
- 如果希望模型在相关上下文自动调用，可将其改为 `false` 或删除该字段

## 职责分工

AI 负责：

1. 确认发布范围（对比 tag 差异）
2. 更新 `CHANGELOG.md` 并提交

`make release` 脚本自动处理：

1. 质量检查（`make check`，strict 模式，不做 auto-fix）
2. 版本文件更新（`package.json`、`Cargo.toml`、`tauri.conf.json`、`Cargo.lock`）
3. 创建 release commit（`chore(release): X.Y.Z`）
4. 创建 Git tag
5. 推送到远端（触发 CI/CD）

> **不要**在 `make release` 之前单独运行 `make check` 或 `precheck.sh`，
> 因为脚本内部已包含质量检查，重复运行会浪费时间且可能因中间文件修改
> 导致检查结果不一致。

## Changelog 规范

1. 仅基于可验证来源写入
   - 仅允许使用"上一个 release tag 到 `HEAD`"范围内的提交/PR/代码变更作为来源
   - 禁止编造、猜测、夸大收益（例如性能提升百分比）或写入未验证结论
   - 无法归因到证据的内容，不得写入

2. 格式约定（与现有 CHANGELOG 保持一致）
   - 版本标题使用 `## X.Y.Z`（不加方括号，不加日期）
   - 分类使用三级标题：`### Added`、`### Changed`、`### Fixed`、`### Testing`、`### Docs` 等
   - 空分类不要保留
   - 版本校验脚本依赖 `^## X.Y.Z` 正则匹配，**不要使用方括号或附加日期**

3. 语言与粒度
   - 以用户价值为主，避免纯内部实现细节堆砌
   - 每条尽量"一条可读变化"，禁止大段冗长描述
   - 可使用简短子项补充，但必须仍可追溯

## Instructions

1. 先澄清发布方式与边界（若不明确）
   - 确认是否发布正式版（`make release`）或仅准备发布内容
   - 确认目标版本来源（如已通过 `make patch/minor/major` 预先升级）
   - 确认是否需要创建 GitHub Release 说明

2. 确认发布范围与工作区状态
   - 查看当前版本（应为 `X.Y.Z-0` 开发版本）
   - 若当前版本不是开发版本，先执行 `make patch YES=1` 创建开发版本
   - 对比最近 tag 到 `HEAD` 的差异，确认本次发布内容边界
   - 确认工作区无未提交变更

3. 更新发布说明
   - 更新 `CHANGELOG.md`，直接写在目标版本标题下
   - 先整理证据清单（提交/PR），再写入条目
   - 不得把原始 commit log 原样粘贴为 changelog

4. **人工确认 CHANGELOG（必须）**
   - 将完整的 CHANGELOG 新增内容展示给用户
   - 明确告知用户："请确认 CHANGELOG 内容，确认无误后我再继续提交和发布"
   - **必须等待用户明确确认后才能进入下一步**
   - 若用户要求修改，按反馈修正后再次请求确认，直到用户满意

5. 提交 CHANGELOG
   - 仅提交 `CHANGELOG.md`
   - 提交信息：`chore: add CHANGELOG entry for X.Y.Z`

6. 执行发布命令
   - 运行 `make release`
   - 脚本会自动运行质量检查、更新版本、提交、打 tag、推送
   - 若失败，停止并说明失败原因，**不要手动补救**

7. 返回发布结果
   - 返回版本号、tag、关键提交与后续验证建议
   - 附上 CI 构建链接

## Output Format

按下面结构输出，先结果后风险：

```markdown
## Release Result
- 版本：...
- Tag：...
- 状态：成功/失败

## Scope
- 关键变更：...
- 包含提交：...

## Validation
- 已执行：...
- 待验证：...

## Risks
- 发现风险与建议：...
```

## Retag（重新触发构建）

当 CI/CD 构建需要重新触发时（例如构建失败但代码无需更改），可使用 `make retag`：

- 仅适用于已发布的正式版本（非开发版本）
- 会强制推送当前版本的 tag 到远端
- 触发 GitHub Actions 重新构建

## 执行原则

- 质量检查由 `make release` 保证，不额外运行
- 遇到不确定或高风险操作，先停下并询问用户
- 不做破坏性 Git 操作
- 仅基于可验证结果汇报发布状态
- 若 changelog 内容缺少证据，宁可少写，不可猜写
