# 架构设计：Vein Breaker

> 状态：v0.1（2026-10-26）
> **实现状态**：已有一个可运行的**射击原型**（Prototype 0.1），实际代码结构与本文的差异
> 见 [原型说明](./prototype-01.md)。本文保留为「目标架构」，§2、§6.2 已按射击方向更新；
> §6.3、§7 中标注「目标」的部分尚未实现。
> 前置阅读：[设计简报](./design-brief.md)、[AI 工作流](./ai-workflow.md)
> 目标：给出一个"能落地、能被 AI 验证、能撑到 20 关"的骨架，而不是教科书式的分层图。

---

## 1. 技术选型（已做依赖解析验证）

用 `cargo generate-lockfile` + `cargo tree` 在临时探针工程里验证过版本关系：

| 层 | 选择 | 版本 | 与 Bevy 0.20 兼容 | 说明 |
|---|---|---|---|---|
| 引擎 | [Bevy](https://bevy.org/news/bevy-0-20/) | 0.20.0 | — | MSRV 1.97.1，本机 1.99.1 |
| 物理 | [Avian](https://joonaa.dev/blog/14/avian-0-8) | 0.8.0 | ✅ | 官方声明支持 0.20；角色控制器 + CCD + SpatialQuery 齐全 |
| 输入 | [bevy_enhanced_input](https://crates.io/crates/bevy_enhanced_input) | 0.27.0 | ✅ | 动作映射、上下文、手柄；比裸 `ButtonInput` 更适合"射击/交互/暂停"多上下文 |
| 资产加载 | [bevy_asset_loader](https://crates.io/crates/bevy_asset_loader) | 0.28.1 | ✅ | 按状态加载、进度条；避免手写资源加载状态机 |
| 随机 | `rand` + `rand_chacha` | 0.10 / 0.10 | ✅ | 固定种子 = 可复现、可测试 |
| 序列化 | `serde` + `ron` | 1.0 / 0.12 | ✅ | 关卡与数值表 |
| 粒子 | bevy_hanabi | 0.19.0 | ❌ 仍锁 Bevy 0.19 | **MVP 不用**，用对象池碎片；等 0.20 版本 |
| FPS 控制器 | bevy_fps_controller | 19.0.0 | ❌ 仍锁 Bevy 0.19 | **不直接依赖**；参考其 Source 风格移动模型，自己实现 |
| 调试面板 | bevy-inspector-egui | 0.38.0 | ⏳ 待验证 | M0 用探针验证后再加 |

> 教训：Bevy 生态的 crate 版本号经常跟着 Bevy 走（`bevy_hanabi 0.19` 对应 Bevy 0.19），
> **看到版本号先查它依赖哪个 bevy**，不要看主版本号猜。

**验证状态（2026-10-26，临时探针工程）**

- `cargo generate-lockfile`：解析出唯一的 `bevy 0.20.0`（混入 hanabi 时会出现 0.19.1，已排除）；
- `cargo check`（Bevy + Avian，最小 3D 场景）：**冷启动 3 分 01 秒，通过**；
- 增量 `cargo check`（只改游戏 crate）：**< 1 秒**；
- 探针确认可编译：`RigidBody::Kinematic`、`Collider::capsule`、`TransformInterpolation`、
  `CollisionLayers::new(LayerMask, LayerMask)`、`CollisionEventsEnabled`、
  `MessageReader<CollisionStart>`、`SpatialQuery::cast_ray`、`MoveAndSlide`（SystemParam）。

**探针纠正的两处错误认知**（正是为什么要建事实库）：

1. Avian 0.8 的分层类型是 `LayerMask`，**不是** 0.7 的 `CollisionLayer` / `Group`；
2. `MoveAndSlide` 是 **SystemParam**，不是可 spawn 的组件；`MoveAndSlideConfig` 也不是组件。

### 1.1 为什么是 Bevy

| 引擎 | 3D 能力 | 编辑器 | ECS | AI 可读性 | 结论 |
|---|---|---|---|---|---|
| **Bevy 0.20** | 强（PBR、Solari 光追、WESL） | 无正式编辑器（BSN/Feathers 在演进） | 原生 | 高：全是 Rust 代码，没有二进制场景 | ✅ 选它 |
| Fyrox | 中 | 有 | 有自己的 ECS | 中：编辑器产出的场景不利于 diff | 备选 |
| macroquad | 弱（3D 基础） | 无 | 无 | 高 | 不适合 3D FPS |
| three-d | 中 | 无 | 无 | 中 | 生态较小 |
| Godot + gdext | 强 | 强 | 节点树 | 中：GDScript/GDExtension 混合 | 非 Rust 原生，偏离需求 |

**Bevy 的核心优势恰好是 AI 协作的核心需求**：场景即代码（可 diff、可生成）、
数据与行为分离（系统可单测）、插件即模块边界（改一个 feature 不碰别的）。

**代价**：编译慢、API 每个版本都在动、没有可视化编辑器调关卡。前两条用工作流对冲，
第三条用 RON 关卡格式对冲。

### 1.2 为什么是 Avian 而不是 Rapier

| | Avian 0.8 | bevy_rapier3d 0.37 |
|---|---|---|
| 与 Bevy 的集成 | 原生 ECS，组件即物理属性 | 封装 Rapier，需要同步层 |
| 角色控制器 | `MoveAndSlide`（运动学） | `KinematicCharacterController` |
| 弹球需求（高弹性、CCD、低摩擦） | 支持，0.8 重写了 CCD，专治高速穿透 | 支持 |
| 调试渲染 | 内置 gizmos | 内置 |
| AI 友好度 | 组件名直白、文档集中 | API 面更大 |

两者都能做。选 Avian 的理由是**组件化更彻底**，AI 改一个属性不需要理解插件内部同步逻辑；
且 0.8 的 swept CCD 正好解决"钻核 40 m/s 穿墙"这个必然会遇到的问题。

---

## 2. 运行时结构

```
main()
 └─ App
    ├─ DefaultPlugins           窗口 / 渲染 / 资产 / 输入 / 音频 / UI
    ├─ PhysicsPlugins           Avian（固定步长）
    ├─ EnhancedInputPlugin      动作映射
    ├─ CorePlugin               States / Config / RNG / Save
    ├─ GameplayPlugins
    │   ├─ PlayerPlugin         移动、视角、碰撞体
    │   ├─ WeaponPlugin          hitscan 射击、后坐、命中判定
    │   ├─ ViewModelPlugin       第一人称枪械模型
    │   ├─ BrickPlugin          砖块网格：生成、受伤、破碎
    │   ├─ MiningPlugin         矿石掉落、磁吸、计数
    │   └─ LevelPlugin          关卡加载/卸载、胜负判定
    ├─ PresentationPlugins
    │   ├─ CameraPlugin         相机 rig、FOV、抖动
    │   ├─ FxPlugin             碎片、火花、命中反馈
    │   ├─ HudPlugin            准星、计数、提示
    │   └─ AudioPlugin          SFX 池、混音
    └─ DevPlugins               仅 feature = "dev"：调试面板、gizmos、控制台
```

### 2.1 状态机

```
Boot ──► MainMenu ──► Playing ──► LevelCleared ──► Playing（下一关）
                        │  ▲
                        ▼  │
                      Paused
                        │
                        ▼
                     GameOver ──► MainMenu
```

- `Playing` 内部不再切子状态，用资源 `RunState` 表达"进行中/结算中"，避免状态爆炸；
- 关卡加载放在 `OnEnter(Playing)`，卸载放在 `OnExit(Playing)`，用 `LevelRoot` 标记整棵实体树。

### 2.2 数据流（单向）

```
输入(EnhancedInput) → 意图(Intent 组件/消息) → 模拟(物理+规则) → 事实(Message) → 表现(音画/UI)
```

表现层**只读事实**，不反向改模拟。这条约束能让"加特效"永远不破坏玩法逻辑。

---

## 3. 代码结构

```
vein-breaker/
├─ Cargo.toml                 # workspace
├─ rust-toolchain.toml
├─ .cargo/config.toml         # 编译优化 / 链接器
├─ .agents/skills/            # 项目技能（DSH 自动发现）
├─ crates/
│  ├─ vb_core/                # 纯逻辑，无 Bevy 依赖
│  │  └─ src/
│  │     ├─ lib.rs
│  │     ├─ grid.rs            # GridPos、索引、邻接
│  │     ├─ brick.rs           # BrickKind、HP、颜色分档
│  │     ├─ damage.rs          # 伤害结算、穿透
│  │     ├─ economy.rs         # 矿石、升级价格
│  │     ├─ leveldef.rs        # RON 反序列化 + 合法性校验
│  │     └─ rng.rs             # ChaCha8 包装、可复现
│  └─ vb_game/                # Bevy app
│     └─ src/
│        ├─ main.rs            # 只做 App 组装
│        ├─ app.rs             # 插件注册表
│        ├─ states.rs          # AppState / RunState
│        ├─ shared/            # 跨 feature 的组件、消息、工具
│        │  ├─ components.rs   # Transform 同步、DespawnOnExit 等
│        │  └─ messages.rs     # 全局事实
│        ├─ features/
│        │  ├─ player/{mod,components,systems,plugin}.rs
│        │  ├─ drill/{...}
│        │  ├─ bricks/{...}
│        │  ├─ mining/{...}
│        │  ├─ level/{...}
│        │  ├─ hud/{...}
│        │  └─ fx/{...}
│        └─ dev/               # feature-gated
├─ assets/
│  ├─ levels/*.ron
│  ├─ data/{tuning,bricks,upgrades}.ron
│  ├─ models/ textures/ audio/ fonts/
└─ docs/
```

### 3.1 为什么拆 `vb_core`

| 方案 | 单测速度 | 强制边界 | 编译增量 | 适合 AI |
|---|---|---|---|---|
| 单 crate，全用 Bevy 类型 | 需要 App/World，慢 | 无 | 改一行重编整个 crate | 一般 |
| `vb_core` 无 Bevy 依赖 | 毫秒级 | 强（编译器强制） | 只重编 core | ✅ |

`vb_core` 只用 `glam`（Bevy 也用同一个版本）、`serde`、`rand_chacha`、`thiserror`。
它承载：网格与关卡解析、伤害/经济规则、关卡合法性校验、RNG。
**这些恰好是最容易写错、也最值得测的部分。**

> 注意：不要为了"纯粹"把 ECS 逻辑也搬进 core。core 只放**纯函数与数据**，
> 任何需要实体生命周期的东西留在 `vb_game`。

### 3.2 feature 目录的固定形状

```rust
// features/bricks/mod.rs
mod components;
mod systems;
mod plugin;

pub use components::*;
pub use plugin::BrickPlugin;
```

每个 feature 只暴露：插件 + 必要的组件/消息。系统一律 `pub(crate)` 或私有。
这样 AI 改 `bricks` 时，能改到的范围是可预测的。

---

## 4. 核心数据模型

### 4.1 组件

| 组件 | 关键字段 | 归属 | 说明 |
|---|---|---|---|
| `Player` | — | player | 标记 |
| `Look` | `yaw: f32, pitch: f32` | player | 视角，pitch 夹在 ±89° |
| `MoveConfig` | `speed, accel, air_control, friction` | player | 从 `tuning.ron` 读 |
| `PlayerBody` | — | player | 运动学胶囊，Avian `MoveAndSlide` |
| `DrillCore` | `state: DrillState, owner: Entity` | drill | 状态机 |
| `DrillConfig` | `speed, restitution, min_speed, max_speed` | drill | 可被升级修改 |
| `Brick` | `kind: BrickKind, hp: i16` | bricks | 可破坏砖 |
| `GridPos` | `x, y, z: u16` | bricks | 逻辑坐标，用于关卡判定与存档 |
| `OreDrop` | `value: u32` | mining | 掉落物 |
| `Magnet` | `radius: f32` | mining | 磁吸范围 |
| `LevelRoot` | — | level | 整关父实体，卸载用 |
| `HudValue` | `kind: HudKind` | hud | 数值绑定，不每帧格式化字符串 |

### 4.2 资源

| 资源 | 内容 | 生命周期 |
|---|---|---|
| `Tuning` | 全部可调数值（RON 加载） | 全局 |
| `BrickCatalog` | 砖类型表（HP、颜色、掉落） | 全局 |
| `LevelCatalog` | 关卡清单 | 全局 |
| `Run` | `ore, cores_left, bricks_left, level_idx, elapsed` | 每局 |
| `GameRng` | `ChaCha8Rng`（种子来自 `--seed` 或时间） | 每局 |
| `Intent` | 本帧输入意图（发射/召回/交互） | 每帧清空 |

### 4.3 消息（已发生的事实）

| 消息 | 负载 | 谁发 | 谁收 |
|---|---|---|---|
| `DrillFired` | `entity, origin, dir` | drill | audio, fx |
| `DrillBounced` | `pos, normal, speed` | drill | audio, fx, camera |
| `DrillLost` | `reason` | drill | run, hud, audio |
| `BrickDamaged` | `brick, hp_left` | bricks | fx, audio |
| `BrickBroken` | `brick, kind, pos` | bricks | mining, score, fx |
| `OreCollected` | `value, total` | mining | hud, audio |
| `LevelCleared` | `time, cores_left` | level | ui |
| `RunFailed` | `reason` | run | ui |

约定：
- **`Message` 用于高频、跨系统的事实流**（碰撞、破碎、计分）；
- **`EntityEvent` 用于"针对某个实体"的一次性事件**（如 `OnHit`），用 observer 处理；
- 消息在 `FixedUpdate` 产生，在 `Update` 消费也没问题（双缓冲），
  这样**不依赖同帧系统顺序**，是比"精确排系统"更稳的做法。

### 4.4 状态

```rust
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState { #[default] Boot, MainMenu, Playing, Paused, LevelCleared, GameOver }
```

`RunState` 用资源而非状态：`{ phase: RunPhase, ore, cores_left, ... }`，
因为"结算中"和"游戏中"共享大量实体，切状态会触发不必要的 despawn。

---

## 5. 调度顺序

固定步长（60 Hz）里，逻辑顺序：

```
1. read_intent        收集输入 → Intent
2. player_move        设置运动学速度（MoveAndSlide）
3. drill_update       发射 / 召回 / 速度钳制 / 防卡死
4. [Avian 物理步进]   碰撞、CCD、接触
5. resolve_contacts   读 CollisionStart → 发 BrickDamaged / DrillBounced
6. apply_damage       HP 扣减、破碎判定
7. spawn_rewards      矿石、碎片
8. check_end          清场 / 失败判定
9. cleanup            超时实体、消息清理
```

要点：

- **命中结算不要和物理同帧强绑**。用 `CollisionStart` 消息，在下一帧处理也正确，
  这样不必精确知道 Avian 的物理 SystemSet 名称，升级 Avian 时不会崩。
- 表现层（相机、HUD、粒子）全部放 `Update`，用 `Time<Real>` 的 delta 做插值。
- 物理与渲染之间开 `TransformInterpolation`（Avian 提供），否则高刷屏会看到抖动。

> Avian 0.8 在物理步进结束后的 `CollisionEventSystems` 集合里触发 `CollisionStart`
> （源码文档确认）。它同时派生 `EntityEvent` 与 `Message`，两种消费方式都可用。
> 其余 SystemSet 名称以 `cargo doc` 为准，M0 时补进 `docs/bevy-0.20-api.md`。

---

## 6. 三个核心机制

### 6.1 第一人称控制器

**方案**：运动学刚体 + 胶囊 + Avian `MoveAndSlide`。

- 身体：`RigidBody::Kinematic` + `Collider::capsule(0.4, 1.0)`；
- 移动：在自己的系统里把 `MoveAndSlide` 作为**系统参数**使用（它不是组件，不能 spawn），
  调用 `move_and_slide(...)` 传入期望位移与 `MoveAndSlideConfig`；
  它负责扫掠、贴墙滑行、去穿透——这三件事自己写极容易出 bug；
- 视角：yaw 转身体，pitch 转相机，`pitch.clamp(-89°, 89°)`；
- 相机：`Camera3d` 作为子实体，偏移 `(0, 1.6, 0)`；
- 移动：加速度模型（不是直接设速度），地面/空中不同加速度；
- 不做（v0）：跳跃、蹲、滑铲、攀爬 —— 但 `MoveConfig` 预留字段。

**为什么不直接依赖 `bevy_fps_controller`**：它当前锁 Bevy 0.19。它的价值是移动模型
（Source 风格的加速、空中控制、斜坡处理），可以读代码移植，而不是当作依赖。

**验收**：在 1m 宽的走廊里贴着墙走不抖；斜面上速度不突变；60 秒不晕。

### 6.2 射击（hitscan）

```
按下左键 ──冷却就绪──► 从相机中心发射射线
                          ├─命中矿砖 ──► 扣 HP ──► 换材质档位
                          │                └─HP=0 ──► 销毁 + 生成矿石 + 计数
                          ├─命中基岩 ──► 只出命中反馈
                          └─未命中 ──► 曳光到最远距离
```

实现要点：

- 射线起点用相机位置，方向用 `Transform::forward()`，最远 120m；
- 过滤掉玩家自身：`SpatialQueryFilter::default().with_excluded_entities([player])`；
- 命中实体与矿砖实体**是同一个**（矿砖的 `Collider` 与 `RigidBody` 同实体），
  所以 `bricks.get_mut(hit.entity)` 可以直接拿到 `Brick`；
- 相邻矿砖碰撞体互相重叠 2cm，避免射线从共面缝隙漏到后墙；
- 后坐：每发增加一个衰减的 pitch 偏移，由相机同步系统叠加；
- 曳光与命中点用 `Gizmo` 队列绘制（寿命 0.07–0.12s），不生成实体；
- 射速用资源里的冷却计时，而不是 `Timer`——原型只需要一个数。

**为什么先做 hitscan**：要验证的是「瞄准→命中→破碎→回收」这条链路是否好玩。
实体弹丸会引入飞行时间、重力、穿透等额外变量，等链路成立后再加。

### 6.3 砖块系统（目标：当前为简化实现）

当前实现：布局写死在 `level.rs`，矿砖 HP/价值来自 `vb_core`，破坏时直接 `despawn` 并生成矿石，
尚未使用消息或 RON。下面是目标形态。

- 关卡 RON → `LevelDef` → 校验 → 生成 `Brick` 实体（静态碰撞体）；
- 所有砖共享一个 `Mesh`（1m 立方），材质按 `BrickKind` 复用（**不要每块砖一个材质**，
  否则 draw call 爆炸）；
- HP 用颜色/发光强度表达，不做血条；
- 破碎：发 `BrickBroken` → 生成矿石与碎片 → `despawn`；
- 清场用 `despawn_all`（Bevy 0.20 新增的批量反生成），比逐实体 despawn 快 1.5–1.7 倍；
- 不可破坏的边框砖单独 `BrickKind::Bedrock`，不参与胜利判定，只负责反弹。

**性能**：2000 块静态砖对 Avian 0.8 是小场景（它能处理百万级静态体）。
真正的瓶颈是 draw call 与碎片，不是物理。

---

## 7. 关卡数据格式（目标）

> 当前布局写死在 `crates/vb_game/src/level.rs` 的 `WALL_PATTERN`，改一次要重编译。
> 下面是迁移目标。

`assets/levels/shaft_01.ron`：

```ron
(
    id: "shaft_01",
    name: "一号竖井",
    kill_plane_y: -2.0,
    spawn: (position: (6.0, 1.0, 6.0), yaw: 180.0),
    grid: (
        origin: (0.0, 2.0, 0.0),
        cell: 1.0,
        rows: [
            "BBBBBBBBBBBB",
            "B..........B",
            "B..OO..OO..B",
            "B..OO..OO..B",
            "B..........B",
            "BBBBBBBBBBBB",
        ],
    ),
    legend: {
        'B': "bedrock",
        'O': "ore_copper",
        '.': "empty",
    },
)
```

`assets/data/bricks.ron`：

```ron
{
    "bedrock":    (hp: -1, color: (0.25, 0.25, 0.28), drop: 0, indestructible: true),
    "ore_copper": (hp: 2,  color: (0.72, 0.35, 0.16), drop: 2),
    "ore_iron":   (hp: 3,  color: (0.55, 0.58, 0.62), drop: 3),
    "unstable":   (hp: 1,  color: (0.85, 0.55, 0.15), drop: 1, explosive: true),
}
```

为什么用 RON：

- 与 Rust 类型一一对应，`serde` 直接反序列化，改关卡不用改代码；
- 纯文本可 diff、可由 AI 生成、可写脚本校验；
- Bevy 资产系统支持热重载，调关卡时不用重启。

校验放在 `vb_core::leveldef::validate`：行宽一致、字符都在 legend 里、
至少有一块可破坏砖、出生点不在砖块里、kill plane 在网格下方。

---

## 8. 测试策略

| 层 | 测什么 | 怎么做 | 速度 |
|---|---|---|---|
| 单元 | 网格解析、伤害、经济、RNG | `vb_core` 内 `#[test]` | ms |
| 系统 | 一个系统在给定 World 下的行为 | `App` + `MinimalPlugins`，`app.update()` 后断言 | 10–100ms |
| 集成 | "开一枪 → 砖块 HP 减 1" | 无渲染 App + 手动插入实体，跑 N 帧 | <1s |
| 确定性 | 同种子跑 600 帧状态一致 | 记录 `(brick_count, ore, ball_pos)` 哈希 | <1s |
| 视觉 | 关键画面不回归 | `bevy_ci_testing` 截图（M0 验证可行性） | 秒级 |

**无头测试的关键**：玩法系统不能依赖渲染插件。用 `MinimalPlugins` + 手动插入
`Assets<Mesh>` 等必要资源，或把玩法逻辑写成不碰渲染类型的系统。
这也是 §3.1 拆 `vb_core` 的另一个理由。

---

## 9. 性能预算

| 项 | 预算 | 手段 |
|---|---|---|
| 分辨率/帧率 | 1080p / 60fps（GTX 1060 级） | 关掉不必要后处理 |
| 砖块数 | ≤ 2000 / 关 | 共享 mesh + 按种类复用材质 |
| 动态刚体 | ≤ 200 | 矿石合并、碎片对象池 |
| 粒子 | ≤ 300 同时 | 对象池，不用 hanabi（0.20 未就绪） |
| Draw call | < 500 | 同材质合批；避免每砖独立材质 |
| 每帧分配 | 接近 0 | HUD 只在变化时更新字符串；复用 `Vec` |
| 阴影 | 1 个方向光 / 2048 shadow map | 矿灯用自发光 + 少量点光 |
| MSAA | 4x | 需要时降到 2x |

MVP 阶段**不做**：SSAO、体积雾、光追（Solari）、DLSS。这些在 0.20 都能开，
但会让"是不是玩法的问题"变得无法判断。

---

## 10. 调试与可观测性

| 工具 | 用途 | 何时做 |
|---|---|---|
| `F1` 调试 HUD | FPS、实体数、球速、状态、当前关卡 | M1 |
| `F2` 物理 gizmo | 碰撞体、射线、接触点 | M1 |
| `--seed N` | 复现随机 | M1 |
| `--level ID` | 直达某关 | M1 |
| `--god` | 不掉球 / 无限矿石 | M2 |
| `bevy-inspector-egui` | 运行时改组件值 | M2（验证兼容后） |
| `tracing` span | 每个 SystemSet 的耗时 | M3 |
| 内置控制台 | `spawn_brick`、`clear_level` | M3 |

对 AI 来说，**能自证**比"看起来对"重要：调试 HUD 上的数字就是它的验收依据。

---

## 11. 扩展点（留好口子，不提前实现）

- **武器**：`Launcher` 组件 + `ProjectileSpec`（RON），支持多发射器；
- **砖块行为**：`BrickKind` 加枚举变体（爆炸、移动、再生），行为用 observer 挂；
- **关卡修饰**：`LevelModifiers { gravity_scale, ball_speed_mul, fog }`；
- **元进度**：`Run` 与 `Profile` 分离，后者落盘；
- **多人**：不预留。ECS + 消息的架构天然可拆，但提前抽象会拖慢 MVP。

---

## 12. 决策记录（ADR）

| 编号 | 决策 | 理由 | 替代方案 |
|---|---|---|---|
| ADR-0001 | Bevy 0.20 + Avian 0.8 | 原生 ECS、3D 能力、CCD、AI 可读性 | Fyrox、Rapier |
| ADR-0002 | workspace 拆 `vb_core` / `vb_game` | 毫秒级测试、编译边界 | 单 crate |
| ADR-0003 | 事件驱动 + FixedUpdate | 不依赖系统顺序、可无头测试 | 直接查询/精确排序 |
| ADR-0004 | MVP 不用 hanabi / bevy_fps_controller | 二者仍锁 Bevy 0.19 | 从 git 依赖（风险高） |
| ADR-0005 | RON 关卡 + 热重载 | 可 diff、可生成、可校验 | 硬编码 / 编辑器 |
| ADR-0006 | 原型期用动态刚体 + 速度控制代替 `MoveAndSlide` | 先跑通射击链路，平坦地面足够 | 需要斜坡/台阶时替换 |

---

## 13. 里程碑映射

| 里程碑 | 架构上要完成的事 |
|---|---|
| M0 | workspace、`CorePlugin`、`AppState`、空场景、L0–L3 脚本 |
| M1 | `PlayerPlugin`、`CameraPlugin`、`BrickPlugin` 最小版、射击命中 |
| M2 | `WeaponPlugin` 完整射击手感、`LevelPlugin`、`MiningPlugin`、HUD、清场与重开 |
| M3 | `FxPlugin`、`AudioPlugin`、多关卡、商店 |
| M4 | 设置/存档/打包、性能调优 |

---

## 14. 参考

- [Bevy 0.20](https://bevy.org/news/bevy-0-20/) ｜ [迁移指南](https://bevy.org/learn/migration-guides/0-19-to-0-20/)
- [Avian Physics 0.8](https://joonaa.dev/blog/14/avian-0-8) ｜ [Avian 仓库](https://github.com/avianphysics/avian)
- [bevy_fps_controller](https://github.com/qhdwight/bevy_fps_controller)（移动模型参考，0.19）
- [bevy-starter](https://github.com/nolantait/bevy-starter)（插件结构参考）
- [Bevy New 2D 设计文档](https://github.com/TheBevyFlock/bevy_new_2d/blob/main/docs/design.md)（项目骨架参考）
