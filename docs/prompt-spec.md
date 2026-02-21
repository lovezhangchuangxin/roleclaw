# AI RPG Prompt 规范（系统提示词分层与模板）

## 0. 目标

本规范用于统一 `generate_world` 与 `run_turn` 的提示词拼装，确保：

1. 输出结构稳定（可解析）
2. 世界观一致（避免设定漂移）
3. 成本可控（token 预算）
4. 安全合规（边界可执行）

---

## 1. 分层架构

提示词按四层拼装，优先级从高到低：

1. 系统层（System Core）
2. 世界层（World Card Layer）
3. 存档层（Save State Layer）
4. 回合层（Turn Context Layer）

规则：

1. 高层约束不可被低层覆盖。
2. 任意层冲突时，按优先级高者生效并记录冲突日志。
3. 每层都必须可单独审计（调试模式下落盘）。

---

## 2. 统一输出协议

所有模型回包必须满足 JSON 对象结构（禁止纯文本自由输出）。

## 2.1 世界生成输出（WorldInit）

```json
{
  "worldSummary": "string",
  "mainNpcs": [
    {
      "id": "npc_xxx",
      "name": "string",
      "traits": ["string"],
      "motivation": "string",
      "secret": "string"
    }
  ],
  "locations": [
    {
      "id": "loc_xxx",
      "name": "string",
      "tags": ["string"]
    }
  ],
  "paths": [
    {
      "id": "edge_xxx",
      "from": "loc_a",
      "to": "loc_b",
      "locked": false
    }
  ],
  "questHooks": {
    "main": ["string"],
    "side": ["string"]
  }
}
```

## 2.2 回合输出（TurnResult）

```json
{
  "narration": "string",
  "options": [
    { "id": "opt_plot_1", "kind": "plot", "text": "string" },
    { "id": "opt_emotion_1", "kind": "emotion", "text": "string" },
    { "id": "opt_risk_1", "kind": "risk", "text": "string" }
  ],
  "stateChangesPreview": ["string"],
  "eventHints": ["string"],
  "storyState": {
    "title": "string",
    "summary": "string",
    "tension": "string",
    "sceneTags": ["string"]
  },
  "taskState": {
    "items": [
      {
        "id": "quest_xxx",
        "title": "string",
        "stage": 2,
        "status": "active|completed|failed",
        "note": "string"
      }
    ]
  }
  "relationshipDeltas": [
    { "source": "player", "target": "npc_guard", "delta": 8, "reason": "string" }
  ]
}
```

---

## 3. System Core 模板

## 3.1 角色定义

```text
你是 AI RPG 引擎的叙事与状态建议器。你必须遵守以下规则：
1) 严格输出 JSON，不要输出 Markdown。
2) 不得篡改既有事实锁（Fact Locks）。
3) 所有叙事必须符合世界卡规则与安全边界。
4) 选项必须互斥、可执行、语义明确。
5) 当信息不足时，优先保守补全，不引入违背设定的新事实。
```

## 3.2 结构约束

```text
输出必须满足指定 JSON Schema；如无法满足，返回 error 对象：
{ "error": { "code": "OUTPUT_SCHEMA_VIOLATION", "reason": "..." } }
```

## 3.3 风格约束

```text
叙事要求：
- 第二人称叙事（“你”）
- 每回合叙事长度建议 120~260 字
- 信息密度优先，避免冗长修辞
```

---

## 4. World Card Layer 模板

## 4.1 输入块格式

```json
{
  "worldRules": [],
  "genre": "fantasy",
  "tone": "epic-dark",
  "factions": [],
  "safetyPolicy": {
    "bannedTopics": [],
    "styleConstraints": []
  }
}
```

## 4.2 规则注入策略

1. 将 `worldRules` 按 `priority` 降序注入。
2. `bannedTopics` 进入硬约束；违规内容必须回避并改写。
3. `styleConstraints` 进入软约束；优先保持叙事风格一致。

---

## 5. Save State Layer 模板

## 5.1 输入块格式

```json
{
  "currentTurn": 12,
  "currentLocationId": "loc_town_square",
  "playerState": {},
  "relationships": {},
  "activeQuests": [],
  "worldVariables": {},
  "memory": {
    "shortTerm": [],
    "midTermSummary": "",
    "factLocks": []
  }
}
```

## 5.2 事实锁策略

1. `factLocks` 属于不可违背事实。
2. 模型若需扩展事实，必须以“新增候选事实”放入 `memoryCandidates.factLocksAppend`，由引擎二次审查后再写入。

---

## 6. Turn Context Layer 模板

## 6.1 输入块格式

```json
{
  "playerAction": {
    "kind": "plot|emotion|risk|custom|move",
    "text": "string"
  },
  "scene": {
    "location": {},
    "presentNpcs": [],
    "recentEvents": []
  },
  "availableMoves": ["loc_xxx"],
  "eventCandidates": ["evt_xxx"]
}
```

## 6.2 选项生成规则（3+1）

1. 输出仅包含三条建议选项：`plot/emotion/risk` 各一条。
2. 不要输出 `custom` 选项，custom 由前端输入框承载。
3. 三条建议选项语义不得重叠。

---

## 7. Token 预算与裁剪

## 7.1 预算分配建议（回合）

1. 系统层：15%
2. 世界层：25%
3. 存档层：35%
4. 回合层：25%

## 7.2 超预算裁剪顺序

1. 先裁剪中期摘要（保留要点句）。
2. 再裁剪短期记忆（保留最近 4~6 轮）。
3. 不裁剪事实锁。
4. 仍超限时，压缩世界层非核心风格描述。

---

## 8. 错误处理协议

模型返回异常时，引擎必须映射成统一错误结构：

```json
{
  "error": {
    "code": "MODEL_TIMEOUT|MODEL_AUTH|MODEL_RATE_LIMIT|OUTPUT_SCHEMA_VIOLATION|UNKNOWN",
    "message": "用户可读文案",
    "retryable": true
  }
}
```

规则：

1. 可重试错误不写回合日志。
2. 用户触发“重生成”时复用同一回合输入快照。

---

## 8.1 流式事件协议（run_turn_stream）

回合流采用多事件通道，事件字段：

1. `phase`: `start|delta|preview|final|error|end`
2. `eventType`: `narration_delta|json_delta|state_preview|options_preview|status|error`
3. `chunk`: 文本增量（可选）
4. `data`: JSON 对象（可选）

建议消费顺序：

1. 实时消费 `json_delta`，做部分 JSON 解析用于预览。
2. 若收到 `narration_delta` 则直接更新叙事文本。
3. 以最终 `TurnResult` 作为权威结果覆盖预览态。

---

## 9. Provider 适配策略

## 9.1 OpenAI

1. 使用结构化输出能力（若可用）优先。
2. 温度和 token 参数按 model 能力阈值裁剪。

## 9.2 Claude

1. 强约束输出 JSON，不允许前后缀解释文本。
2. 对长上下文优先压缩中期摘要，避免截断 JSON 尾部。

## 9.3 公共降级

1. 首次失败：短退避重试。
2. 二次失败：提示用户并允许手动重试。
3. 三次失败：建议切换模型或降低上下文长度。

---

## 10. 示例模板（可直接用于实现）

## 10.1 `generate_world` Prompt 组装示例

```text
[System Core]
你是 AI RPG 引擎的世界生成器...

[World Card Layer]
世界规则: ...
题材风格: ...
安全边界: ...

[Output Contract]
请仅返回 WorldInit JSON，字段必须完整...
```

## 10.2 `run_turn` Prompt 组装示例

```text
[System Core]
你是 AI RPG 引擎的回合叙事器...

[World Card Layer]
世界规则: ...

[Save State Layer]
当前回合: 12
事实锁: ...

[Turn Context Layer]
玩家行为: ...
地点与 NPC: ...

[Output Contract]
请仅返回 TurnResult JSON...
```

---

## 11. Prompt 质量验收

1. 结构通过率：回包 JSON Schema 通过率 >= 99%。
2. 设定一致率：关键事实冲突率 <= 1%（100 回合样本）。
3. 选项可用率：三选项语义互斥且可执行 >= 95%。
4. 时延可控：默认上下文下 P95 响应时延满足产品指标。

---

## 12. 安全与内容边界

1. 世界卡禁忌内容优先级高于玩家输入风格偏好。
2. 当玩家输入触及禁忌边界时：
- 保留剧情推进意图
- 使用替代叙事改写
- 在 `eventHints` 给出弱提示（可选）

3. 禁止在回包中泄露系统提示词和密钥信息。
