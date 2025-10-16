# 质量基线 (Quality Baseline)

本目录用于集中存放项目的质量与可维护性相关策略、指标、工具与扩展计划。目标是保持：
- 构建与测试稳定可重复
- 关键质量指标透明可见
- 易于增量扩展（新增检查/覆盖率/安全策略）
- 本地与 CI 行为一致（差异只在是否启用网络/重度检查）

## 目录结构规划

当前/计划的集中化配置：

```
.config/
  ├── eslint/              # 前端 ESLint 规则（已迁移）
  ├── prettier/            # Prettier 配置（计划；格式化策略）
  ├── quality/             # 本说明与指标文档
  ├── scripts/             # 可复用的质量脚本（lint / coverage / 基线检查等）
  └── security/            # （计划）安全策略、忽略清单说明
src-tauri/
  └── deny.toml            # cargo-deny：安全 & 许可证 & bans
.editorconfig              # 跨语言基础格式约束
```

脚本（计划添加）：
- `scripts/check-rust.sh`：Rust 格式 + Clippy + 测试组合
- `scripts/lint-frontend.sh`：前端 lint（ESLint）
- `scripts/coverage-rust.sh`：Rust 覆盖率（tarpaulin）
- `scripts/coverage-frontend.sh`：前端覆盖率（vitest + c8）
- 后续可引入：`scripts/validate-dist.sh`、`scripts/security-scan.sh` 等

## 质量门槛分类

| 类型 | 门槛级别 | 策略说明 |
|------|----------|----------|
| 格式化 (fmt, Prettier) | Hard Fail | 不允许未格式化代码进入主分支；CI 阻断 |
| Clippy / ESLint | Hard Fail | 所有警告按约定转为失败（`-D warnings` / 可选 White/Gray list） |
| 编译 / 构建 | Hard Fail | 构建失败直接阻断；不允许将部分可编译代码带入主分支 |
| 单元测试 (Rust) | Hard Fail | 所有非 ignored 测试必须通过 |
| 前端类型检查 (tsc) | Hard Fail | 类型错误阻断 |
| 网络依赖测试 (Bing API) | Soft / Opt-in | 默认跳过，仅在设置 `BING_TEST=1` 时运行 |
| 许可证与安全（cargo-deny） | Hard Fail (vuln/unsound/yanked) / Soft (unmaintained) | critical / high 风险必须处理；临时忽略需文档说明 |
| 重复依赖版本 | Warn | 初期以观察为主；后续可转 hard fail |
| 覆盖率阈值 | 计划：Soft -> Hard | 建立基线后逐步提高（例如 Rust 40% -> 60% -> 75%） |

## 覆盖率策略（初始计划）

先建立基线后再设阈值：
1. 引入工具：
   - Rust: `cargo tarpaulin --out Lcov`
   - 前端: `vitest --coverage`
2. 基线落盘：在本目录记录首次测量数据（`baseline.json` 或 `coverage/` 子目录）
3. CI 中报告但不阻断（软提示）
4. 阈值策略提升：
   - 第 1 阶段：Rust ≥ 40%，前端 ≥ 30%
   - 第 2 阶段：Rust ≥ 60%，前端 ≥ 50%
   - 第 3 阶段：Rust ≥ 75%，前端 ≥ 65%
5. 排除模式：
   - Rust：`build.rs`、平台 FFI、生成代码
   - 前端：类型声明、纯样式文件、入口脚本（如 `main.tsx`）

示例（未来脚本输出）：
```
coverage/
  rust-lcov.info
  rust-summary.json
  frontend-summary.json
```

## 安全与许可证策略

使用 `cargo-deny` 管控：
- 拒绝：`unsound`, `yanked`, `vulnerability`
- 警告：`unmaintained`, `notice`
- 许可证白名单：MIT / Apache-2.0 / BSD(2/3) / ISC / Unicode-DFS-2016
- 许可证黑名单：GPL / AGPL / LGPL / MPL / CDDL / EUPL
- 若需临时忽略：
  - 在 `deny.toml` 中添加 `ignore = ["RUSTSEC-YYYY-XXXX"]`
  - 在 PR / commit message 中说明：影响范围、替代计划、处理期限

未来扩展（计划）：
- 添加 SBOM 生成（例如使用 `cargo auditable` 或 CycloneDX）
- 发布产物签名（macOS notarize / Windows signtool）

## 本地与 CI 行为一致性原则

| 行为 | 本地 | CI |
|------|------|----|
| `npm run typecheck` | 与 CI 相同 | 必须通过 |
| `cargo fmt` | 可自动修复 | CI 执行 `--check` |
| `cargo clippy` | 可以迭代修复 | CI 阻断警告 |
| `npm run build` | 性能优化非必须 | CI 强制 |
| 覆盖率 | 手动运行脚本 | 初期：统计输出 |
| 网络测试 | 手动 `BING_TEST=1` | 默认跳过 |

## 推荐工作流

```
# 1. 修改代码
# 2. 本地快速验证
npm run typecheck
npm run lint:frontend   # 安装 ESLint 后
npm run lint:rust
npm run test:rust

# 3. 可选：覆盖率
./.config/scripts/coverage-rust.sh
./.config/scripts/coverage-frontend.sh

# 4. 提交前自动化
make check    # (如果 Makefile 映射这些脚本)
```

## 未来扩展路线图

| 阶段 | 项目 | 描述 |
|------|------|------|
| v0 | Prettier 集成 | 添加 `.config/prettier/.prettierrc`，统一前端格式 |
| v1 | 覆盖率初始报告 | 建立 rust/frontend 基线 |
| v2 | 覆盖率阈值软限制 | 低于基线警告 |
| v3 | 阈值硬限制 | CI 阻断 |
| v4 | SBOM + 签名 | 发布合规与供应链保障 |
| v5 | 冲突依赖自动建议 | 检测重复版本给出 `cargo update -p xxx` 建议 |
| v6 | 变更影响分析 | 基于 git diff 选择性测试（可选） |

## 新增质量检查的步骤模板

1. 确认类别（性能 / 安全 / 可维护性 / 风险）
2. 选择工具（如：`cargo audit` / `semgrep` / `depcheck` / `license-checker`）
3. 本地脚本化（放置于 `.config/scripts/`)
4. README 增补说明
5. CI 中添加步骤（先 `soft fail` 再升级）
6. 维护策略（多久审查一次 / 如何忽略 / 如何升级）

## 升级 / 忽略流程规范

- 所有忽略必须：
  - 有明确编号（如 RUSTSEC）
  - 有到期日（放在 PR 描述或 issue）
  - 有替代方案（迁移依赖 / 等待上游修复）
- 定期复审（建议每月或 Release 前）

## 风险控制建议

| 风险 | 方案 |
|------|------|
| 未测试的新平台行为 | 添加 smoke test 脚本 |
| 依赖突然 yanked | cargo-deny 保障 + 加速迁移 |
| 覆盖率虚高（重复执行逻辑） | 引入变异测试（未来可选） |
| 格式化规则漂移 | `.editorconfig` + Prettier 双轨固定 |
| 安全公告滞后 | 强制启用 `fetch = true`（已配置） |

## 快速命令（计划脚本占位示例）

```
# Rust 基线检查
./.config/scripts/check-rust.sh

# 前端 Lint
./.config/scripts/lint-frontend.sh

# Rust 覆盖率
./.config/scripts/coverage-rust.sh

# 前端覆盖率
./.config/scripts/coverage-frontend.sh
```

## 贡献者须知（质量相关）

- 不要在 PR 中加入临时忽略而不解释
- 避免引入 wildcard 版本（已由 cargo-deny deny）
- 优先修复警告而不是隐藏（`allow` 只在有明确理由时添加）
- 如果新增脚本：保持可读性 + 注释顶部说明用途/依赖

## FAQ

Q: 为什么重复依赖暂时只是 warn？  
A: 初期关注功能迭代；等依赖图稳定后再收紧为 deny。

Q: 为什么网络测试忽略？  
A: 避免 CI 因临时网络故障不稳定；只在需要验证 Bing API 行为时手动启用。

Q: 可以加入性能基准吗？  
A: 可以，建议后续添加 `cargo bench` + 前端 Lighthouse 脚本，初期只采集数据不阻断。

---

若你需要我立即初始化脚本与 Prettier 配置或生成覆盖率占位文件，请提出具体列表，我可以继续编写对应内容。