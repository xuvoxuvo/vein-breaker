# AI 开发工作流调研与建议

> 状态：v0.1（2026-10-26）
> 结论先行：**没有现成的「FPS + 打砖块 + Bevy」技能包**；但 Bevy 生态已经有一套成熟的
> Agent Skill 目录结构，可以低成本移植。真正决定成败的不是"找 skill"，而是
> **规格前置 + 小步验证 + 版本事实库**这三件事。

---

## 0. 结论（TL;DR）

1. **Skill 有，但都停在 Bevy 0.19。** 最完整的是 `nolantait/bevy-starter` 的
   `.agents/skills/`（24 个细分技能），目录格式与 DSH 完全兼容，但内容针对 0.19。
   直接用会在 `Message`/`Event`、BSN 语法等 0.20 变更上翻车，**建议选择性移植 + 逐条编译验证**。
2. **工作流不要上重型框架。** 单人/小团队 + AI，用「一页 spec → 一个垂直切片 →
   四级自动验证 → 一条 ADR」就够了。Spec Kit / BMAD 的价值在多人协作，对独立游戏是负担。
3. **架构上把"可验证性"当第一需求。** 纯逻辑放进无 Bevy 依赖的 crate，
   让 AI 能在秒级跑完单测，而不是每次改一行等 40 秒编译。
4. **最大的隐性成本是 Bevy API 漂移。** 模型的先验知识普遍停在 0.13–0.16，
   必须建立 `docs/bevy-0.20-api.md` 事实库，规则是"没编译过的 API 不许写进代码"。

---

## 1. 现有 Skill / 工作流盘点

### 1.1 技能库

| 来源 | 内容 | 与 DSH 兼容 | 版本状态 | 建议 |
|---|---|---|---|---|
| [nolantait/bevy-starter · `.agents/skills`](https://github.com/nolantait/bevy-starter/tree/master/.agents/skills) | 24 个 Bevy 细分技能：`bevy-rendering` `bevy-systems` `bevy-queries` `bevy-scenes` `bevy-bsn` `bevy-ui` `bevy-audio` `bevy-picking` `bevy-timers` `avian` 等 | ✅ 同构（`<name>/SKILL.md`） | Bevy **0.19**，Avian 0.7 | **主力参考**。挑 4–6 个移植到 0.20 |
| [gamedev-skills/awesome-gamedev-agent-skills](https://github.com/gamedev-skills/awesome-gamedev-agent-skills) | 67 个技能，按引擎/类型/流程组织：`other-engines/bevy-ecs`、`genres/*`、`workflows/prototype-fast`、`game-jam`、`steam-publish` | ✅ 同构 | 未标注版本，Bevy 只有 1 个浅层技能 | 借鉴 `prototype-fast` 的流程，Bevy 部分不必用 |
| [thedivergentai/Game-Dev-Agentic-Skills](https://github.com/thedivergentai/game-dev-agentic-skills) | Godot 深（99 技能 / 27 类型蓝图），Bevy 为 "docs-first parity" | ⚠️ 需转换 | Godot 为主 | 只看类型蓝图（breakout / wave defense 的拆解） |
| [mgrillo75/claude-code-game-studios](https://github.com/mgrillo75/claude-code-game-studios) | 49 个 agent + 72 个 workflow 的"工作室"编排 | ❌ Claude Code 专属 | — | 不适用；思路可参考 |
| [BMAD-METHOD](https://github.com/bmad-code-org/BMAD-METHOD) + game-dev module | 角色扮演式（PM/Architect/Dev/QA）多 agent 流程 | ⚠️ 重 | — | 独立游戏过重，不建议 |
| [GitHub Spec Kit](https://github.github.com/spec-kit/) | `/specify → /plan → /tasks → /implement` 的 spec-driven 工具链 | ✅ 通用 | 活跃 | **只借模板结构**，不必装 CLI |

**判断**：这些技能的价值在"压缩上下文"——把 Bevy 的领域知识写成 300–800 字的按需加载文档，
避免每次会话重新解释 ECS。但**没有任何一个理解你的游戏规则**，所以项目自有技能比外部技能更重要。

### 1.2 DSH 的 Skill 装载机制（已核实）

DSH 会扫描以下根目录，按 rank 合并（数字越小优先级越高）：

| Rank | 来源 | 路径 |
|---|---|---|
| 100 | 项目 `.dsh` | `<projectRoot>/.dsh/skills` |
| 200 | 项目 `.agents` | `<projectRoot>/.agents/skills` |
| 300 | 自定义 | 配置项 `customSkillDirs` |
| 400 | 用户 `.dsh` | `~/.dsh/skills` |
| 500 | 用户 `.agents` | `~/.agents/skills` |

关键规则：

- 只扫描**一层**：`<root>/<name>/SKILL.md` 或 `<root>/<name>.md`，嵌套的不会被发现；
- `SKILL.md` 需要 YAML frontmatter，必填 `name`（kebab-case）与 `description`；
- 增删改会自动热加载，不需要重启；
- 项目根 = 最近的包含 `.git` 的祖先目录，没有则用当前工作目录
  → **先 `git init`**，否则以后在子目录启动会话时技能会找不到；
- 技能可以被模型自动调用，也可以由你用 `/name` 显式调用。

因此本仓库采用 `.agents/skills/`，与 bevy-starter 同构，将来可整包同步上游。

### 1.3 建议自建的 5 个项目技能

| 技能 | 作用 | 何时建 |
|---|---|---|
| `vb-dev-loop` | 本工作流的可执行版本（spec → slice → verify） | **已建** |
| `bevy-0.20-api` | 事实库：只收录编译验证过的签名与坑 | M0 起持续追加 |
| `avian-3d-physics` | 角色控制器、碰撞层、SpatialQuery、CCD | M0 |
| `vb-game-rules` | 数值表、不变量、验收标准（AI 改数值前必读） | M1 |
| `vb-level-format` | RON 关卡 schema + 校验规则 | M2 |

> 原则：**技能是"随时可删的缓存"，不是文档**。过期就删，不要维护两套真相。

---

## 2. 推荐工作流：Spec → Slice → Verify → Record

### 2.1 分工

- **你**：定义体验、拍板数值、试玩、验收、决定"砍什么"。
- **AI**：拆解、实现、写测试、补文档、做迁移与重构。
- **共同**：每个切片结束复盘 10 分钟，把结论写回 `AGENTS.md` 或 ADR。

### 2.2 六步循环（一轮 0.5–2 天）

1. **Brief（你，10 分钟）**
   一句话说清这次要玩家感受到什么。例："让玩家第一次成功用身体挡回钻核时，有明确的音效与顿帧。"
2. **Spec（AI 起草，你改）**
   用 [`docs/specs/000-template.md`](./specs/000-template.md)，控制在 1 页。
   必含：验收标准（可测）、非目标（明确不做什么）、风险。
3. **Plan（AI，5 分钟）**
   列出要改的文件、顺序、测试点。你只审一件事：**有没有越界**（改到别的模块）。
4. **Implement（AI）**
   一次只做一个垂直切片，diff ≤ 400 行。超过就拆。
5. **Verify（AI + 你）**
   走 §3 的验证阶梯。L0–L3 必须全绿才交给你；L4 截图；L5 你试玩。
6. **Record（AI，2 分钟）**
   - 做了架构决策 → `docs/adr/NNNN-*.md`
   - 发现新 API 事实 → `docs/bevy-0.20-api.md`
   - 改了约定 → `AGENTS.md`
   - 调了数值 → `assets/data/tuning.ron` + 一句为什么

### 2.3 为什么不是"一句话让 AI 写完整个游戏"

| 一次性大改 | 小步切片 |
|---|---|
| 编译错误与逻辑错误混在一起 | 错误范围可控 |
| 上下文塞满 20 个文件 | 每次只读 3–5 个文件 |
| 手感问题无法定位到具体改动 | 每次只动一个变量 |
| 回滚成本高 | 回滚一个 commit |

Bevy 的编译反馈以十秒计，意味着**每轮迭代的边界必须小**，这不是风格问题，是物理约束。

---

## 3. 验证阶梯（本项目核心机制）

| 级别 | 检查什么 | 命令（M0 后生效） | 谁跑 | 目标耗时 |
|---|---|---|---|---|
| L0 | 能编译 | `cargo check --workspace --all-targets` | AI | 增量 < 20s |
| L1 | 风格与静态问题 | `cargo fmt --check` + `cargo clippy --workspace --all-targets -- -D warnings` | AI | < 60s |
| L2 | 纯逻辑正确 | `cargo test -p vb_core` | AI | < 5s |
| L3 | ECS 行为正确 | `cargo test -p vb_game --features headless` | AI | < 30s |
| L4 | 画面没坏 | `cargo run --features dev -- --level shaft_01 --seed 1 --screenshot` | AI/CI | < 90s |
| L5 | 好玩 | 3 分钟试玩清单（见下） | **你** | — |

L5 试玩清单（每次交付都跑）：

- [ ] 30 秒内能发射第一枚钻核
- [ ] 钻核能稳定弹跳，不穿墙、不卡角
- [ ] 掉进竖井时能明确知道"我失去了什么"
- [ ] 用身体挡回一次钻核，反馈清晰
- [ ] 清空最后一排时有结算
- [ ] 重开一局不需要读文档

> AI 无法替代 L5。任何"手感已调好"的结论都必须由人确认。

---

## 4. 上下文工程（让 AI 不跑偏的硬规则）

1. **`AGENTS.md` ≤ 150 行**，只放稳定事实：技术栈、命令、目录、约定、禁止项。
   每次会话都会注入，写多了等于每轮都在烧上下文。
2. **`docs/bevy-0.20-api.md` 是唯一 API 真相源**。规则：
   - 不许凭记忆写 Bevy API；
   - 不确定就写 10 行探针 `examples/probe_xxx.rs`，`cargo check` 通过后再落地；
   - 通过后把签名与坑记进事实库。
3. **文件 ≤ 300 行，一个 feature 一个目录**。AI 改错文件的最大原因是目录里什么都有一点。
4. **命名可 grep**：
   - 插件 `XxxPlugin`、系统集 `XxxSet`、事件 `OnXxx`、配置 `XxxConfig`；
   - 不要用 `utils.rs` / `helpers.rs` 这种黑洞文件名。
5. **跨模块只走显式接口**：`Message`（已发生的事实）或共享 `Resource`；
   禁止在 `brick` 系统里直接查 `player` 组件。
6. **每次会话结束回写**：新坑写事实库，新约定写 `AGENTS.md`，新决策写 ADR。
   不写就等于下一轮重新踩。

---

## 5. 并行与 DSH 特性

| 场景 | 用什么 | 注意 |
|---|---|---|
| 只读调研（API 对比、版本核查） | `subagent` 并行 | 便宜，适合 2–4 路 |
| 机械批处理（重命名、批量加 derive） | `workflow` | 不适合有耦合的逻辑改动 |
| 跨天的目标（"完成 M2 垂直切片"） | `create_goal` + `todo_write` | 目标要可判定完成 |
| 每日构建巡检 | `schedule_create` | 可选，单人项目通常不需要 |
| 代码审查 | `subagent_fork` + 只读 | 让它挑刺，不要让它改 |

**并发写规则**：`Cargo.toml`、`src/main.rs`、`AGENTS.md` 这类全局文件同一时间只能有一个 agent 改。

---

## 6. 编译速度（决定 AI 的迭代频率）

Bevy 首次编译是分钟级，增量是十秒级。能做的：

```toml
# Cargo.toml
[profile.dev]
opt-level = 1          # 自己的代码
[profile.dev.package."*"]
opt-level = 3          # 依赖全优化，运行不卡
```

- 开发时启用 `bevy/dynamic_linking`（显著减少链接时间，但要求 `cargo run` 一致）；
- **只 `cargo check`，不要 `cargo build`**；只有要试玩时才 build；
- 拆出无 Bevy 依赖的 `vb_core`，纯逻辑测试是毫秒级；
- Windows Defender 对 `target/` 的实时扫描会明显拖慢编译，可加排除项（自行评估安全风险）；
- 实测数据记录在 §8。

---

## 7. 里程碑与完成定义（DoD）

| 里程碑 | 目标 | 交付 | DoD |
|---|---|---|---|
| **M0 引导**（0.5–1 天） | 能编译、能验证、能描述 | workspace、锁定版本、空场景、CI 脚本、`AGENTS.md`、事实库、5 个技能 | `cargo run` 出现一个可移动相机；L0–L3 全绿 |
| **M1 走路骨架**（2–3 天） | 第一人称手感成立 | 移动/视角/碰撞、一个房间、射击命中砖块使其消失 | 能连续玩 60 秒不晕、不穿墙 |
| **M2 垂直切片**（4–6 天） | 完整循环成立 | 钻核弹跳/掉落/挡球、清场、重开、1 个手工关卡、HUD | L5 清单全过；一局 3–5 分钟 |
| **M3 内容**（1–2 周） | 有得玩 | 5 关、3 种砖、商店、音效、基础 VFX | 30 分钟不重复、不崩 |
| **M4 打磨**（1 周+） | 能发布 | 设置、存档、平衡、打包 | 干净机器上双击可玩 |

---

## 8. 实测与待验证

| 项 | 状态 | 数据 |
|---|---|---|
| Bevy 0.20 + Avian 0.8 依赖解析 | ✅ 已验证 | `cargo generate-lockfile` 成功，单一 Bevy 0.20.0 |
| Bevy 0.20 + Avian 0.8 编译 | ✅ 已验证 | 冷启动 `cargo check` **3m01s** 通过；增量 **<1s** |
| Avian 角色控制器 / 分层 / 碰撞事件 API | ✅ 已验证 | `MoveAndSlide` 是 SystemParam；分层用 `LayerMask` |
| `bevy_ci_testing` 截图回归 | ⏳ 待验证 | M0 时做 10 行探针 |
| `bevy_fps_controller` 0.20 适配 | ❌ 当前 19.0.0 锁 Bevy 0.19 | 先用其移动模型做参考实现 |
| `bevy_hanabi` 0.20 适配 | ❌ 当前 0.19.0 锁 Bevy 0.19 | MVP 用对象池粒子 |

---

## 9. 风险与对策

| 风险 | 早期迹象 | 对策 |
|---|---|---|
| Bevy API 漂移导致幻觉 | 编译错误集中在 `Event`/`Message`、`Camera3d`、BSN 语法 | 事实库 + 探针编译；不许凭记忆写 |
| 编译慢拖垮迭代 | 每次改动等 60s+ | 拆分 crate、`cargo check`、dynamic_linking |
| 范围蔓延 | spec 里出现"顺便做个…" | 非目标写进 spec；新想法进 backlog 不进行动项 |
| 手感调不出来 | 试玩 3 次仍说不清哪里不对 | 把"手感"拆成可测参数（速度、加速度、反弹角、镜头 FOV） |
| 美术资产拖垮进度 | 卡在找模型/贴图 | 全程用几何体 + 纯色材质到 M3，最后统一替换 |
| 单一 AI 会话上下文耗尽 | 改到一半丢失上下文 | 每轮结束提交 + 写 handoff 到 `docs/` |

---

## 10. 参考

- [Bevy 0.20 发布说明](https://bevy.org/news/bevy-0-20/) ｜ [0.19 → 0.20 迁移指南](https://bevy.org/learn/migration-guides/0-19-to-0-20/)
- [Avian Physics 0.8](https://joonaa.dev/blog/14/avian-0-8)
- [Bevy Starter（技能目录 + 插件结构）](https://github.com/nolantait/bevy-starter)
- [Bevy New 2D（项目骨架与设计文档）](https://github.com/TheBevyFlock/bevy_new_2d)
- [GitHub Spec Kit](https://github.github.com/spec-kit/)
- [Claude Agent Skills 规范](https://docs.claude.com/en/docs/agents-and-tools/agent-skills/overview)
