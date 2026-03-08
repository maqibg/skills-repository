# skills管理器

`skills管理器` 是一个面向多 Agent 开发者的桌面应用，目标是把本地 skills 的扫描、统一仓库、分发、安全、模板注入、国际化与主题切换整合到同一个控制台中。

## 当前阶段

当前仓库已进入第一阶段骨架开发：

- React + TypeScript + Vite 前端壳
- Tauri 2 + Rust 后端骨架
- SQLite 初始化与 settings 存储
- 中英日 i18n
- 明暗主题切换
- 本地 skills 扫描基础版
- 任务中心事件总线

## 运行方式

```bash
pnpm install
pnpm tauri dev
```

## 关键文档

- `PRD.md`：产品需求真源
- `TechDesign.md`：技术设计真源
- `PrototypeDesign.md`：页面与原型设计说明
- `PrototypePromptPack.md`：AI 设计工具提示词包
- `AGENTS.md`：项目协作规范

## 注意事项

- 当前仍处于第一阶段，不包含市场安装、分发闭环、安全阻断和模板注入的完整实现。
- 重要变更必须优先回写文档，不要只改代码不改真源文档。
