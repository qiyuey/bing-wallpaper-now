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

### Step 1: 询问升级类型

使用 AskQuestion 工具询问用户：

```
问题: "选择版本升级类型"
选项:
  - patch: 补丁版本 (X.Y.Z → X.Y.Z+1-0)
  - minor: 次版本 (X.Y.Z → X.Y+1.0-0)
  - major: 主版本 (X.Y.Z → X+1.0.0-0)
```

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

### Step 6: 询问是否提交

使用 AskQuestion 询问用户是否立即提交并推送变更。

- **用户确认提交**：读取 `~/.cursor/skills/commit/SKILL.md`，若文件存在则按其流程执行；若文件不存在，则执行以下回退流程：
  1. `git add -A`
  2. 根据 diff 生成 Conventional Commits 格式的 commit message
  3. `git commit -m "<message>"`
  4. `git push`（无 upstream 时用 `git push -u origin HEAD`）
- **用户拒绝**：告知变更文件已修改但未提交，由用户自行处理
