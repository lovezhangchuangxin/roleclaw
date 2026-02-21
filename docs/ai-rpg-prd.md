# AI 角色扮演游戏 PRD（Tauri2 + Rust + Vue3 + shadcn/vue）

## 0. 文档信息

- 文档版本：v1.0
- 更新日期：2026-02-21
- 目标版本：MVP（桌面端）
- 读者：产品、策划、前端、后端、测试

---

## 1. 目标与交付

### 1.1 项目目标

打造一款由 AI 大语言模型驱动的高自由度角色扮演游戏。区别于固定剧本 RPG，玩家可通过世界卡、角色设定和自由输入持续生成和塑造剧情，实现“无限分支 + 可控叙事 + 可回放”。

### 1.2 交付物

1. 一套可玩的 MVP 游戏主循环（新建存档 -> 世界生成 -> 地图探索 -> 对话推进 -> 事件触发 -> 存档回放）。
2. 一套可扩展系统骨架（模型接入、事件引擎、世界卡管理、存档系统）。
3. 一组可落地的接口与数据结构定义（Rust Command + TypeScript 类型）。

### 1.3 首版边界（MVP）

- 叙事驱动，不引入独立数值战斗系统。
- 支持 OpenAI 与 Claude 两类 API。
- 支持世界卡预设、导入导出。
- 支持地图节点探索与路径移动。
- 支持事件系统 v1 与快照+日志存档回放。

---

## 2. 产品定位与核心卖点

### 2.1 定位

“AI 原生叙事 RPG 编辑-游玩一体平台”：既是玩家游戏终端，也是世界构建工具。

### 2.2 核心卖点

1. 无限灵活：剧情非固定分支，玩家输入可即时改变走向。
2. 高自定义：世界卡、模型参数、角色身份可自由配置。
3. 可控生成：通过系统提示词分层和事件规则约束，保证世界观稳定。
4. 可回溯：每回合日志和状态 diff 可追踪、可回放、可分叉。

### 2.3 设计原则

1. 可控：优先确保设定一致性，避免 AI 漂移。
2. 可解释：系统状态变化对玩家可见、对开发可调试。
3. 可扩展：事件规则和世界卡可增量扩展。
4. 可复现：同配置+同种子可重演关键结果。

---

## 3. 用户与使用场景

### 3.1 目标用户

1. 叙事 RPG 玩家：重视代入感、剧情深度和角色互动。
2. 世界观创作者：希望低门槛搭建可游玩世界。
3. AI 玩家：愿意调教模型与提示词，追求个性化体验。

### 3.2 核心场景

1. 玩家用预设世界卡快速开档，沉浸式推进主线。
2. 玩家导入社区世界卡，体验不同风格剧本宇宙。
3. 玩家中途切换模型参数，优化叙事风格与稳定性。
4. 玩家回放关键节点，从历史回合分叉新剧情线。

---

## 4. 核心玩法循环（叙事驱动）

### 4.1 新建存档流程

1. 选择世界卡模板（可预览规则与风格）。
2. 配置玩家角色（身份、背景、初始关系标签）。
3. 选择模型 Provider 与参数（OpenAI/Claude，温度等）。
4. 触发世界生成，落地初始地点、NPC、剧情钩子。
5. 进入首回合。

### 4.2 回合循环

1. 读取当前场景上下文（地点、在场 NPC、任务状态、最近事件）。
2. 调用模型生成：
   - 当前叙事段落
   - 三个建议选项（推进剧情/情感互动/风险探索）
3. 玩家执行操作：
   - 选择三选项之一
   - 或输入自定义“第四选项”
4. 系统进行事件判定并执行动作。
5. 写入日志、更新快照，进入下一回合。

### 4.3 探索循环

1. 地图以“地点节点 + 路径边”构成。
2. 玩家只能沿可达边移动到邻接节点。
3. 到达地点时触发“地点进入事件”。
4. 与 NPC 互动时触发“交互事件”。

### 4.4 单回合时序（逻辑）

1. 输入聚合：玩家输入 + 当前世界状态 + 记忆摘要。
2. 生成阶段：模型返回叙事和选项。
3. 判定阶段：事件引擎扫描触发器并命中规则。
4. 执行阶段：应用状态变化（关系、地点、任务、变量）。
5. 持久化阶段：写回 snapshot 与 event log。

---

## 5. 世界生成与世界卡系统

### 5.1 世界卡定义

世界卡（World Card）本质是结构化 Prompt 模板，包含：

1. 世界规则：物理规律、魔法/科技边界、社会规则。
2. 题材风格：奇幻/科幻/悬疑等，叙事语气和文本风格。
3. 安全边界：禁忌内容、叙事限制、伦理策略。
4. 初始势力：阵营关系、冲突轴。
5. 地点池：候选地点模板及标签。
6. 角色原型池：NPC 原型、性格标签、动机模板。

### 5.2 世界生成产物

1. 世界观摘要（可展示给玩家）。
2. 主要 NPC 列表（性格、目标、秘密、关系初值）。
3. 地点图谱（节点和路径）。
4. 主线钩子和若干支线入口。
5. 初始事件集（至少包含地点进入与首次交互）。

### 5.3 多模板管理

1. 预设模板：内置官方卡。
2. 自定义模板：玩家创建/复制/编辑。
3. 导入导出：JSON 文件导入导出。
4. 版本机制：`schema_version` + `content_version`，保证兼容升级。

### 5.4 提示词分层策略

1. 系统层（最高优先级）：通用游戏规则、输出结构约束、安全边界。
2. 世界层：世界卡规则与设定。
3. 存档层：当前世界状态、人物关系、已发生关键事实。
4. 回合层：当前地点与玩家行为。

---

## 6. 对话系统（3+1）

### 6.1 选项机制

1. 选项 A：剧情推进型（目标导向）。
2. 选项 B：关系互动型（情感/社交）。
3. 选项 C：风险探索型（高收益高不确定）。
4. 选项 D：玩家自定义输入（与前三项同等权重）。

### 6.2 回包结构

每次对话响应包含：

1. `narration`：场景叙事文本。
2. `options`：三条候选选项。
3. `state_changes_preview`：状态变化摘要（可见层）。
4. `event_hints`：可能触发的事件提示（可配置隐藏或弱提示）。

### 6.3 对话记忆策略

1. 短期记忆：最近 N 轮原始对话。
2. 中期记忆：按章节摘要，定期压缩。
3. 长期事实库：不可违背事实（身份、地点解锁、关键关系）。

### 6.4 控制成本与稳定性

1. Token 预算按层分配，超长时优先压缩中期记忆。
2. 引入“事实锁”机制，减少设定自相矛盾。
3. 支持同回合重生成，不重复写事件日志。

---

## 7. 地图与移动系统（Canvas）

### 7.1 地图结构

1. 地点节点（Node）：`id/name/x/y/tags/npcs/events`.
2. 路径边（Edge）：`from/to/locked/conditions`.
3. 地图元信息：主题、缩放、层级标签（可扩展）。

### 7.2 移动规则

1. 仅允许邻接移动。
2. 锁定路径需满足条件后解锁（事件动作触发）。
3. 特殊事件可临时封锁路径，形成动态地图状态。

### 7.3 地点内容

1. 在场 NPC 列表。
2. 可互动对象（道具、机关、线索）。
3. 地点危险度与环境状态。
4. 可触发事件池。

### 7.4 交互反馈

1. 可达边高亮，禁行边置灰。
2. 悬浮显示地点摘要、任务关联和潜在风险标签。
3. 移动后弹出“到达事件卡片”。

---

## 8. 事件系统（可扩展核心）

### 8.1 事件分类

1. 地点进入事件（OnEnterLocation）。
2. NPC 互动事件（OnNpcInteraction）。
3. 物品与线索事件（OnAcquireItem/OnInspectClue）。
4. 关系阈值事件（OnRelationshipThreshold）。
5. 时间推进事件（OnTurnElapsed）。
6. 随机遭遇事件（OnRandomEncounter）。
7. 主线节点事件（OnMainQuestProgress）。

### 8.2 事件模型

`触发条件 -> 守卫条件 -> 动作列表 -> 后置效果 -> 冷却策略`

### 8.3 动作原语（Action）

1. `set_variable` / `inc_variable`
2. `unlock_location` / `lock_path`
3. `update_relationship`
4. `inject_quest` / `advance_quest_stage`
5. `spawn_npc` / `despawn_npc`
6. `append_log`

### 8.4 链式触发

1. 支持事件 A 触发事件 B。
2. 防循环机制：同回合最大链深限制 + 去重键。
3. 支持“延迟触发”（下一回合生效）。

### 8.5 调试能力

1. 记录触发源、命中条件、执行动作、失败原因。
2. 可导出事件执行轨迹，供开发定位逻辑问题。

---

## 9. 存档与回放

### 9.1 存档策略

采用“快照 + 事件日志”：

1. `snapshot.json`：当前完整状态，用于快速加载。
2. `events.ndjson`：每回合行为与状态变更，用于回放与分叉。

### 9.2 快照内容

1. 世界状态（地点、NPC、任务、变量）。
2. 玩家状态（身份、位置、资源/标签）。
3. 关系矩阵（玩家-NPC、NPC-NPC）。
4. 模型配置（provider、model、参数）。
5. 当前回合索引与时间戳。

### 9.3 日志内容

1. 输入：玩家选择/自定义文本。
2. 输出：叙事文本、选项、提示。
3. 事件：命中清单与动作结果。
4. diff：本回合状态变化摘要。

### 9.4 回放与分叉

1. 支持逐回合回看。
2. 支持从任一历史回合“另存为新存档”。
3. 分叉后继承历史日志并记录 `parent_save_id`。

---

## 10. AI 模型接入（OpenAI + Claude）

### 10.1 模型策略

1. 每个存档绑定一个主模型。
2. 游戏运行中可切换模型（保存切换历史）。
3. 切换后继续同一存档，不清空剧情状态。

### 10.2 Provider 适配层

统一内部请求接口，屏蔽差异：

1. 消息格式映射（system/user/assistant）。
2. 参数映射（temperature/max_tokens）。
3. 错误码统一化（超时、鉴权、频率限制）。

### 10.3 失败降级策略

1. 短超时 + 指数退避重试。
2. 重试失败后允许“手动重生成”。
3. 保证失败请求不污染状态日志。
4. 错误消息用户可读化（附建议操作）。

### 10.4 安全与密钥管理

1. API Key 本地安全存储（系统密钥链或加密存储层）。
2. 不将明文密钥写入存档或日志。
3. 请求日志对敏感字段脱敏。

---

## 11. 关键系统页面（IA）

### 11.1 主菜单

1. 新建游戏
2. 继续游戏
3. 存档管理
4. 设置

### 11.2 游戏主界面

1. 左侧：地图 Canvas（节点与路径）
2. 中部：叙事面板（场景描述、事件反馈）
3. 下部：3+1 选项输入区
4. 右侧：状态栏（任务、关系、地点信息）

### 11.3 世界卡管理

1. 模板列表与搜索
2. 编辑器（规则、地点池、角色池）
3. 导入/导出
4. 版本与兼容性提示

### 11.4 模型设置

1. Provider 选择（OpenAI/Claude）
2. 模型名称与参数
3. API Key 输入与连通性测试

### 11.5 游戏设置

1. 主题切换
2. 字体与缩放
3. 消息速度与动画选项
4. 日志级别

---

## 12. 主题与 UI 设计要点

### 12.1 主题方案（至少三套）

1. 沉浸幻想：羊皮纸质感、暖色光影。
2. 科幻终端：高对比、网格感、故障动效。
3. 古典档案：低饱和、书卷风、细线框架。

### 12.2 Design Token（建议）

1. 颜色：`--bg/--fg/--primary/--accent/--danger`
2. 圆角：`--radius-sm/md/lg`
3. 阴影：`--shadow-soft/hard`
4. 间距：`--space-1...--space-8`

### 12.3 反馈规则

1. 事件触发：统一 toast + 日志条目。
2. 关系变化：数值与标签双提示。
3. 任务推进：侧栏高亮 + 回合结算展示。

---

## 13. MVP 里程碑与范围拆解

### Milestone 1：世界生成与开档闭环

1. 世界卡读取与校验。
2. 模型配置与连通性测试。
3. 新建存档并生成初始世界。

### Milestone 2：地图与对话闭环

1. 地图渲染与移动判定。
2. 3+1 对话系统。
3. 回合推进和基础状态更新。

### Milestone 3：事件与存档回放

1. 事件引擎 v1（触发、守卫、动作、链式）。
2. 快照 + ndjson 日志。
3. 回放与分叉存档。

### Milestone 4：体验完善

1. 多主题与视觉优化。
2. 世界卡导入导出。
3. 稳定性与错误处理强化。

---

## 14. 公共接口与类型定义

### 14.1 Rust（Tauri Commands）

```rust
create_save(config) -> SaveMeta
load_save(save_id) -> SaveBundle
list_saves() -> Vec<SaveMeta>
delete_save(save_id) -> ()
generate_world(input) -> WorldInit
run_turn(turn_input) -> TurnResult
move_to_location(save_id, location_id) -> MoveResult
trigger_event(payload) -> EventResult
import_world_card(file) -> WorldCard
export_world_card(card_id, file) -> ()
test_model_provider(config) -> ConnectivityResult
```

### 14.2 TypeScript 核心类型（建议草案）

```ts
type Provider = "openai" | "claude";

interface ModelProviderConfig {
  provider: Provider;
  model: string;
  temperature: number;
  maxTokens: number;
  timeoutMs: number;
}

interface WorldRule {
  id: string;
  title: string;
  content: string;
  priority: number;
}

interface CharacterArchetype {
  id: string;
  name: string;
  traits: string[];
  motivation: string;
  secret?: string;
}

interface LocationNode {
  id: string;
  name: string;
  x: number;
  y: number;
  tags: string[];
  npcIds: string[];
  eventIds: string[];
}

interface PathEdge {
  id: string;
  from: string;
  to: string;
  locked: boolean;
  conditions?: string[];
}

interface WorldCard {
  id: string;
  name: string;
  schemaVersion: string;
  contentVersion: number;
  genre: string;
  tone: string;
  rules: WorldRule[];
  locationPool: LocationNode[];
  archetypePool: CharacterArchetype[];
}

interface DialogueOption {
  id: string;
  kind: "plot" | "emotion" | "risk" | "custom";
  text: string;
}

interface TurnInput {
  saveId: string;
  optionId?: string;
  customText?: string;
}

interface TurnResult {
  narration: string;
  options: DialogueOption[];
  stateChangesPreview: string[];
  eventHints: string[];
}

interface EventLogEntry {
  turn: number;
  timestamp: string;
  input: TurnInput;
  output: TurnResult;
  triggeredEventIds: string[];
  stateDiff: Record<string, unknown>;
}
```

### 14.3 事件相关类型（建议草案）

```ts
interface TriggerCondition {
  type: string;
  params: Record<string, unknown>;
}

interface GuardCondition {
  expr: string; // 例如 relationship.npc_001 >= 30
}

interface EventAction {
  type:
    | "set_variable"
    | "inc_variable"
    | "unlock_location"
    | "lock_path"
    | "update_relationship"
    | "inject_quest"
    | "append_log";
  params: Record<string, unknown>;
}

interface GameEvent {
  id: string;
  name: string;
  trigger: TriggerCondition;
  guards: GuardCondition[];
  actions: EventAction[];
  cooldownTurns?: number;
  nextEventIds?: string[];
}
```

### 14.4 存储结构

```text
/saves/<save_id>/meta.json
/saves/<save_id>/snapshot.json
/saves/<save_id>/events.ndjson
/world-cards/<card_id>.json
```

---

## 15. 测试与验收场景

### 15.1 功能验收

1. 新档生成：不同世界卡可成功生成，首回合可进入。
2. 对话闭环：三选项与自定义输入均可推进剧情并落日志。
3. 地图规则：不可跳跃至非邻接地点；到达触发进入事件。
4. 事件链：满足前置后按顺序触发，状态更新正确。
5. 存档恢复：重启后加载快照可继续，日志与状态一致。
6. 模型切换：同存档切换 OpenAI/Claude 后可继续游玩。
7. 导入导出：世界卡导入可用，导出文件可复用。
8. 异常处理：超时、鉴权失败、无网络提示明确且可恢复。

### 15.2 非功能验收

1. 回合响应：常规回合在可接受时延内返回（以产品指标定义）。
2. 稳定性：长时游玩（200+ 回合）无崩溃、无存档损坏。
3. 可调试性：事件执行轨迹可定位问题回合与规则。

---

## 16. 风险与对策

1. AI 幻觉导致设定漂移：
   - 对策：事实锁、分层提示词、关键事实校验。
2. 上下文过长导致成本和延迟上涨：
   - 对策：多层记忆摘要与 token 预算器。
3. 事件链复杂后难调试：
   - 对策：标准化动作原语、链深限制、执行轨迹导出。
4. 多 Provider 兼容问题：
   - 对策：适配层统一、错误码归一化、连通性测试。

---

## 17. 默认假设与已锁定决策

1. 单人离线优先，联网仅用于模型 API 调用。
2. 叙事驱动，不做独立数值战斗系统。
3. 文档与产品主语言为中文。
4. 首版优先桌面端（Tauri），不单独做移动端适配。
5. 模型策略为“单主模型 + 运行时可切换”。
6. 存档策略为“快照 + 事件日志”。
7. 先交付可玩和可扩展框架，再逐步扩展复杂玩法。

---

## 18. 附录：MVP 完成定义（DoD）

满足以下条件即视为 MVP 达成：

1. 玩家可从主菜单完成开档并进入游戏主界面。
2. 可执行“移动 -> 对话 -> 事件 -> 下一回合”的连续循环。
3. 世界卡可导入导出，且至少有 3 个可游玩模板。
4. OpenAI 与 Claude 均可完成基础回合请求。
5. 存档可读写、可回放、可分叉。
6. 主题切换可用，核心页面无阻塞级 UI 问题。

