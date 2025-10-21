# 版本管理指南

本文档说明如何管理 Bing Wallpaper Now 的版本号和发布流程。

## 🎯 快速开始

### 使用 Makefile（推荐）

```bash
# 补丁版本升级 (0.1.0 -> 0.1.1) - 用于 Bug 修复
make version-patch

# 次版本升级 (0.1.0 -> 0.2.0) - 用于新功能
make version-minor

# 主版本升级 (0.1.0 -> 1.0.0) - 用于重大更改
make version-major
```

### 使用脚本

```bash
# 升级补丁版本
./scripts/version.sh patch

# 升级次版本
./scripts/version.sh minor

# 升级主版本
./scripts/version.sh major

# 设置指定版本
./scripts/version.sh 1.2.3
```

---

## 📋 版本号规范（语义化版本）

遵循 [Semantic Versioning 2.0.0](https://semver.org/lang/zh-CN/)：

```
主版本号.次版本号.补丁版本号
MAJOR.MINOR.PATCH
```

### 何时升级版本号

| 版本类型  | 何时使用         | 示例变更         | 命令                 |
| --------- | ---------------- | ---------------- | -------------------- |
| **PATCH** | Bug 修复，小改进 | 修复壁纸设置失败 | `make version-patch` |
| **MINOR** | 新功能，向后兼容 | 添加多显示器支持 | `make version-minor` |
| **MAJOR** | 破坏性更改       | 重构存储结构     | `make version-major` |

---

## 🔄 自动化流程

`version.sh` 脚本会自动完成以下操作：

### 1. 检查环境

- ✅ 验证 Git 仓库状态
- ⚠️ 检查是否有未提交的更改（可继续）

### 2. 更新版本号

自动更新以下三个文件：

| 文件                        | 说明           |
| --------------------------- | -------------- |
| `package.json`              | Node.js 包配置 |
| `src-tauri/Cargo.toml`      | Rust 包配置    |
| `src-tauri/tauri.conf.json` | Tauri 应用配置 |

### 3. 更新 Cargo.lock

```bash
cargo update -p bing-wallpaper-now
```

### 4. Git 提交

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock
git commit -m "chore(release): x.y.z"
```

### 5. 创建 Git 标签

```bash
git tag -a vx.y.z -m "Release x.y.z"
```

### 6. 推送（可选）

脚本会询问是否立即推送：

```bash
git push origin main
git push origin vx.y.z
```

---

## 🎬 完整示例

### 场景 1: 发布补丁版本修复 Bug

```bash
# 1. 修复 Bug
vim src-tauri/src/wallpaper_manager.rs
git add -A
git commit -m "fix: resolve wallpaper setting issue"

# 2. 升级版本（0.1.0 -> 0.1.1）
make version-patch

# 输出：
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#   Tauri 版本更新
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
#
# ℹ 当前版本: 0.1.0
# ℹ 新版本:   0.1.1
#
# 确认更新版本？(y/N) y
#
# ✓ package.json 已更新为 0.1.1
# ✓ src-tauri/Cargo.toml 已更新为 0.1.1
# ✓ src-tauri/tauri.conf.json 已更新为 0.1.1
# ✓ Cargo.lock 已更新
# ✓ 已创建提交
# ✓ 已创建标签 v0.1.1
#
# ✓ 版本已更新为 0.1.1
#
# ℹ 下一步：
#   git push origin main
#   git push origin v0.1.1
#
# 是否立即推送到远程？(y/N) y
#
# ✓ 已推送到远程
# ✓ 完成！
```

### 场景 2: 发布新功能

```bash
# 1. 开发新功能
git checkout -b feature/multi-monitor
# ... 开发 ...
git commit -m "feat: add multi-monitor support"
git checkout main
git merge feature/multi-monitor

# 2. 升级次版本（0.1.1 -> 0.2.0）
make version-minor

# 3. 推送
# （脚本会询问）
```

### 场景 3: 手动设置版本号

```bash
# 直接设置为 1.0.0
./scripts/version.sh 1.0.0
```

---

## 📦 发布 Checklist

在发布新版本前，确保完成以下步骤：

### 1. 代码质量检查

```bash
make check-ci
```

### 2. 运行完整测试

```bash
make test
```

### 3. 构建测试

```bash
make bundle
```

### 4. 更新 CHANGELOG（可选）

```bash
vim CHANGELOG.md
git add CHANGELOG.md
git commit -m "docs: update changelog for vx.y.z"
```

### 5. 升级版本

```bash
make version-patch  # 或 minor / major
```

### 6. 验证发布

- 检查 GitHub Releases
- 验证标签已推送
- 测试安装包

---

## 🛠️ 高级用法

### 仅更新版本号，不提交

编辑脚本，注释掉 `git_commit_and_tag` 函数调用：

```bash
# 手动运行各个步骤
./scripts/version.sh patch
# 手动审查更改
git diff
# 手动提交
git add -A
git commit -m "chore(release): x.y.z"
git tag -a vx.y.z -m "Release x.y.z"
```

### 批量更新多个版本文件

如果有其他文件也需要更新版本号，编辑 `scripts/version.sh`：

```bash
# 添加自定义更新函数
update_custom_file() {
    local new_version=$1
    # 你的更新逻辑
}

# 在 main 函数中调用
update_custom_file "$new_version"
```

### 预发布版本

```bash
# 手动设置预发布版本
./scripts/version.sh 0.2.0-beta.1
./scripts/version.sh 1.0.0-rc.1
```

---

## 🔍 故障排除

### 问题 1: 工作目录有未提交的更改

**错误信息：**

```
⚠ 工作目录有未提交的更改
是否继续？(y/N)
```

**解决方案：**

```bash
# 选项 1: 提交更改
git add -A
git commit -m "your message"

# 选项 2: 暂存更改
git stash

# 选项 3: 继续（输入 y）
```

### 问题 2: sed 命令在不同平台行为不同

脚本已处理 macOS 和 Linux 的差异。

如果仍有问题，安装 `jq`：

```bash
# macOS
brew install jq

# Linux
sudo apt-get install jq
```

### 问题 3: 版本号更新不一致

**症状：** 三个文件的版本号不同步

**解决方案：**

```bash
# 手动同步版本号
vim package.json
vim src-tauri/Cargo.toml
vim src-tauri/tauri.conf.json

# 重新运行脚本
make version-patch
```

### 问题 4: 标签已存在

**错误信息：**

```
fatal: tag 'v0.1.1' already exists
```

**解决方案：**

```bash
# 删除本地标签
git tag -d v0.1.1

# 删除远程标签（谨慎！）
git push origin :refs/tags/v0.1.1

# 重新创建
make version-patch
```

---

## 📚 相关资源

- [语义化版本规范](https://semver.org/lang/zh-CN/)
- [Conventional Commits](https://www.conventionalcommits.org/zh-hans/)
- [Tauri 版本管理](https://tauri.app/v1/guides/building/)
- [Cargo 版本管理](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)

---

## 🤝 提交规范

推荐使用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/) 规范：

| 类型               | 说明       | 版本影响 |
| ------------------ | ---------- | -------- |
| `feat:`            | 新功能     | MINOR    |
| `fix:`             | Bug 修复   | PATCH    |
| `docs:`            | 文档更新   | 无       |
| `style:`           | 代码格式   | 无       |
| `refactor:`        | 重构       | 无/MINOR |
| `perf:`            | 性能优化   | PATCH    |
| `test:`            | 测试相关   | 无       |
| `chore:`           | 构建/工具  | 无       |
| `BREAKING CHANGE:` | 破坏性更改 | MAJOR    |

### 示例

```bash
# 新功能 -> 升级 MINOR
git commit -m "feat: add dark mode support"
make version-minor

# Bug 修复 -> 升级 PATCH
git commit -m "fix: resolve memory leak"
make version-patch

# 破坏性更改 -> 升级 MAJOR
git commit -m "feat!: change API structure

BREAKING CHANGE: API endpoints have been restructured"
make version-major
```

---

## 🚀 CI/CD 集成

### GitHub Actions 自动发布

创建 `.github/workflows/release.yml`：

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build Release
        run: |
          make bundle
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/**/*
```

### 自动化版本号管理

使用工具如 `standard-version` 或 `semantic-release`（可选）：

```bash
# 安装
pnpm add -D standard-version

# package.json
{
  "scripts": {
    "release": "standard-version"
  }
}

# 运行
pnpm run release
```

---

## 💡 最佳实践

1. **提交前运行测试**

   ```bash
   make check-ci && make test
   ```

2. **使用语义化版本**
   - 遵循 SemVer 规范
   - 保持版本号有意义

3. **编写 CHANGELOG**
   - 记录每个版本的变更
   - 方便用户了解更新内容

4. **标签命名一致**
   - 使用 `v` 前缀：`v1.0.0`
   - 保持格式统一

5. **定期发布**
   - 不要积累太多更改
   - 保持发布节奏

6. **备份重要标签**
   ```bash
   git tag -l | xargs -I {} git tag -v {} 2>/dev/null
   ```

---

## ❓ FAQ

**Q: 如何撤销版本升级？**

A: 如果还没推送：

```bash
git reset --hard HEAD~1
git tag -d vx.y.z
```

**Q: 如何查看所有版本标签？**

A:

```bash
git tag -l
# 或
git tag -l "v*" --sort=-v:refname
```

**Q: 版本号从哪里读取？**

A: 优先从 `package.json` 读取当前版本。

**Q: 支持预发布版本吗？**

A: 支持，使用完整版本号：

```bash
./scripts/version.sh 1.0.0-beta.1
```

**Q: 如何与 npm version 集成？**

A: 可以在 `package.json` 添加钩子：

```json
{
  "scripts": {
    "version": "./scripts/version.sh $npm_package_version"
  }
}
```

---

祝发布愉快！🎉
