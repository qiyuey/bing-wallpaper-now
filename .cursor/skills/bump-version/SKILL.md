---
name: bump-version
description: 升级项目版本号并更新前后端供应链依赖。Use when the user asks to bump version, upgrade version, 升版本, 提升版本号, or says "/bump".
---

# 升级版本号

升级项目开发版本号（patch/minor/major），并更新前后端依赖到最新兼容版本。

## When to Use

- 用户请求"升版本 / bump version / 提升版本号 / 升级版本"
- 发布完成后需要创建下一个开发版本并同步依赖

## Instructions

### Step 1: 分析变更并以选项方式询问升级类型

先快速浏览自上次 release tag 以来的提交/变更，按以下规则判断推荐级别：

| 级别  | 适用场景                                   |
| ----- | ------------------------------------------ |
| patch | Bug 修复、小功能调整、文案/样式优化        |
| minor | 中等功能新增或改进，用户可明显感知         |
| major | 大功能或体验重大变化，值得单独宣传         |

> 本项目始终向后兼容，版本号不以"是否有破坏性变更"区分，而是以**功能规模**区分。

**必须**以选项形式呈现，让用户通过点击选项回答，并在推荐项后标注"（推荐）"。例如：

- **patch**（推荐）— 补丁版本 (X.Y.Z → X.Y.Z+1-0)
- **minor** — 次版本 (X.Y.Z → X.Y+1.0-0)
- **major** — 主版本 (X.Y.Z → X+1.0.0-0)

同时用一句话简要说明推荐理由（如"本次变更主要为 bug 修复，建议 patch"）。

根据用户选中的选项执行 Step 2；若用户未选或环境无法点击，可提示「请回复 patch / minor / major 或 1 / 2 / 3」作为兜底。

### Step 2: 升级版本号

根据用户选择执行对应命令（`YES=1` 跳过交互确认）：

```bash
make patch YES=1   # 或 minor / major
```

- 检查输出确认版本已更新
- 若失败则停止并报告原因

### Step 3: 更新前端依赖

```bash
pnpm update
```

### Step 4: 更新 Rust 依赖

```bash
cd src-tauri && cargo update
```

### Step 5: 汇总结果

输出格式：

```markdown
## Version Bump Summary
- **版本**: 旧版本 → 新版本
- **pnpm**: X 个包更新 / 无变更
- **Cargo**: Y 个 crate 更新 / 无变更
- **状态**: 依赖变更文件已修改，未提交
```

### Step 6: 以选项方式询问是否提交

**必须**以选项形式呈现，让用户通过点击选项回答：

- **提交并推送** — 立即执行 commit 与 push
- **暂不提交** — 仅告知变更已修改、未提交，由用户自行处理

说明文案示例：「是否现在提交并推送？请点击上方选项之一。」

- **用户选择「提交并推送」**：读取 `~/.cursor/skills/commit/SKILL.md`，若文件存在则按其流程执行；若文件不存在，则执行以下回退流程：
  1. `git add -A`
  2. 根据 diff 生成 Conventional Commits 格式的 commit message
  3. `git commit -m "<message>"`
  4. `git push`（无 upstream 时用 `git push -u origin HEAD`）
- **用户选择「暂不提交」**：告知变更文件已修改但未提交，由用户自行处理
