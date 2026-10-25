# Bevy 0.20 / Avian 0.8 API 事实库

> 本文件是**唯一 API 真相源**。规则：
> 1. 没在这里写过的 API，不许凭记忆用；
> 2. 每条必须标注验证等级与日期；
> 3. 不确定 → 写 10 行探针 → `cargo check` → 把结果抄进来。
>
> 为什么需要它：模型的先验知识普遍停在 Bevy 0.13–0.16，而 0.20 已经改了事件命名、
> BSN 语法、相机 API；Avian 0.8 也重命名了分层类型。凭记忆写 = 稳定产出编译错误。

## 验证等级

| 等级 | 含义 | 可信度 |
|---|---|---|
| **A** | 在本机探针工程 `cargo check` 通过 | 最高，可直接用 |
| **B** | 官方 0.20 发布说明 / 迁移指南明确写出 | 高，仍建议首次使用时确认 |
| **C** | 来自旧版文档或社区，未在本机验证 | 低，用前必须验证 |

探针工程：`%TEMP%\vb-spike`（Bevy 0.20.0 + Avian 0.8.0），2026-10-26 验证。

---

## 1. Bevy 核心（A 级，已编译）

```rust
use bevy::prelude::*;

// 3D 相机
commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
));

// 网格实体
commands.spawn((
    Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    MeshMaterial3d(materials.add(Color::srgb(0.8, 0.4, 0.2))),
    Transform::from_xyz(0.0, 5.0, 0.0),
));

// 光源
commands.spawn((PointLight::default(), Transform::from_xyz(4.0, 8.0, 4.0)));
```

- `Camera3d` 是组件，不再有 `Camera3dBundle`；
- `Mesh3d` / `MeshMaterial3d` 取代旧的 `PbrBundle` 字段；
- `Color::srgb(r, g, b)` 可直接转成 `StandardMaterial`。

## 2. Avian 0.8（A 级，已编译）

```rust
use avian3d::prelude::*;

App::new().add_plugins((DefaultPlugins, PhysicsPlugins::default()));

// 运动学角色
commands.spawn((
    RigidBody::Kinematic,
    Collider::capsule(0.4, 1.0),   // 半径, 长度
    TransformInterpolation,
    CollisionLayers::new(LayerMask(0b0001), LayerMask::ALL),
    CollisionEventsEnabled,
));

// 碰撞事件（消息方式）
fn on_hit(mut hits: MessageReader<CollisionStart>) {
    for hit in hits.read() {
        let _ = (hit.collider1, hit.collider2, hit.body1, hit.body2);
    }
}

// 空间查询
fn aim(spatial: SpatialQuery) {
    let filter = SpatialQueryFilter::default();
    let _hit = spatial.cast_ray(Vec3::ZERO, Dir3::X, 100.0, true, &filter);
}

// 角色控制器（系统参数，不是组件）
fn move_player(mut move_and_slide: MoveAndSlide) {
    let _ = &move_and_slide.colliders;
    // move_and_slide.move_and_slide(entity, ...) 见 §5
}
```

### 2.1 纠正：`MoveAndSlide` 不是组件

0.7 的社区文档里有 "insert `MoveAndSlide::default()`" 的写法。0.8 中它是
`#[derive(SystemParam)]`，无法 spawn。`MoveAndSlideConfig` 也只是函数参数结构体，
不是组件。正确用法是在系统里作为参数注入，然后显式调用。

### 2.2 纠正：分层类型是 `LayerMask`

| 0.7 写法 | 0.8 写法 |
|---|---|
| `CollisionLayer::from_bits(1)` | `LayerMask(0b0001)` 或 `LayerMask::from_bits(..)` |
| `CollisionLayers::new(Group::GROUP_1, ...)` | `CollisionLayers::new(LayerMask, LayerMask)` |

`CollisionLayers` 的字段是 `memberships` / `filters`（都是 `LayerMask`）。

### 2.3 碰撞事件时机

`CollisionStart` 在物理步进**之后**的 `CollisionEventSystems` 系统集中触发。
它同时派生 `EntityEvent` 与 `Message`：批量处理用 `MessageReader`，
针对单个实体用 `.observe(|e: On<CollisionStart>| ...)`。

## 3. Bevy 0.20 重要变更（B 级，官方发布说明）

| 变更 | 说明 |
|---|---|
| BSN 场景引用必须带 `@` | `@scene_fn()`、`@SceneComponent`、`@{expr}`；列表用 `--` 分隔实体 |
| `Ready` 事件 | 场景（含子树）完全生成后触发，适合"场景就绪再挂逻辑" |
| `despawn_all` | 批量反生成，比逐个快 1.3–1.7× |
| WESL 着色器 | 自定义着色器改用 `.wesl`；纯 WGSL 仍可用 |
| Mesh shader | 支持，但 Web 平台不可用 |
| `Val::Em` / `Val::Rem` | UI 支持相对字号；默认字号改为 `rem(1)` |
| `FixedNode` | UI 节点相对相机视口定位，不继承父布局 |
| `chain_weak` | 弱顺序，非冲突系统可并行 |
| 调度随机化 | `ScheduleBuildSettings { shuffle_seed }`，用于验证系统顺序假设 |
| panic 转错误 | 系统 panic 可被 `FallbackErrorHandler` 捕获，不再必然崩整个 App |
| 纹理压缩 | `compressed_image_saver` + 自动 mipmap |

## 4. 会话中被推翻的错误认知（保留，防止复发）

| 以为 | 实际 | 来源 |
|---|---|---|
| Avian 角色控制器 spawn `MoveAndSlide` 组件 | 是 SystemParam，必须显式调用 | A 级探针 |
| 分层用 `CollisionLayer` / `Group` | 0.8 改为 `LayerMask` | A 级探针 |
| 碰撞事件用 `EventReader` | 用 `MessageReader`（`CollisionStart` 也是 Message） | A 级探针 |
| `Camera3dBundle` / `PbrBundle` | 已移除，用组件元组 | A 级探针 |
| `bevy_hanabi 0.19` 可用于 Bevy 0.20 | 它锁 Bevy 0.19，会引入第二份 Bevy | 依赖树 |

## 5. 待验证（C 级，用前先探针）

| 项 | 验证方法 |
|---|---|
| `MoveAndSlide::move_and_slide` 的完整签名（entity、位移、config、callback） | 读 `avian3d-0.8.0/src/character_controller/move_and_slide.rs`，写最小调用 |
| `SweptCcd` / `SpeculativeCcd` 组件字段 | 写探针 spawn |
| `PhysicsDebugPlugin` 的启用方式 | 探针 + 跑起来看 gizmo |
| `TransformInterpolation` 是否需要额外插件 | 读 Avian `PhysicsPlugins` 文档 |
| `bevy_enhanced_input 0.27` 的 action/context 宏写法 | 探针 + 官方 examples |
| `bevy_asset_loader 0.28` 的 `loading_state` 派生宏 | 探针 |
| `bevy_ci_testing` 截图回归 | 启用 feature，跑 10 帧出图 |
| `App::new() + MinimalPlugins` 做无头 ECS 测试 | 探针：spawn 实体 → `app.update()` → 断言 |

## 6. 记录模板

新增一条时复制：

```markdown
### <主题>

- 等级：A / B / C
- 日期：YYYY-MM-DD
- 版本：bevy x.y.z / avian3d x.y.z
- 来源：探针工程 / 发布说明链接
- 代码：
  ```rust
  // 最小可编译片段
  ```
- 坑：<一句话>
```
