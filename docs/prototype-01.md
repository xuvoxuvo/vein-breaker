# 原型 0.1 实现说明

> 状态：2026-10-26
> 对应代码：本仓库当前提交。玩法规则见 [gameplay-spec](./gameplay-spec.md)。

## 1. 文件地图

| 文件 | 职责 |
|---|---|
| `crates/vb_core/src/lib.rs` | 纯规则：矿种、HP、矿石价值、伤害结算 + 单测 |
| `crates/vb_game/src/main.rs` | App 组装：插件顺序、窗口、ClearColor |
| `.../level.rs` | 矿室、矿砖布局、材质、灯光、`R` 重建 |
| `.../player.rs` | 动态胶囊、WASD、跳跃、鼠标视角、相机同步、光标锁定 |
| `.../weapon.rs` | hitscan 射击、后坐、命中判定、换材质、生成矿石 |
| `.../mining.rs` | 矿石实体的弹道 + 磁吸 + 计数 |
| `.../viewmodel.rs` | 第一人称枪械模型（相机相对定位） |
| `.../fx.rs` | 曳光与命中点的 gizmo 队列 |
| `.../hud.rs` | 矿石/剩余矿砖、准星、操作提示 |
| `.../dev.rs` | `--screenshot` 截图模式、`Esc` 退出 |
| `.../shared.rs` | `Inventory`、`BricksLeft`、材质与资产句柄 |

## 2. 关键实现决策

| 决策 | 原因 | 何时替换 |
|---|---|---|
| 玩家用 `RigidBody::Dynamic` + 每帧设水平速度 | 原型阶段最快跑通；平坦地面足够 | 需要斜坡/台阶时换 Avian `MoveAndSlide` |
| hitscan 而非实体弹丸 | 先验证「瞄准→命中→破碎→回收」链路 | 手感稳定后再考虑 |
| 矿石用运动学而非刚体 | 数量多、寿命短，省碰撞对 | 需要被地形阻挡时再换 |
| 矿砖按剩余 HP 换 3 档共享材质 | 有「打裂了」的反馈，又不炸 draw call | 需要更细裂纹时换贴图/着色器 |
| 关卡写死在 `level.rs` | 避免早期被数据格式绑住 | 下一步迁 RON |
| 枪械用世界空间近似（非独立 view camera） | 实现简单 | 穿墙/遮挡问题明显时改 RenderLayers |

## 3. 踩过的坑（Bevy 0.20 / Avian 0.8）

1. **光标设置不在 `Window` 上**：0.20 把 `cursor_options` 拆成了独立的
   `CursorOptions` 组件（`Window` 通过 `#[require]` 自动带上），要查 `&mut CursorOptions`。
2. **查询冲突 B0001**：同一系统里 `&Transform`（玩家）与 `&mut Transform`（矿石/砖块）
   必须用 `Without<...>` 显式声明不相交，否则运行期 panic。
3. **Windows 下 debug 可执行文件超过 2GB 无法启动**：Bevy 依赖的完整 debuginfo
   让链接产物达到 ~2.4GB，Windows 报「不是有效的 Win32 应用程序」。
   已在 `Cargo.toml` 里设 `debug = 0` 与 `strip = "debuginfo"`。
4. **cargo 走代理会卡在索引更新**：依赖已缓存时用 `cargo build --offline`。
5. **射线从矿砖缝隙漏过去**：相邻矿砖的碰撞体正好共面，射线会从缝里穿到后墙。
   把碰撞体从 1.0 放大到 1.02（半宽 0.51）解决。

## 4. 运行

```powershell
cargo run --offline
# 或直接截图验证
cargo run --offline -- --screenshot docs/images/prototype.png
```

首次构建仍然要编译整棵 Bevy 依赖树（本项目实测 30 分钟量级，之后增量约 25 秒）。

## 5. 下一步重构点

1. 玩家控制器换成 Avian `MoveAndSlide`（运动学胶囊），支持斜坡与台阶；
2. 关卡与数值迁到 `assets/levels/*.ron` + `assets/data/tuning.ron`；
3. 加 `AppState`（Playing / Paused / Results）与撤离流程；
4. 音效与碎屑对象池；
5. 把 `dev.rs` 的截图逻辑拆成 `--screenshot` / `--bench` 两种模式。
