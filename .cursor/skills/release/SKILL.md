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
- 需要执行正式发布前检查并确认发布范围
- 需要更新变更记录并走标准发布命令

## Invocation Mode

- 当前配置为 `disable-model-invocation: true`，仅在显式 `/release` 时生效
- 如果希望模型在相关上下文自动调用，可将其改为 `false` 或删除该字段

## 职责分工

AI 负责：

1. 确认发布范围（对比 tag 差异）
2. 更新 `CHANGELOG.md` 并提交
3. 按需更新 `AGENTS.md`、`README.md`、`README.zh.md` 并一起提交

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

3. 语言与粒度——以用户视角为首要原则
   - **只写用户可感知的变更**，省略纯内部重构、开发工具链、文档维护等条目
   - **用用户视角描述结果**，而非技术实现细节：写"修复了什么问题"或"现在可以做什么"，不写"启用了哪个配置""重构了哪个模块"
   - **每条变更只写一行**，禁止嵌套子项（子弹列表）
   - **合并相关变更**为一条，用逗号或顿号连接关键点，而非拆成多条
   - 措辞精炼直白，去掉"无需跳转浏览器""替代手写比较逻辑"等冗余修饰
   - 示例对比：
     ```
     # 冗长且技术化 ❌
     - 应用内更新：支持在更新弹窗中直接下载安装新版本，无需跳转浏览器
       - 下载进度条、取消下载、错误提示
       - 下载完成后自动重启
     - 开发模式版本模拟：`make dev MV=0.0.1` 可模拟旧版本触发更新流程
     - 修复应用内更新：启用 updater 签名产物生成，修复签名密钥配置问题
     - 修复 macOS 双架构更新包同名覆盖问题

     # 精简且面向用户 ✅
     - 应用内更新：支持直接下载安装新版本，带进度显示和自动重启
     - 修复应用内自动更新无法正常工作的问题
     ```
   - **判断标准**：把条目念给一个普通用户听，他能理解"这对我有什么影响"吗？如果不能，重写

## Instructions

1. 工作区预检（一次性完成，不委派外部 skill）
   - 并行获取：当前版本、`git status`、最近 tag 到 HEAD 的提交列表
   - **未提交变更**：直接 `git add -A && git commit -m "chore: pre-release changes"` 并 `git push`，不走 commit skill
   - **非开发版本**（无 `-0` 后缀）：以选项询问 patch/minor/major，运行 `make <level> YES=1`（仅 bump 版本号，不更新依赖）
   - 预检完成后，工作区应干净且版本为 `X.Y.Z-0`
   - 对比最近 tag 到 HEAD 的差异，确认本次发布内容边界

2. 更新发布说明
   - 更新 `CHANGELOG.md`，直接写在目标版本标题下
   - 先整理证据清单（提交/PR），再写入条目
   - 不得把原始 commit log 原样粘贴为 changelog

3. 按需更新项目文档
   - 对比本次发布范围内的变更，判断以下文件是否需要同步更新：
     - `AGENTS.md`：项目结构、命令、代码规范、Tauri 配置等开发指南
     - `README.md`：面向用户的英文功能介绍、使用说明、FAQ
     - `README.zh.md`：面向用户的中文功能介绍、使用说明、FAQ
   - 更新原则：
     - 仅当本次发布涉及**新增/删除文件、新增/移除功能、命令变化、配置变化**时才需要更新
     - 保持三个文件各自的现有风格和语言（AGENTS.md 中英混用、README.md 英文、README.zh.md 中文）
     - 不做无意义的润色或重写，只同步实际变更
   - 若无需更新，跳过此步骤

4. **确认发布内容（必须）**
   - 展示 CHANGELOG 新增内容；若步骤 3 中更新了文档，一并展示变更摘要
   - 以选项形式请求确认：
     - **确认发布**
     - **取消**
   - 用户也可直接输入修改意见代替点击选项，AI 按反馈修正后重新展示并确认
   - **必须等待用户确认后才能进入下一步**

5. 提交发布文档
   - 提交 `CHANGELOG.md` 及本次更新的文档文件（`AGENTS.md`、`README.md`、`README.zh.md`）
   - **不要调用 commit skill**，直接用固定格式提交：
     ```bash
     git add CHANGELOG.md AGENTS.md README.md README.zh.md  # 仅添加实际修改的文件
     git commit -m "chore: add CHANGELOG entry for X.Y.Z"
     ```
   - 若同时更新了文档，提交信息改为 `docs: update docs and CHANGELOG for X.Y.Z`

6. 执行发布命令
   - 运行 `make release`
   - 脚本会自动运行质量检查、更新版本、提交、打 tag、推送
   - 若失败：
     - 分析错误原因并展示
     - 若为可修复的代码问题（lint、compile、format）：修复并提交，重试 `make release`（最多重试 1 次）
     - 若为不可恢复的错误或重试仍失败：停止并说明原因

7. 返回发布结果
   - 返回版本号、tag、关键提交与后续验证建议
   - 附上 CI 构建链接

8. 监控 CI 构建
   - 使用 `gh run list --limit 1` 获取本次 tag 触发的 workflow run
   - 每隔 30–60 秒用 `gh run view <run_id>` 轮询状态，直到所有 job 完成或超时（15 分钟）
   - 轮询期间简要播报进度（如"Bundle 阶段进行中，已完成 3/6 平台"）
   - 全部成功时输出最终结果并结束
   - 若有 job 失败：
     - 用 `gh run view <run_id> --log-failed` 获取失败日志摘要
     - 展示失败 job 名称和关键错误信息
     - 以选项形式询问用户：
       - **排查并修复** — 分析日志、定位原因、尝试修复；修复提交后以选项询问是否执行 `make retag` 重新触发构建，若用户确认则 retag 并回到步骤 8 重新监控
       - **暂不处理** — 结束流程，留给用户后续手动处理

## Output

发布成功后，简要汇总：版本号、tag、CI 构建链接。有风险或待验证事项时一并说明。无需固定模板，自然语言即可。

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
