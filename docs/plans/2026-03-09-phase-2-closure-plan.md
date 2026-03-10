# Phase 2 Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 收口第二阶段主链路，让扫描结果、概览统计、错误模型和未接通模块表达都回到真实状态，为后续市场安装、安全扫描与多 Agent 分发打下稳定基础。

**Architecture:** 以 Rust 后端为单一真源重建“扫描 -> 持久化 -> bootstrap/overview -> 前端渲染”链路，避免前端拼装假数据。前端仅负责状态呈现、显式未接通占位、主题/i18n 同步；Repository 负责 SQLite 落库与聚合；Service 负责扫描编排与错误可见性。

**Tech Stack:** React 19 + TypeScript + Zustand + Vite + Tauri v2 + Rust + rusqlite + SQLite

---

## 当前阶段判断

| 项目 | 现状 | 结论 |
|---|---|---|
| 扫描链路 | 已能扫描内存结果，但未落库 | 需要补 Repository 持久化 |
| Overview | 前端存在假 `0` 统计 | 需要改为后端真值或显式未知 |
| 启动与设置 | 已修正 bootstrap 错误升级和隐式重扫 | 作为第二阶段已完成子项保留 |
| Security 页 | 仍是伪结果态 | 需要改为“未接通”表达 |
| i18n | 多处用户可见文案未完全收口 | 第二阶段内需要收口核心路径 |

## 执行顺序

| 顺序 | 任务 | 原因 |
|---|---|---|
| 1 | 扫描结果持久化与 overview 真值 | 是后续一切治理能力的基础 |
| 2 | 扫描错误显式化 | 避免“扫描成功但结果不完整” |
| 3 | 前端 overview / Security 占位收口 | 去掉伪真实状态 |
| 4 | Agent 路径归属语义收口 | 防止共享路径误归属 |
| 5 | 第二阶段验证与文档回写 | 防止实现与真源继续漂移 |

### Task 1: 建立扫描持久化基础

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/mod.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/scan.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Test: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`

**Steps:**
1. 为扫描落库设计 repository API，至少包含 `replace_scan_snapshot`、`load_overview_stats`、`load_scan_snapshot`。
2. 先写 Rust 单测，覆盖“扫描后重启前后 skills/projects/distributions 快照可读取”。
3. 在 `scan.rs` 中把扫描结果写入 SQLite，而不是直接以内存结果结束任务。
4. 调整 `commands/app.rs` 的 `persist` 步骤，使其真正表示“已落库”而不是假状态。
5. 运行 `cargo test`，确认扫描单测和现有测试仍通过。

**Done when:**
- 扫描完成后数据库中存在稳定的 `projects` / `skills` / `skill_distributions` 快照
- 重启后 `bootstrap_app` 可以读取上次扫描结果的 overview

### Task 2: 让 overview 只来自后端真值

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/bootstrap.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/types.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/stores/use-task-store.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/stores/use-app-store.ts`
- Test: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/scan.rs`

**Steps:**
1. 在后端补 `overview` 聚合逻辑，至少返回真实 `totalSkills`、`duplicatePaths`，其余未接通值改为可区分“未知/未接通”的表达。
2. 如果后端当前还不能提供 `riskySkills` / `reclaimableBytes` / `templateCount`，明确约定字段语义，不再让前端写死成 `0`。
3. 删除 `use-task-store.ts` 中扫描完成后手工拼装 overview 的逻辑，改成消费后端返回值或保留旧值。
4. 确保 `bootstrap_app` 返回的 overview 与扫描完成后的 overview 口径一致。
5. 补测试，覆盖“扫描后 overview 更新”和“重启后 overview 保持”。

**Done when:**
- 前端不再把未知治理指标写成 `0`
- `overview` 成为后端单一真源

### Task 3: 让扫描错误显式可见

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/tasks/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Test: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`

**Steps:**
1. 把 `canonical_string` 改成返回 `Result<String>`，不要在失败时退回原始路径。
2. 把 `discover_skill_dirs` 改成收集 `walkdir` 错误，而不是 `filter_map(Result::ok)` 直接吞掉。
3. 为非法路径、权限失败、坏链接设计清晰的任务失败或 `partial` 语义。
4. 让任务事件 helper 返回 `Result` 或至少在失败时显式打日志，不再无声吞掉。
5. 补覆盖异常路径的单测。

**Done when:**
- 扫描异常不会再显示为“成功但结果不完整”
- 日志和任务状态能说明失败原因

### Task 4: 收口前端的伪结果态和未接通模块表达

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src/pages/SecurityPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/OverviewPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/SkillsPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/zh-CN/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/en-US/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/ja-JP/common.json`
- Test: `E:/vibecoding-project/skills-repository/src/pages/*.tsx`

**Steps:**
1. 把 Security 页从固定 `0` 卡片改成明确的“未接通/即将开发”状态。
2. 把 Overview 页中会误导为真实结果的占位文案改成显式未接通文案。
3. 把硬编码用户可见文案继续收口到 i18n，至少覆盖第二阶段核心路径。
4. 检查主题 token 使用，保证未接通状态在亮暗主题都可读。
5. 运行 `corepack pnpm typecheck` 和定向 `eslint`。

**Done when:**
- 用户不会再看到伪造的安全/治理结果
- 第二阶段核心页面在中英日下不出现关键硬编码文案

### Task 5: 补 Agent 路径归属语义

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/agent_registry.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/types.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/types/app.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/SkillsPage.tsx`
- Test: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`

**Steps:**
1. 为 Agent 路径加入 `primary / alias / priority / compatibleAgents` 语义。
2. 扫描时先解析路径归属，再去重，避免 `.agents/skills` 被错误抢占。
3. 必要时把前端 `SkillRecord.agent` 从单值升级成更能表达共享归属的结构。
4. 补单测覆盖共享路径和兼容别名路径。
5. 确认 Skills 页展示不会误导用户归属关系。

**Done when:**
- 共享标准路径不会再被错误归属到单一 Agent

### Task 6: 第二阶段验证收口

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/tep-docs/TechDesign.md`
- Modify: `E:/vibecoding-project/skills-repository/tep-docs/PRD.md`
- Optional: `E:/vibecoding-project/skills-repository/README.md`

**Steps:**
1. 对照实现回写文档，把已实现、未实现、待后续阶段完成的契约重新标清。
2. 运行最小验证命令：`corepack pnpm typecheck`、`cargo test`。
3. 如修改了前端核心页面，补跑定向 `eslint`；如完成桌面链路，补一次 `corepack pnpm tauri dev` 冒烟。
4. 记录第二阶段剩余非目标项：市场 provider、真实安装、安全评分、模板注入、更新治理。

**Done when:**
- 文档与实现认知一致
- 第二阶段关闭时没有继续伪装未接通能力

## 交付验收清单

| 类别 | 必须满足 |
|---|---|
| 数据真值 | 扫描结果与 overview 不再依赖前端假数据 |
| 错误可见 | 扫描异常、事件失败不再静默吞掉 |
| 页面表达 | Security/Overview 未接通模块不再显示伪真实 `0` |
| i18n | 第二阶段核心路径不出现关键硬编码用户文案 |
| 验证 | `corepack pnpm typecheck`、`cargo test` 通过 |

## 建议分批提交

| 提交批次 | 范围 | 推荐提交信息 |
|---|---|---|
| Batch 1 | 扫描持久化 + overview 真值 | `feat: persist scan snapshot and derive overview from backend` |
| Batch 2 | 扫描错误可见 + 任务事件错误处理 | `fix: surface scan and task event failures explicitly` |
| Batch 3 | 前端未接通态与 i18n 收口 | `refactor: replace fake security and overview placeholders` |
| Batch 4 | Agent 路径归属语义 | `feat: preserve agent path priority in scan results` |

## 新窗口执行建议

| 建议 | 内容 |
|---|---|
| 开发入口 | 先从 Task 1 开始，不要先改 Security 页 |
| 首个检查命令 | `cargo test` |
| 第二个检查命令 | `corepack pnpm typecheck` |
| 开发顺序 | 后端真值 -> 前端消费 -> 文案/i18n 收口 |

