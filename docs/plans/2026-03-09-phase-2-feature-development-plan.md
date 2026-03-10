# Phase 2 Feature Development Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 在第二阶段收口任务基础上，完成当前仍未开发或未闭环的主链路功能：市场搜索、技能安装、安全扫描与阻断、多 Agent 分发、模板管理与模板注入。

**Architecture:** 以 Rust 后端作为功能真源，把市场、安装、安全、分发、模板这几条链路都收口到 `command -> service -> repository -> filesystem/SQLite`。前端只负责状态、任务展示和表单交互，不在页面层拼业务规则；长任务统一走 `task:progress / completed / failed` 事件。

**Tech Stack:** React 19 + TypeScript + Zustand + Vite + Tauri v2 + Rust + SQLite + rusqlite + tempfile + walkdir

---

## 前置条件

| 项目 | 要求 |
|---|---|
| 依赖计划 | 先完成或至少稳定推进 `E:/vibecoding-project/skills-repository/docs/plans/2026-03-09-phase-2-closure-plan.md` |
| 最低真值要求 | 扫描结果、overview、错误模型不能继续依赖假 `0` 或伪 `persist` |
| 开发原则 | 不做 mock success；未接通能力必须明确标记；高风险路径优先显式失败 |

## 功能范围

| 优先级 | 功能 | 当前状态 | 本计划是否覆盖 |
|---|---|---|---|
| P0 | 市场搜索 | 页面壳存在，真实 provider 未接入 | 是 |
| P0 | 技能安装 | 未实现真实下载、落盘、索引入库 | 是 |
| P0 | 安全扫描与阻断 | 页面壳存在，安全引擎未接通 | 是 |
| P0 | 多 Agent 分发 | 仅扫描路径，无分发链路 | 是 |
| P1 | 模板管理 | 未实现 | 是 |
| P1 | 模板注入 | 未实现 | 是 |
| P1 | 更新治理 | 文档有预留，但不属于本次“未开发功能主链路”首批 | 否，放后续专项计划 |

## 建议执行顺序

| 顺序 | 功能 | 原因 |
|---|---|---|
| 1 | 市场搜索 | 安装的入口，先建立真实 market provider 与缓存契约 |
| 2 | 安装 + 安全预扫描 | 没有安装闭环，后续分发和模板都无法真正落地 |
| 3 | 多 Agent 分发 | 安装完成后的核心价值交付 |
| 4 | 模板管理 | 为模板注入建立可复用资产层 |
| 5 | 模板注入 | 依赖安装与分发能力 |
| 6 | 安全页/UI 收口 | 在真实安全数据接通后再完成前端 |

### Task 1: 补齐第二阶段后端数据契约与迁移

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/db.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/types.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/types/app.ts`

**Steps:**
1. 为第二阶段未开发能力补 migration：至少包含 `market_cache`、`security_reports`，并检查 `skills`、`skill_distributions`、`templates`、`template_items` 是否满足当前文档约束。
2. 设计前后端共享类型：市场搜索结果、安装请求/结果、安全报告、分发请求/结果、模板实体、模板注入结果。
3. 给 `commands/mod.rs` 和 `tauri-client` 预留真实 command surface，不提前暴露假接口。
4. 补与 migration 相关的 Rust 单测，确保数据库可初始化并兼容旧数据。
5. 运行 `cargo test`。

**Done when:**
- 第二阶段主链路需要的表结构和类型契约齐全
- 不再需要靠前端猜测未实现字段

### Task 2: 实现市场搜索基础层（Provider Adapter + Cache + Command）

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/adapters/mod.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/adapters/market.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/market.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/services/market.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/lib.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/lib/tauri-client.ts`

**Steps:**
1. 定义统一的 market provider trait / adapter 输出格式，支持 provider 独立状态上报。
2. 先接入一个真实 provider，确保不是 mock 数据。
3. 为搜索结果增加 SQLite 缓存，命中缓存与 provider 故障要可区分。
4. 新增 `search_market_skills` command，返回 `results + providers + pagination/cache metadata`。
5. 写 Rust 单测覆盖：provider 成功、单 provider 失败、缓存命中。
6. 运行 `cargo test`。

**Done when:**
- 市场搜索返回真实 provider 数据
- 单一 provider 故障不会污染全部结果

### Task 3: 接通市场页与市场状态管理

**Files:**
- Create: `E:/vibecoding-project/skills-repository/src/stores/use-market-store.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/MarketPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/types/app.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/zh-CN/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/en-US/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/ja-JP/common.json`

**Steps:**
1. 为市场页建立 Zustand store，管理 query、结果集、provider 状态、加载态、错误态。
2. 把当前占位页替换为真实搜索页，支持搜索、空状态、provider 状态展示。
3. 所有用户可见文案走 i18n，不新增硬编码英文。
4. 对未接通的 provider 或需要 key 的 provider，显式展示原因，不做静默降级。
5. 运行 `corepack pnpm typecheck`，必要时定向 `eslint`。

**Done when:**
- 用户可在 Market 页看到真实搜索结果和 provider 状态

### Task 4: 实现安装主链路（下载 -> 临时目录 -> 安全预扫描 -> canonical store -> 落库）

**Files:**
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/skills.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/services/install.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/security/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/tasks/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/app_state.rs`

**Steps:**
1. 设计安装输入输出：来源、下载 URL、目标 skill 元数据、期望落盘路径。
2. 用临时目录完成下载与解包，安全扫描在进入 canonical store 前执行。
3. 高风险命中时显式阻断并输出原因；中风险允许继续但要有提示。
4. 安装成功后写入 `skills`、必要时写 `operation_logs`，并返回统一结果。
5. 任务事件要体现 `download / security_check / persist / cleanup` 步骤。
6. 写集成测试覆盖：安装成功、高风险阻断、失败回滚。
7. 运行 `cargo test`。

**Done when:**
- 能真实安装一个市场 skill 到 canonical store
- 高风险 skill 默认被阻断，不会静默落盘

### Task 5: 实现多 Agent 分发服务

**Files:**
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/services/distribution.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/distributions.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/domain/agent_registry.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/lib/tauri-client.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/types/app.ts`

**Steps:**
1. 为分发设计请求结构：skill、目标 Agent、目标类型（全局/项目/自定义）、模式（native/symlink/copy）。
2. 按文档实现 Windows 优先策略：`symlink` 权限失败时显式报错，不做 silent fallback。
3. 成功后写入 `skill_distributions`，失败写日志并返回原因。
4. 覆盖两类核心测试：全局 `symlink` 成功 / 权限失败，项目 `copy` 成功。
5. 运行 `cargo test`。

**Done when:**
- 一个已安装 skill 能被真实分发到至少两个目标
- Windows 权限失败有明确原因

### Task 6: 接通 Skills 页的分发与详情能力

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src/stores/use-skills-store.ts`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/SkillsPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/components/layout/AppShell.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/zh-CN/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/en-US/common.json`
- Modify: `E:/vibecoding-project/skills-repository/src/locales/ja-JP/common.json`

**Steps:**
1. 为 Skills 页补详情、分发动作入口、分发状态列。
2. 从后端拉取真实 distribution 数据，不再只展示扫描路径。
3. 对未分发、已分发、分发失败做明确状态区分。
4. 分发动作先注册任务监听，再触发 command。
5. 运行 `corepack pnpm typecheck`。

**Done when:**
- Skills 页能从“只读清单”升级为真实治理入口

### Task 7: 实现安全报告存储与 Security 页真实数据

**Files:**
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/security.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/security/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/install.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/scan.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/SecurityPage.tsx`
- Create: `E:/vibecoding-project/skills-repository/src/stores/use-security-store.ts`

**Steps:**
1. 为安全报告定义规则输出、评分、阻断等级与明细结构。
2. 安装前扫描要写入 `security_reports`；重扫能力可通过独立 command 暴露。
3. Security 页从真实数据渲染，不再显示假 `0`。
4. 未扫描、已扫描、被阻断、可继续的状态都要显式可见。
5. 补 i18n 文案和最小测试。

**Done when:**
- Security 页展示真实安全报告
- review finding 中的“假 0”问题自然消失

### Task 8: 实现模板管理（CRUD）

**Files:**
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/repositories/templates.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/services/templates.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/lib/tauri-client.ts`
- Create: `E:/vibecoding-project/skills-repository/src/stores/use-templates-store.ts`
- Create: `E:/vibecoding-project/skills-repository/src/pages/TemplatesPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/app/router.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/components/layout/AppShell.tsx`

**Steps:**
1. 为模板实体与模板项建立 CRUD repository/service。
2. 提供创建、编辑、删除、查看模板详情的 command。
3. 前端建立模板页和模板 store，支持列表和表单。
4. 只实现当前 PRD 需要的 MVP 范围，不扩展分享/市场能力。
5. 运行 `cargo test` + `corepack pnpm typecheck`。

**Done when:**
- 用户可以创建、编辑、删除模板

### Task 9: 实现模板注入闭环

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/services/templates.rs`
- Create: `E:/vibecoding-project/skills-repository/src-tauri/src/services/template_injection.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/commands/app.rs`
- Modify: `E:/vibecoding-project/skills-repository/src-tauri/src/tasks/mod.rs`
- Modify: `E:/vibecoding-project/skills-repository/src/pages/TemplatesPage.tsx`
- Modify: `E:/vibecoding-project/skills-repository/src/types/app.ts`

**Steps:**
1. 设计模板注入输入：目标项目路径、模板、覆盖策略。
2. 注入时复用现有安装/分发能力，不另造平行链路。
3. 对每一项输出 `installed / skipped / failed` 结果。
4. 部分失败返回 `partial` 语义，不吞错误。
5. 补集成测试覆盖：成功、模板项缺失、部分失败。

**Done when:**
- 模板注入能输出清晰结果明细

### Task 10: 第二阶段功能验收与文档回写

**Files:**
- Modify: `E:/vibecoding-project/skills-repository/tep-docs/PRD.md`
- Modify: `E:/vibecoding-project/skills-repository/tep-docs/TechDesign.md`
- Modify: `E:/vibecoding-project/skills-repository/README.md`

**Steps:**
1. 回写哪些第二阶段功能已闭环、哪些仍保留到后续阶段。
2. 运行最小验证：`cargo test`、`corepack pnpm typecheck`。
3. 如功能已完整接入桌面端，补一次 `corepack pnpm tauri dev` 冒烟验证。
4. 按验收矩阵记录证据，不做“无证据完成声明”。

**Done when:**
- 文档与实现一致
- 新窗口开发者能凭文档判断哪些第二阶段主链路已完成

## 批次建议

| Batch | 范围 | 建议 |
|---|---|---|
| Batch A | Task 1-3 | 先打通市场搜索和契约底座 |
| Batch B | Task 4-6 | 完成安装与分发主链路 |
| Batch C | Task 7-9 | 完成安全页、模板管理、模板注入 |
| Batch D | Task 10 | 验证、文档回写、收口 |

## 推荐验证命令

| 场景 | 命令 |
|---|---|
| Rust 功能开发后 | `cargo test` |
| 前端类型检查 | `corepack pnpm typecheck` |
| 定向前端 lint | `corepack pnpm exec eslint src/pages/MarketPage.tsx src/pages/SecurityPage.tsx src/stores/*.ts` |
| 桌面冒烟 | `corepack pnpm tauri dev` |

## 新窗口执行建议

| 建议 | 内容 |
|---|---|
| 第一条消息 | “请按 `E:/vibecoding-project/skills-repository/docs/plans/2026-03-09-phase-2-feature-development-plan.md` 从 Task 1 开始执行。” |
| 执行顺序 | 先按 Batch A/B/C/D，不要跳过安装链路直接做模板注入 |
| 风险提示 | 若收口计划未完成，先不要把 overview/security 的假数据重新带回功能开发分支 |

