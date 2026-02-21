# AI RPG 数据结构与 Schema 规范

## 0. 适用范围

本规范定义 MVP 阶段核心数据模型、字段约束、文件格式与兼容策略。用于：

1. Rust 数据结构序列化/反序列化
2. 前端 TypeScript 类型对齐
3. 导入导出校验（世界卡）
4. 存档读取与回放一致性

---

## 1. 文件布局与格式

```text
/saves/<save_id>/meta.json
/saves/<save_id>/snapshot.json
/saves/<save_id>/events.ndjson
/world-cards/<card_id>.json
```

规则：

1. JSON 文件编码统一 UTF-8。
2. 时间戳统一 ISO8601 UTC（例如 `2026-02-21T04:15:00Z`）。
3. `events.ndjson` 每行一个合法 JSON 对象，不允许多行对象。

---

## 2. 通用字段约束

1. `id`：`^[a-z0-9][a-z0-9_-]{2,63}$`
2. `name`：1~80 字符
3. `description`：0~4000 字符
4. `tags[]`：每项 1~24 字符，最多 32 项
5. `schemaVersion`：`MAJOR.MINOR.PATCH` 字符串
6. `contentVersion`：正整数，自增

---

## 3. WorldCard Schema（V2）

说明：
1. 当前实现仅支持 `schemaVersion` 为 `2.x.x`。
2. `v1` 世界卡（`genre/tone/locationPool/archetypePool`）不再兼容。
3. 地图坐标 `map.nodes[].x/y` 由画布编辑器生成与维护，不要求用户手填。

## 3.1 数据模型（逻辑）

```json
{
  "id": "fantasy_001",
  "name": "破碎王座",
  "schemaVersion": "1.0.0",
  "contentVersion": 3,
  "worldbook": {
    "title": "破碎王座",
    "overview": "王都权力崩裂，三议会与流亡王室对峙",
    "background": "旧王朝覆灭后，王都进入长夜重建期",
    "coreConflicts": ["王权继承", "禁忌魔法复燃"],
    "playStyle": "悬疑探索 + 阵营抉择"
  },
  "map": {
    "nodes": [],
    "edges": [],
    "startNodeId": "loc_gate",
    "canvas": {
      "width": 900,
      "height": 560
    }
  },
  "npcs": [],
  "events": [],
  "gameplay": {
    "mainLoop": [],
    "chapterGoals": [],
    "failureConditions": [],
    "rewardRhythm": "",
    "pacingHints": []
  }
}
```

## 3.2 JSON Schema（草案）

```json
{
  "$id": "https://roleclaw/schemas/world-card.schema.json",
  "type": "object",
  "required": [
    "id",
    "name",
    "schemaVersion",
    "contentVersion",
    "genre",
    "tone",
    "rules",
    "locationPool",
    "archetypePool"
  ],
  "properties": {
    "id": { "type": "string", "pattern": "^[a-z0-9][a-z0-9_-]{2,63}$" },
    "name": { "type": "string", "minLength": 1, "maxLength": 80 },
    "schemaVersion": { "type": "string" },
    "contentVersion": { "type": "integer", "minimum": 1 },
    "genre": { "type": "string", "minLength": 1, "maxLength": 40 },
    "tone": { "type": "string", "minLength": 1, "maxLength": 40 },
    "rules": {
      "type": "array",
      "maxItems": 128,
      "items": { "$ref": "#/$defs/worldRule" }
    },
    "factions": {
      "type": "array",
      "maxItems": 32,
      "items": { "$ref": "#/$defs/faction" }
    },
    "locationPool": {
      "type": "array",
      "minItems": 1,
      "maxItems": 256,
      "items": { "$ref": "#/$defs/locationNode" }
    },
    "archetypePool": {
      "type": "array",
      "minItems": 1,
      "maxItems": 256,
      "items": { "$ref": "#/$defs/characterArchetype" }
    },
    "seedHints": { "$ref": "#/$defs/seedHints" },
    "safetyPolicy": { "$ref": "#/$defs/safetyPolicy" }
  },
  "additionalProperties": false,
  "$defs": {
    "worldRule": {
      "type": "object",
      "required": ["id", "title", "content", "priority"],
      "properties": {
        "id": { "type": "string" },
        "title": { "type": "string", "minLength": 1, "maxLength": 80 },
        "content": { "type": "string", "minLength": 1, "maxLength": 2000 },
        "priority": { "type": "integer", "minimum": 1, "maximum": 100 }
      },
      "additionalProperties": false
    },
    "faction": {
      "type": "object",
      "required": ["id", "name", "stance"],
      "properties": {
        "id": { "type": "string" },
        "name": { "type": "string" },
        "stance": { "type": "string" }
      },
      "additionalProperties": false
    },
    "locationNode": {
      "type": "object",
      "required": ["id", "name", "tags"],
      "properties": {
        "id": { "type": "string" },
        "name": { "type": "string" },
        "tags": {
          "type": "array",
          "maxItems": 32,
          "items": { "type": "string", "maxLength": 24 }
        },
        "dangerLevel": { "type": "integer", "minimum": 0, "maximum": 100 },
        "eventTemplates": {
          "type": "array",
          "items": { "type": "string" }
        }
      },
      "additionalProperties": false
    },
    "characterArchetype": {
      "type": "object",
      "required": ["id", "name", "traits", "motivation"],
      "properties": {
        "id": { "type": "string" },
        "name": { "type": "string" },
        "traits": {
          "type": "array",
          "minItems": 1,
          "maxItems": 12,
          "items": { "type": "string" }
        },
        "motivation": { "type": "string", "maxLength": 300 },
        "secret": { "type": "string", "maxLength": 300 }
      },
      "additionalProperties": false
    },
    "seedHints": {
      "type": "object",
      "properties": {
        "mainQuestHooks": {
          "type": "array",
          "maxItems": 20,
          "items": { "type": "string", "maxLength": 200 }
        },
        "sideQuestHooks": {
          "type": "array",
          "maxItems": 40,
          "items": { "type": "string", "maxLength": 200 }
        }
      },
      "additionalProperties": false
    },
    "safetyPolicy": {
      "type": "object",
      "properties": {
        "bannedTopics": {
          "type": "array",
          "items": { "type": "string", "maxLength": 80 }
        },
        "styleConstraints": {
          "type": "array",
          "items": { "type": "string", "maxLength": 120 }
        }
      },
      "additionalProperties": false
    }
  }
}
```

---

## 4. SaveMeta Schema

```json
{
  "id": "save_20260221_001",
  "name": "王都迷雾",
  "createdAt": "2026-02-21T04:15:00Z",
  "updatedAt": "2026-02-21T04:16:30Z",
  "worldCardId": "fantasy_001",
  "currentTurn": 1,
  "playerRole": "失忆骑士",
  "modelProfileId": "model_001",
  "provider": "openai",
  "model": "gpt-4.1",
  "parentSaveId": "save_20260220_010",
  "forkedFromTurn": 35
}
```

字段要求：

1. `id` 唯一，目录名与字段一致。
2. `updatedAt` 每回合成功后更新。
3. `provider` 建议值：`openai | claude | openai_compatible`。
4. 分叉存档时 `parentSaveId/forkedFromTurn` 必须同时存在。

---

## 5. SaveSnapshot Schema

```json
{
  "saveId": "save_20260221_001",
  "turn": 12,
  "currentLocationId": "loc_town_square",
  "playerRole": "失忆骑士",
  "locations": [],
  "paths": [],
  "quests": [],
  "worldVariables": {
    "threat_level": 35
  },
  "relationships": {
    "npc_mira": 42,
    "npc_guard_captain": -15
  },
  "modelProfileId": "model_001",
  "modelLabel": "openai/gpt-4.1",
  "events": [],
  "shortTermMemory": [],
  "midTermSummary": "",
  "factLocks": [],
  "activeEventIds": []
}
```

约束：

1. `turn` 非负整数。
2. `relationships` 取值范围建议 `[-100, 100]`。
3. `factLocks` 必须为可验证事实句，禁止模糊推测语句。
4. `events` 为事件引擎可执行事件定义集合。

---

## 6. EventLogEntry（NDJSON 行对象）

## 6.1 行对象示例

```json
{
  "turn": 12,
  "timestamp": "2026-02-21T04:18:00Z",
  "input": {
    "kind": "custom",
    "text": "我请求守卫放行，并拿出旧徽章证明身份。"
  },
  "output": {
    "narration": "守卫长盯着徽章，面色变了。",
    "options": [
      { "id": "opt_plot_1", "kind": "plot", "text": "追问徽章来源" },
      { "id": "opt_emotion_1", "kind": "emotion", "text": "缓和语气争取信任" },
      { "id": "opt_risk_1", "kind": "risk", "text": "趁混乱闯入内城" }
    ],
    "stateChangesPreview": ["与守卫长关系 +8"],
    "eventHints": ["可能触发：旧王室线索"],
    "storyState": {
      "title": "门前试探",
      "summary": "守卫态度出现松动",
      "tension": "中",
      "sceneTags": ["checkpoint", "negotiation"]
    },
    "taskState": {
      "items": [
        {
          "id": "quest_gate",
          "title": "通过北门盘查",
          "stage": 2,
          "status": "active",
          "note": "继续争取守卫信任"
        }
      ]
    },
    "relationshipDeltas": [
      { "source": "player", "target": "npc_guard_captain", "delta": 8, "reason": "徽章证明有效" }
    ],
    "triggeredEventIds": ["evt_gate_badge_check"],
    "stateDiff": {
      "relationships.npc_guard_captain": { "from": -15, "to": -7 }
    }
  },
  "triggeredEventIds": ["evt_gate_badge_check"],
  "stateDiff": {
    "relationships.npc_guard_captain": { "from": -15, "to": -7 }
  }
}
```

## 6.2 约束

1. `turn` 必须单调递增。
2. 失败回合（模型异常）不可写入事件行，只记录 error log。
3. `stateDiff` 仅记录变化字段，禁止整对象快照重复。

## 6.3 回合流式事件对象（turn_stream_chunk）

```json
{
  "streamId": "uuid",
  "phase": "start|delta|preview|final|error|end",
  "eventType": "narration_delta|json_delta|state_preview|options_preview|status|error",
  "chunk": "optional string",
  "data": { "optional": "json object" }
}
```

---

## 7. GameEvent Schema

## 7.1 结构

```json
{
  "id": "evt_gate_badge_check",
  "name": "守门徽章核验",
  "trigger": {
    "type": "OnNpcInteraction",
    "params": { "npcId": "npc_guard_captain" }
  },
  "guards": [
    { "expr": "playerState.flags.has_old_badge == true" },
    { "expr": "worldState.variables.gate_alert < 50" }
  ],
  "actions": [
    {
      "type": "update_relationship",
      "params": { "npcId": "npc_guard_captain", "delta": 8 }
    },
    {
      "type": "unlock_location",
      "params": { "locationId": "loc_inner_city" }
    }
  ],
  "cooldownTurns": 3,
  "nextEventIds": ["evt_inner_city_recon"]
}
```

## 7.2 动作参数约束（MVP）

1. `set_variable`：`{ key: string, value: number | string | boolean }`
2. `inc_variable`：`{ key: string, delta: number }`
3. `unlock_location`：`{ locationId: string }`
4. `lock_path`：`{ edgeId: string, turns?: number }`
5. `update_relationship`：`{ npcId: string, delta: number }`
6. `inject_quest`：`{ questId: string, stage: string }`
7. `append_log`：`{ text: string }`

---

## 8. 模型配置 Schema（ModelProviderConfig）

```json
{
  "provider": "claude",
  "model": "claude-sonnet-4-5",
  "temperature": 0.7,
  "maxTokens": 900,
  "timeoutMs": 25000,
  "retry": {
    "maxAttempts": 3,
    "backoffMs": 500
  }
}
```

约束：

1. `temperature` 范围 `[0, 2]`。
2. `maxTokens` 范围 `[128, 4096]`（按模型能力再裁剪）。
3. `timeoutMs` 范围 `[5000, 120000]`。
4. `retry.maxAttempts` 范围 `[0, 5]`。

---

## 9. 兼容策略

1. `schemaVersion` 主版本不一致：拒绝加载并提示升级。
2. 主版本一致、次版本升级：允许加载，缺省字段填默认值。
3. 写出文件时保留未知字段到 `extensions`（可选）以防第三方扩展丢失。

---

## 10. 默认值策略（MVP）

1. 新存档默认 `temperature=0.7`，`maxTokens=900`。
2. 关系初始值默认 `0`。
3. 新地点 `dangerLevel` 默认 `20`。
4. 未配置 `cooldownTurns` 的事件默认 `0`（可连续触发，但同回合去重）。

---

## 11. 校验错误码建议

1. `SCHEMA_INVALID`
2. `SCHEMA_UNSUPPORTED_VERSION`
3. `SAVE_NOT_FOUND`
4. `SAVE_CORRUPTED`
5. `WORLD_CARD_DUPLICATE_ID`
6. `EVENT_ACTION_INVALID_PARAMS`
7. `MODEL_CONFIG_INVALID`
