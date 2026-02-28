# 依赖更新

事务性地拉取最新代码并更新前后端依赖。任何步骤失败立即停止，不做后续操作。

## When to Use

- 用户请求"更新依赖 / update deps / 拉取最新 / 同步上游"
- 开发新功能前需要同步最新状态

## Instructions

按顺序执行以下步骤，**任一步骤失败则停止并报告原因**。

### 前置检查

1. 确认工作区干净（`git status --short` 无输出）
   - 若有未提交变更，停止并提示用户先处理

### Step 1: 拉取最新代码

```bash
git pull --rebase
```

- 若 rebase 冲突，停止并提示用户手动解决
- 记录拉取的新提交数量

### Step 2: 更新前端依赖

```bash
pnpm update
```

### Step 3: 更新 Rust 依赖

```bash
cd src-tauri && cargo update
```

### Step 4: 总结更新信息

汇总输出：

```markdown
## Update Summary
- **Git**: 拉取了 N 个新提交 / 已是最新
- **pnpm**: X 个包更新 / 无变更（列出主要变更包）
- **Cargo**: Y 个 crate 更新 / 无变更（列出主要变更 crate）
```

不要自动提交变更，由用户自行决定何时提交。
