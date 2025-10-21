# 跨平台编译检查脚本

本目录包含用于本地测试跨平台编译的脚本，避免每次都需要推送代码等待 GitHub Actions 失败。

## 📋 可用脚本

### 🏷️ 版本管理

**`version.sh`** - 自动版本管理（类似 npm version）

快速升级版本号并创建 Git 标签：

```bash
# 使用 Makefile（推荐）
make version-patch    # 0.1.0 -> 0.1.1
make version-minor    # 0.1.0 -> 0.2.0
make version-major    # 0.1.0 -> 1.0.0

# 或直接使用脚本
./scripts/version.sh patch
./scripts/version.sh minor
./scripts/version.sh major
./scripts/version.sh 1.2.3  # 指定版本
```

**自动完成的操作：**

- ✅ 更新 `package.json` 版本号
- ✅ 更新 `src-tauri/Cargo.toml` 版本号
- ✅ 更新 `src-tauri/tauri.conf.json` 版本号
- ✅ 更新 `Cargo.lock`
- ✅ Git 提交（`chore(release): x.y.z`）
- ✅ 创建 Git 标签（`vx.y.z`）
- ✅ 可选：推送到远程

详细文档：[VERSION_MANAGEMENT.md](../docs/VERSION_MANAGEMENT.md)

---

### 🔍 跨平台编译检查

## 📋 可用脚本

### 1. `check-quality.sh` - 快速代码质量检查 ⚡ (推荐)

**用途**: 在本地快速验证代码质量，包括 Clippy、格式检查和单元测试

**使用方法**:

```bash
# 方法 1: 直接运行脚本
./scripts/check-quality.sh

# 方法 2: 使用 Makefile
make check-quality
```

**包含的检查**:

- ✅ Clippy 代码检查（捕获常见错误）
- ✅ 代码格式检查
- ✅ 单元测试

**特点**:

- 速度快（不涉及交叉编译）
- 最接近 CI 环境的检查
- **推荐在每次提交前运行**

**代码质量保证**:

- ✅ Clippy 检查捕获常见错误和代码异味
- ✅ 格式检查确保代码风格一致
- ✅ 单元测试验证核心功能
- ✅ 跨平台兼容性检查在 GitHub Actions CI 中自动进行

---

### 2. `check-all-platforms.sh` - 全平台检查 🌍

**用途**: 检查代码在所有主要平台上的编译情况

**使用方法**:

```bash
# 方法 1: 直接运行脚本
./scripts/check-all-platforms.sh

# 方法 2: 使用 Makefile
make check-cross
```

**检查的平台**:

- ✅ Linux (x86_64-unknown-linux-gnu)
- ✅ Windows (x86_64-pc-windows-msvc)
- ✅ macOS Intel (x86_64-apple-darwin)
- ✅ macOS Apple Silicon (aarch64-apple-darwin)

**特点**:

- 全面覆盖
- 自动安装所有必要的编译目标
- 显示详细的结果汇总

---

### 3. `make check-ci` - 模拟 CI 环境 🤖

**用途**: 模拟 GitHub Actions CI 检查流程

**使用方法**:

```bash
make check-ci
```

**包含的检查**:

1. Linux 平台编译检查
2. 运行所有单元测试

**推荐用法**: 在提交和推送前运行此命令

---

## 🚀 典型工作流

### 场景 1: 日常开发（快速检查）

```bash
# 1. 修改代码
vim src-tauri/src/wallpaper_manager.rs

# 2. 快速代码质量检查
make check-quality

# 3. 如果通过，提交代码
git add -A
git commit -m "your message"
git push
```

### 场景 2: 重要修改（全面检查）

```bash
# 1. 修改代码
vim src-tauri/src/wallpaper_manager.rs

# 2. 检查所有平台
make check-cross

# 3. 运行测试
make test

# 4. 或者直接运行 CI 模拟
make check-ci

# 5. 如果通过，提交代码
git add -A
git commit -m "your message"
git push
```

### 场景 3: 修复 CI 失败

```bash
# 1. CI 失败，查看错误信息（例如 Linux 编译错误）

# 2. 本地复现问题
make check-quality

# 3. 修改代码

# 4. 再次检查
make check-quality

# 5. 确认通过后推送
git add -A
git commit -m "fix: resolve Linux compilation error"
git push
```

---

## 🔧 技术细节

### 工作原理

1. **编译目标安装**: 使用 `rustup target add` 安装交叉编译目标
2. **编译检查**: 使用 `cargo check --target <platform>` 进行编译检查
3. **不需要链接器**: `cargo check` 只检查代码能否编译，不需要实际链接，因此不需要安装交叉编译链接器

### 优势

- ⚡ 快速：`cargo check` 比完整编译快得多
- 🎯 准确：与 CI 环境使用相同的编译目标
- 💻 本地：无需等待 CI 运行
- 🔄 可重复：可以反复运行直到通过

### 限制

- ⚠️ **Tauri Linux 交叉编译限制**: Tauri 应用依赖 GTK/WebKit 等 Linux 系统库，在 macOS 上无法进行完整的交叉编译检查
- ⚠️ 只检查编译，不检查链接
- ⚠️ 不会检查平台特定的运行时行为
- ⚠️ 无法检测某些链接器错误

### 实际建议

对于 Tauri 项目，推荐的本地检查流程：

1. **使用 `make check-quality`**: 运行代码质量检查（快速，全面）
2. **使用 `make check-cross`**: 检查跨平台编译兼容性（可选）
3. **依赖 GitHub Actions**: 进行完整的 CI/CD 流程验证

### 首次运行

首次运行 `check-cross` 会自动下载编译目标，需要一些时间：

```bash
$ make check-cross
📦 安装 x86_64-unknown-linux-gnu...
info: downloading component 'rust-std' for 'x86_64-unknown-linux-gnu'
info: installing component 'rust-std' for 'x86_64-unknown-linux-gnu'
...
```

后续运行会很快。`check-quality` 无需下载额外依赖，始终快速。

---

## 📊 与 CI 的对比

| 特性     | 本地检查       | GitHub Actions |
| -------- | -------------- | -------------- |
| 速度     | ⚡ 快 (秒级)   | 🐢 慢 (分钟级) |
| 成本     | 免费           | 有配额限制     |
| 反馈周期 | 即时           | 需要推送等待   |
| 完整性   | 编译检查       | 完整 CI 流程   |
| 建议用途 | 开发时快速验证 | 最终验证       |

---

## 💡 最佳实践

1. **每次修改代码后运行** `make check-quality`
2. **提交前运行** `make check-ci`
3. **重大重构时运行** `make check-cross`
4. **配置 Git pre-push hook**（可选）:

```bash
# .git/hooks/pre-push
#!/bin/sh
make check-quality
```

---

## 🐛 故障排除

### 问题: 脚本没有执行权限

```bash
# 解决方案：
chmod +x scripts/*.sh
```

### 问题: rustup 找不到

```bash
# 解决方案：安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 问题: 编译目标下载失败

```bash
# 解决方案：手动安装
rustup target add x86_64-unknown-linux-gnu
```

---

## 📚 相关资源

- [Rust 交叉编译文档](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo check 文档](https://doc.rust-lang.org/cargo/commands/cargo-check.html)
- [Tauri 跨平台构建指南](https://tauri.app/v1/guides/building/cross-platform)
