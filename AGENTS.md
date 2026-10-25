# AGENTS.md — Vein Breaker

> 本项目的一页契约。任何 AI 会话开始前必须先读本文件。
> 当前状态：**Prototype 0.1 可运行**（射击采矿）。

## 1. 项目

第一人称射击采矿：在坍塌矿脉里用凿岩枪打碎矿层，回收矿石。
Rust + Bevy + Avian，Windows 桌面优先。

- 玩法：[docs/gameplay-spec.md](docs/gameplay-spec.md)
- 设计：[docs/design-brief.md](docs/design-brief.md)
- 实现说明：[docs/prototype-01.md](docs/prototype-01.md)
- 目标架构：[docs/architecture.md](docs/architecture.md)
- 工作流：[docs/ai-workflow.md](docs/ai-workflow.md)

## 2. 技术栈（版本锁定，勿擅自升级）

| crate | 版本 | 备注 |
|---|---|---|
| bevy | 0.20.0 | MSRV 1.97.1 |
| avian3d | 0.8.0 | 对应 Bevy 0.20 |
| rand / rand_chacha | 0.10 | 尚未引入 |
| bevy_enhanced_input | 0.27.0 | 尚未引入（当前用原生输入） |

**不可引入**（仍锁 Bevy 0.19，会拉进第二份 Bevy）：
`bevy_hanabi 0.19`、`bevy_fps_controller 19`。
引入任何新 crate 前，先 `cargo tree -i bevy@0.19.1` 确认没有重复 Bevy。

## 3. 命令

```powershell
cargo run --offline                     # 运行原型
cargo run --offline -- --screenshot docs/images/prototype.png
cargo test -p vb_core                   # 规则单测
cargo build --offline
cargo fmt --check
cargo clippy --offline --workspace --all-targets -- -D warnings
```

- 依赖已缓存时**必须加 `--offline`**：本机 cargo 走代理，联网更新索引会卡住；
- 首次构建约 30 分钟（Bevy 依赖树），之后改 `vb_game` 约 25 秒。

## 4. 目录约定

```
crates/vb_core/         纯规则，禁止依赖 bevy
crates/vb_game/src/     Bevy 应用
  main.rs               插件组装
  level.rs              矿室 / 矿砖 / 材质 / 灯光 / R 重建
  player.rs             移动、视角、跳跃、光标
  weapon.rs             hitscan 射击
  mining.rs             矿石掉落与回收
  viewmodel.rs          枪械模型
  fx.rs                 曳光与命中点
  hud.rs                HUD
  dev.rs                截图 / 退出
  shared.rs             资源定义
assets/                 目前为空，关卡写死在 level.rs
docs/                   设计、规格、架构、原型说明
.agents/skills/         项目技能
```

- `vb_core` **禁止依赖 bevy**；
- 新功能优先新建模块/插件，不在 `main.rs` 堆逻辑；
- 跨模块共享的数据放 `shared.rs`，不要互相 import 内部实现。

## 5. 代码约定

- 文件 ≤ 300 行；单系统 ≤ 80 行；超过就拆。
- 命名：`XxxPlugin` / `XxxConfig` / `XxxQueue`；避免 `utils.rs`、`helpers.rs`。
- 玩法数值最终要进 `assets/data/*.ron`；当前阶段的临时常量集中在
  `vb_core`（规则）与 `level.rs`（摆放）。
- 表现层只消费状态，不得反向修改玩法组件。
- 每块矿砖**不要**创建独立材质；按 HP 档位复用共享材质。

## 6. Bevy 0.20 / Avian 0.8 事实（已核实）

- 事件流用 `Message` / `MessageWriter` / `MessageReader`（不是 `EventReader`）。
- 相机是 `Camera3d`；网格是 `Mesh3d` + `MeshMaterial3d`。
- 光标设置是独立组件 `CursorOptions`（不在 `Window` 上），`Window` 通过 `#[require]` 自动带上。
- 同一系统里同时访问 `&Transform` 与 `&mut Transform` 必须用 `Without<...>` 声明不相交，
  否则运行期 panic（B0001）。
- 字体大小是 `FontSize::Px(..)`，不是 `f32`。
- Avian 的角色控制器 `MoveAndSlide` 是**系统参数**，不是组件；`MoveAndSlideConfig` 也不是组件。
- Avian 0.8 的分层类型是 `LayerMask`（0.7 的 `CollisionLayer`/`Group` 已改名）。
- 碰撞事件在物理步进后的 `CollisionEventSystems` 集合触发；`CollisionStart` 同时是 `Message` 与 `EntityEvent`。
- Windows 下完整 debuginfo 会让可执行文件超过 2GB 而无法启动；
  `Cargo.toml` 已设 `debug = 0` + `strip = "debuginfo"`，**不要改回去**。
- 未确认的 API 一律先写探针 `cargo check`，再把结论写进 `docs/prototype-01.md` 或本节。

## 7. 完成定义（DoD）

1. `cargo fmt --check` 通过；
2. `cargo clippy --offline --workspace --all-targets -- -D warnings` 通过；
3. `cargo test --workspace` 通过；
4. `cargo build --offline` 成功，且二进制能启动（不是只编译通过）；
5. 视觉改动附一张 `--screenshot` 产物；
6. 更新受影响的文档（spec / prototype-01 / 本节）。

**未通过 1–4 的代码不允许说“完成”。**

## 8. 禁止

- 凭记忆写 Bevy/Avian API。
- 升级 Bevy 或 Avian 版本（需人类确认，并单独开一轮迁移）。
- 在 `Update` 里做物理模拟或扣血结算（物理相关逻辑优先 `FixedUpdate`）。
- 为单块矿砖创建独立材质。
- 一次性跨 3 个以上模块的重构而不先写计划。
- 把 `debug` 调回完整 debuginfo（会产出无法启动的 2GB exe）。

## 9. 文档索引

| 文件 | 作用 |
|---|---|
| `docs/design-brief.md` | 方向、缺口、下一步 |
| `docs/gameplay-spec.md` | 已实现的规则与数值 |
| `docs/prototype-01.md` | 代码地图、决策、踩坑 |
| `docs/architecture.md` | 目标架构 |
| `docs/ai-workflow.md` | 开发流程与验证阶梯 |
| `docs/specs/NNN-*.md` | 单次功能规格 |
| `docs/adr/NNNN-*.md` | 架构决策 |

## 10. 项目技能

| 技能 | 作用 |
|---|---|
| `vb-dev-loop` | 每轮开发的执行流程 |
