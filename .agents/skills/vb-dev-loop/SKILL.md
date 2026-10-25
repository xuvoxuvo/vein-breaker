---
name: vb-dev-loop
description: Vein Breaker 的标准开发循环——一页 spec、一个垂直切片、四级自动验证、回写文档。在本仓库实现功能、修 bug 或调数值时使用。
whenToUse: 用户要求在本仓库新增功能、修复缺陷、调整玩法数值，或你即将修改 crates/ 下的代码时。
---

# Vein Breaker 开发循环

本仓库是单人主导 + AI 实现的游戏项目。目标不是"一次写对"，而是**每轮都小、可验证、可回滚**。

## 何时使用

- 新增一个玩法功能
- 修复一个可复现的 bug
- 调整数值/关卡
- 重构某个 feature

不适用：纯文档修改、纯调研（那些不需要 spec 与测试）。

## 前置检查（30 秒）

1. 读根目录 `AGENTS.md`（命令、约定、禁止项）。
2. 读 `docs/architecture.md` 的相关章节，确认改的是哪个 feature。
3. 读 `docs/prototype-01.md` 与 `AGENTS.md` 的「Bevy 0.20 事实」一节。**没有记录的 API 不许凭记忆写。**

## 六步

### 1. Spec（1 页）

复制 `docs/specs/000-template.md` 为 `docs/specs/NNN-<slug>.md`，填：
目标（玩家感受）、规则、数据接口、验收标准、非目标、风险。

验收标准必须写成"给定…当…则…"，否则无法验证。

### 2. Plan（5 行）

在回复里列出：

```
将改：
- crates/vb_game/src/features/<x>/systems.rs（新增 <系统>）
- assets/data/tuning.ron（新增 <字段>）
测试：
- vb_core 单测：<规则>
- ECS 测试：<行为>
不做：<越界项>
```

跨 3 个以上 feature 目录时，先停下来把任务拆小。

### 3. Implement

- 一次只做一个垂直切片，diff ≤ 400 行；
- 新逻辑优先放 `vb_core`（纯函数、可毫秒级测试）；
- 表现层只消费消息，不改玩法状态；
- 数值写进 RON，不硬编码。

### 4. Verify（按阶梯，不许跳）

```powershell
cargo fmt --check
cargo clippy --offline --workspace --all-targets -- -D warnings
cargo test -p vb_core
cargo build --offline
cargo run --offline
```

- 依赖已缓存时一律加 `--offline`：本机 cargo 走代理，联网更新索引会卡住；
- L0–L3 全绿才算"实现完成"；
- `cargo run` 至少跑一次目标场景，确认无 panic、无穿模；
- 视觉改动用 `cargo run --offline -- --screenshot docs/images/prototype.png` 留证据，
  不要用文字描述"看起来正常"。

### 5. Record

| 发生了什么 | 写到哪里 |
|---|---|
| 新的 API 事实/坑 | `docs/bevy-0.20-api.md` |
| 架构决策 | `docs/adr/NNNN-*.md` |
| 改了开发约定 | `AGENTS.md` |
| 调了数值 | RON 文件 + 一行注释说明原因 |
| 功能完成 | 更新 spec 的验收清单 |

**不写 = 下一轮重踩。**

### 6. Report

给用户的回复里包含：

- 做了什么（1–3 行）
- 怎么验证的（贴命令与结果）
- 没做什么 / 已知问题
- 需要人拍板的事

## 硬规则

- 未通过 fmt/clippy/test 的代码，不许说"完成"。
- 不许升级 Bevy/Avian；需要升级时单独开一轮迁移。
- 不许引入 `bevy_hanabi` / `bevy_fps_controller`（仍锁 Bevy 0.19）。
- 不许在 `Update` 里做物理或扣血（优先 `FixedUpdate`）。
- 不许把 `[profile.dev] debug` 调回完整 debuginfo（会产出无法启动的 2GB exe）。
- 一个新 crate 依赖前，先 `cargo tree -i bevy@0.19.1` 检查是否拉进第二份 Bevy。
- 不确定的 API：写 10 行探针 → `cargo check` → 记录结论 → 再实现。

## 失败处理

- 编译错误 > 10 条：回退到上一个 commit，把切片拆小。
- 测试通过但试玩不对：这是 spec 的问题，先改验收标准，再改代码。
- 连续两轮同一处失败：停下来写一段诊断（假设 → 验证方法），不要继续盲改。
