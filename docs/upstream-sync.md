# 上游已处理台账

本文件用于记录 `linbei0/skills-repository` 上游提交在当前 fork 中的处理状态。

统一使用以下三种标记：

- `Upstream-cherry-picked-from: <sha>`
- `Upstream-ported-from: <sha>`
- `Upstream-reviewed-not-ported: <sha>`

## 当前基线

- 上游仓库：`linbei0/skills-repository`
- 对比分支：`master`
- 最近核对日期：`2026-03-22`
- 参考命令：`git fetch https://github.com/linbei0/skills-repository.git master`、`git log --oneline --reverse HEAD..FETCH_HEAD`

## 已原样吸收

当前无 `Upstream-cherry-picked-from` 记录。

## 已手工移植

`Upstream-ported-from: 8698fcd`

原因：`Codex IDE` 目标与 `visibleSkillsTargetsVersion` 迁移已在当前 fork 手工移植并接入本地设置归一化链路。

本地落点：`src-tauri/src/domain/agent_registry.rs`、`src-tauri/src/services/settings.rs`

`Upstream-ported-from: 85d58a5`

原因：GitHub 仓库 skill 更新链路已在当前 fork 手工移植，但实现按本地约束改为走共享 `HttpClient` 与 `ProxySettings`，而不是直接照搬上游请求方式。

本地落点：`src-tauri/src/services/repository_update.rs`、`src/lib/tauri-client.ts`、`src/stores/use-repository-store.ts`

`Upstream-ported-from: 24aba64`

原因：仓库页搜索增强已手工移植，当前吸收了模糊搜索、高亮、分页、清空搜索和结果摘要，但没有引入上游的 `Vitest` 与完整页面重构。

本地落点：`src/lib/repository-search.ts`、`src/components/common/HighlightedText.tsx`、`src/pages/RepositoryPage.tsx`

## 明确跳过

`Upstream-reviewed-not-ported: 41cda6d`

原因：这是 `8698fcd` 的 merge commit，本身没有独立功能价值；对应功能已按 `8698fcd` 记录为手工移植。

`Upstream-reviewed-not-ported: d3aa95e`

原因：该提交主要是仓库模块与测试文件拆分，属于可维护性重构，不属于当前 fork 必须吸收的功能增量。

`Upstream-reviewed-not-ported: 5ac1846`

原因：该提交的版本号更新到 `0.1.1` 已经过时；当前 fork 版本已继续演进，相关 `User-Agent` 也已独立处理。

`Upstream-reviewed-not-ported: dc5d0ae`

原因：该提交主要是仓库页组件拆分。当前 fork 只吸收了搜索增强能力，不采用整页拆分方案，以保持本地改动最小。
