use crate::domain::{
    AiMeta, CardPromptEvent, CharacterArchetype, DialogueOption, EventAction, EventLogEntry,
    EventResult, GameEvent, LocationNode, MapCanvas, MapEdge, MapNode, ModelProviderConfig,
    NpcProfile, PathEdge, QuestState, RelationshipDelta, SaveSnapshot, StoryState, TaskState,
    TaskStateItem, TriggerCondition, TurnInput, TurnResult, TurnStateProposal, WorldBook, WorldCard,
    WorldInit, WorldMap,
};
use crate::llm::{generate_turn_json, stream_turn_json, TurnJsonStreamPiece};
use crate::storage::{
    append_ndjson, collect_recent_logs, load_global_data, load_meta, load_snapshot, now_iso,
    read_json, write_meta, write_snapshot, AppPaths,
};
use crate::validate::{validate_event_log_entry, validate_game_event, validate_save_snapshot};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

const EVENT_CHAIN_MAX_DEPTH: usize = 6;
const RELATIONSHIP_DELTA_LIMIT_PER_TURN: f64 = 20.0;

#[derive(Debug, Clone)]
pub struct TurnStreamEvent {
    pub phase: String,
    pub event_type: Option<String>,
    pub chunk: Option<String>,
    pub data: Option<Value>,
}

pub fn default_world_cards() -> Vec<WorldCard> {
    vec![
        WorldCard {
            id: "fantasy_realm".to_string(),
            name: "破碎王座".to_string(),
            schema_version: "2.0.0".to_string(),
            content_version: 1,
            worldbook: WorldBook {
                title: "破碎王座".to_string(),
                overview: "王都权力崩裂，三议会与流亡王室对峙".to_string(),
                background: "旧王朝覆灭后，王都进入长夜重建期".to_string(),
                core_conflicts: vec!["王权继承".to_string(), "禁忌魔法复燃".to_string()],
                play_style: "悬疑探索 + 阵营抉择".to_string(),
            },
            map: WorldMap {
                nodes: vec![
                    MapNode {
                    id: "loc_gate".to_string(),
                    name: "北门".to_string(),
                    description: "驻军把守的入城关口".to_string(),
                    x: 120.0,
                    y: 140.0,
                    tags: vec!["city".to_string(), "checkpoint".to_string()],
                },
                    MapNode {
                    id: "loc_square".to_string(),
                    name: "钟楼广场".to_string(),
                    description: "商贩与流言聚集之地".to_string(),
                    x: 320.0,
                    y: 220.0,
                    tags: vec!["city".to_string(), "crowded".to_string()],
                },
                    MapNode {
                    id: "loc_tower".to_string(),
                    name: "旧法师塔".to_string(),
                    description: "封锁中的古代法术研究设施".to_string(),
                    x: 520.0,
                    y: 120.0,
                    tags: vec!["mystic".to_string(), "danger".to_string()],
                },
                ],
                edges: vec![
                    MapEdge {
                        id: "edge_gate_square".to_string(),
                        a: "loc_gate".to_string(),
                        b: "loc_square".to_string(),
                        locked: false,
                        unlock_conditions: vec![],
                    },
                    MapEdge {
                        id: "edge_square_tower".to_string(),
                        a: "loc_square".to_string(),
                        b: "loc_tower".to_string(),
                        locked: false,
                        unlock_conditions: vec![],
                    },
                ],
                start_node_id: "loc_gate".to_string(),
                canvas: MapCanvas {
                    width: 900,
                    height: 560,
                },
            },
            npcs: vec![
                NpcProfile {
                    id: "npc_guard".to_string(),
                    name: "守卫长卡恩".to_string(),
                    personality: vec!["警惕".to_string(), "讲规则".to_string()],
                    identity: "王都北门守卫长".to_string(),
                },
                NpcProfile {
                    id: "npc_bard".to_string(),
                    name: "吟游诗人米拉".to_string(),
                    personality: vec!["健谈".to_string(), "圆滑".to_string()],
                    identity: "流动情报贩子".to_string(),
                },
            ],
            events: vec![CardPromptEvent {
                id: "evt_enter_gate".to_string(),
                name: "初到北门".to_string(),
                prompt: "你来到北门，守卫正在盘查来往人群，这里可能有王室线索。".to_string(),
            }],
            chapter_goals: vec![crate::domain::ChapterGoal {
                    id: "chapter_1".to_string(),
                    title: "迷雾开端".to_string(),
                    prompt: "确认王都局势与关键阵营，拿到第一份王室线索。".to_string(),
                }],
        },
        WorldCard {
            id: "cyber_city".to_string(),
            name: "霓虹深井".to_string(),
            schema_version: "2.0.0".to_string(),
            content_version: 1,
            worldbook: WorldBook {
                title: "霓虹深井".to_string(),
                overview: "算力财团垄断城市，地下网络暗流涌动".to_string(),
                background: "主核事故后，城市被四大企业分区统治".to_string(),
                core_conflicts: vec!["数据主权".to_string(), "企业战争".to_string()],
                play_style: "赛博调查 + 潜入抉择".to_string(),
            },
            map: WorldMap {
                nodes: vec![
                    MapNode {
                    id: "loc_dock".to_string(),
                    name: "灰港接入站".to_string(),
                    description: "黑市硬件与匿名委托中转点".to_string(),
                    x: 130.0,
                    y: 260.0,
                    tags: vec!["port".to_string(), "black-market".to_string()],
                },
                    MapNode {
                    id: "loc_tower".to_string(),
                    name: "主核塔".to_string(),
                    description: "企业控制的核心算力枢纽".to_string(),
                    x: 430.0,
                    y: 120.0,
                    tags: vec!["corp".to_string(), "restricted".to_string()],
                },
                ],
                edges: vec![MapEdge {
                    id: "edge_dock_tower".to_string(),
                    a: "loc_dock".to_string(),
                    b: "loc_tower".to_string(),
                    locked: false,
                    unlock_conditions: vec![],
                }],
                start_node_id: "loc_dock".to_string(),
                canvas: MapCanvas {
                    width: 900,
                    height: 560,
                },
            },
            npcs: vec![NpcProfile {
                id: "npc_broker".to_string(),
                name: "中间人 R-9".to_string(),
                personality: vec!["理性".to_string(), "逐利".to_string()],
                identity: "黑市交易撮合人".to_string(),
            }],
            events: vec![],
            chapter_goals: vec![],
        },
    ]
}

pub fn generate_world_from_card(card: &WorldCard, player_role: &str) -> WorldInit {
    let locations = if card.map.nodes.is_empty() {
        vec![crate::domain::LocationNode {
            id: "loc_origin".to_string(),
            name: "起始点".to_string(),
            x: 180.0,
            y: 180.0,
            tags: vec!["origin".to_string()],
            npc_ids: Vec::new(),
            event_ids: Vec::new(),
        }]
    } else {
        card.map
            .nodes
            .iter()
            .map(|node| LocationNode {
                id: node.id.clone(),
                name: node.name.clone(),
                x: node.x,
                y: node.y,
                tags: node.tags.clone(),
                npc_ids: vec![],
                event_ids: card
                    .events
                    .iter()
                    .map(|evt| evt.id.clone())
                    .collect(),
            })
            .collect()
    };

    let mut paths: Vec<PathEdge> = card
        .map
        .edges
        .iter()
        .map(|edge| PathEdge {
            id: edge.id.clone(),
            from: edge.a.clone(),
            to: edge.b.clone(),
            locked: edge.locked,
            conditions: edge.unlock_conditions.clone(),
        })
        .collect();
    if paths.is_empty() && locations.len() > 1 {
        for i in 0..(locations.len() - 1) {
            paths.push(PathEdge {
                id: format!("edge_{}_{}", locations[i].id, locations[i + 1].id),
                from: locations[i].id.clone(),
                to: locations[i + 1].id.clone(),
                locked: false,
                conditions: Vec::new(),
            });
        }
    }

    WorldInit {
        world_summary: format!(
            "你进入了《{}》。{}\n背景：{}\n你的身份是“{}”。",
            card.worldbook.title, card.worldbook.overview, card.worldbook.background, player_role
        ),
        main_npcs: card
            .npcs
            .iter()
            .map(|npc| CharacterArchetype {
                id: npc.id.clone(),
                name: npc.name.clone(),
                traits: npc.personality.clone(),
                motivation: npc.identity.clone(),
                secret: None,
            })
            .collect(),
        locations,
        paths,
        quest_hooks: if card.chapter_goals.is_empty() {
            vec![
                "调查第一条异动线索".to_string(),
                "建立与关键 NPC 的初始信任".to_string(),
                "找出影响当前区域的核心冲突".to_string(),
            ]
        } else {
            card
                .chapter_goals
                .iter()
                .map(|goal| goal.prompt.clone())
                .collect()
        },
    }
}

pub fn seed_events_for_world(locations: &[LocationNode]) -> Vec<GameEvent> {
    let mut out = Vec::new();
    for loc in locations {
        let id = format!("evt_on_enter_{}", loc.id);
        let event = GameEvent {
            id,
            name: format!("进入{}", loc.name),
            trigger: crate::domain::TriggerCondition {
                r#type: "on_enter_location".to_string(),
                params: BTreeMap::from([("locationId".to_string(), Value::String(loc.id.clone()))]),
            },
            guards: Vec::new(),
            actions: vec![crate::domain::EventAction {
                r#type: "append_log".to_string(),
                params: BTreeMap::from([(
                    "message".to_string(),
                    Value::String(format!("你抵达了 {}", loc.name)),
                )]),
            }],
            cooldown_turns: None,
            next_event_ids: Vec::new(),
        };
        out.push(event);
    }
    out
}

pub fn project_card_events(card: &WorldCard) -> Vec<GameEvent> {
    card.events
        .iter()
        .map(|evt| {
            let mut params = BTreeMap::new();
            params.insert("message".to_string(), Value::String(evt.prompt.clone()));
            GameEvent {
            id: evt.id.clone(),
            name: evt.name.clone(),
            trigger: TriggerCondition {
                r#type: "on_turn_elapsed".to_string(),
                params: BTreeMap::new(),
            },
            guards: vec![],
            actions: vec![EventAction {
                r#type: "append_log".to_string(),
                params,
            }],
            cooldown_turns: None,
            next_event_ids: vec![],
        }})
        .collect()
}

fn load_prompt_context(
    paths: &AppPaths,
    meta: &crate::domain::SaveMeta,
    snapshot: &SaveSnapshot,
) -> (Vec<String>, String) {
    let path = paths
        .world_cards_dir
        .join(format!("{}.json", meta.world_card_id));
    let Ok(card) = read_json::<WorldCard>(&path) else {
        return (Vec::new(), String::new());
    };
    let event_prompts = card.events.iter().map(|evt| evt.prompt.clone()).collect();
    if card.chapter_goals.is_empty() {
        return (event_prompts, String::new());
    }
    let idx = ((snapshot.turn as usize) / 3).min(card.chapter_goals.len().saturating_sub(1));
    let chapter_prompt = card.chapter_goals[idx].prompt.clone();
    (event_prompts, chapter_prompt)
}

fn push_short_memory(snapshot: &mut SaveSnapshot, line: String) {
    snapshot.short_term_memory.push(line);
    if snapshot.short_term_memory.len() > 8 {
        let keep_from = snapshot.short_term_memory.len() - 8;
        snapshot.short_term_memory = snapshot.short_term_memory.split_off(keep_from);
    }
}

fn trim_by_chars(input: &str, max_chars: usize) -> String {
    input.chars().take(max_chars).collect()
}

fn summarize_relationships(snapshot: &SaveSnapshot) -> String {
    let mut pairs = Vec::new();
    for (id, value) in &snapshot.relationships {
        let text = value
            .as_f64()
            .map(|v| format!("{v:.1}"))
            .or_else(|| value.as_str().map(|v| v.to_string()))
            .unwrap_or_else(|| value.to_string());
        pairs.push(format!("{id}:{text}"));
    }
    trim_by_chars(&pairs.join(" | "), 360)
}

fn summarize_quests(snapshot: &SaveSnapshot) -> String {
    let lines = snapshot
        .quests
        .iter()
        .map(|quest| {
            format!(
                "{}({})-stage:{}-{}",
                quest.id,
                quest.title,
                quest.stage,
                if quest.completed { "completed" } else { "active" }
            )
        })
        .collect::<Vec<_>>();
    trim_by_chars(&lines.join(" | "), 400)
}

fn summarize_recent_logs(logs: &[EventLogEntry]) -> String {
    let lines = logs
        .iter()
        .map(|row| {
            format!(
                "T{}:{}|{}",
                row.turn,
                trim_by_chars(&row.input.option_id.clone().unwrap_or_default(), 30),
                trim_by_chars(&row.output.state_changes_preview.join("/"), 60)
            )
        })
        .collect::<Vec<_>>();
    trim_by_chars(&lines.join(" || "), 500)
}

fn load_world_card(paths: &AppPaths, meta: &crate::domain::SaveMeta) -> Option<WorldCard> {
    let path = paths
        .world_cards_dir
        .join(format!("{}.json", meta.world_card_id));
    read_json::<WorldCard>(&path).ok()
}

fn build_turn_generation_context(
    snapshot: &SaveSnapshot,
    selected: &str,
    recent_logs: &[EventLogEntry],
    world_card: Option<&WorldCard>,
    chapter_prompt: &str,
    event_prompts: &[String],
) -> String {
    let current_loc = snapshot
        .locations
        .iter()
        .find(|loc| loc.id == snapshot.current_location_id)
        .map(|loc| format!("{}({})", loc.name, loc.id))
        .unwrap_or_else(|| snapshot.current_location_id.clone());
    let world_layer = if let Some(card) = world_card {
        format!(
            "[World Card Layer]\ntitle={}\noverview={}\nbackground={}\ncoreConflicts={}\nplayStyle={}\nchapterGoal={}\neventPrompts={}",
            card.worldbook.title,
            trim_by_chars(&card.worldbook.overview, 220),
            trim_by_chars(&card.worldbook.background, 220),
            trim_by_chars(&card.worldbook.core_conflicts.join(" | "), 220),
            trim_by_chars(&card.worldbook.play_style, 120),
            trim_by_chars(chapter_prompt, 180),
            trim_by_chars(&event_prompts.join(" || "), 260)
        )
    } else {
        format!(
            "[World Card Layer]\nchapterGoal={}\neventPrompts={}",
            trim_by_chars(chapter_prompt, 180),
            trim_by_chars(&event_prompts.join(" || "), 260)
        )
    };
    let system_layer = "[System Core]\n你是 AI RPG 回合叙事与状态建议器。必须只输出合法 JSON 对象。\n约束：\n1) 不得违背事实锁。\n2) narration 使用第二人称中文，120~260字。\n3) options 必须为3条互斥可执行选项。\n4) 第四选项由玩家自由输入，你不能输出 custom。\n5) 若信息不足，保守推进并保持世界一致性。";
    let world_layer = format!(
        "{}",
        world_layer
    );
    let save_layer = format!(
        "[Save State Layer]\nturn={}\nlocation={}\nworldSummary={}\nrelationships={}\nquests={}\nworldVariables={}\nshortTermMemory={}\nmidTermSummary={}\nfactLocks={}\nrecentTurns={}",
        snapshot.turn,
        current_loc,
        trim_by_chars(&snapshot.world_summary, 450),
        summarize_relationships(snapshot),
        summarize_quests(snapshot),
        trim_by_chars(&snapshot.world_variables.len().to_string(), 80),
        trim_by_chars(&snapshot.short_term_memory.join(" || "), 320),
        trim_by_chars(&snapshot.mid_term_summary, 260),
        trim_by_chars(&snapshot.fact_locks.join(" | "), 320),
        summarize_recent_logs(recent_logs)
    );
    let reachable_locations = snapshot
        .paths
        .iter()
        .filter(|edge| !edge.locked)
        .filter_map(|edge| {
            if edge.from == snapshot.current_location_id {
                Some(edge.to.clone())
            } else if edge.to == snapshot.current_location_id {
                Some(edge.from.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    let turn_layer = format!(
        "[Turn Context Layer]\nplayerRole={}\nplayerAction={}\nreachableLocations={}\ncurrentTurnGoal={}",
        snapshot.player_role,
        selected,
        reachable_locations,
        trim_by_chars(chapter_prompt, 180)
    );
    let output_contract = "[Output Contract]\n输出 JSON 且仅包含以下字段：\n{\n  \"narration\": string,\n  \"options\": [{\"kind\": string, \"text\": string}],\n  \"stateChangesPreview\": string[],\n  \"eventHints\": string[],\n  \"storyState\": {\"title\": string, \"summary\": string, \"tension\": string, \"sceneTags\": string[]},\n  \"taskState\": {\"items\": [{\"id\": string, \"title\": string, \"stage\": number, \"status\": \"active|completed|failed\", \"note\": string}]},\n  \"relationshipDeltas\": [{\"source\": string, \"target\": string, \"delta\": number, \"reason\": string}]\n}\n注意：options 必须恰好3条，且不要输出 id。";
    trim_by_chars(
        &format!(
            "{}\n{}\n{}\n{}\n{}",
            system_layer, world_layer, save_layer, turn_layer, output_contract
        ),
        7000,
    )
}

fn extract_json_object(raw: &str) -> Option<&str> {
    let trimmed = raw.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }
    if let Some(start_idx) = trimmed.find('{') {
        let mut depth = 0i32;
        let mut end_idx = None;
        for (idx, ch) in trimmed.char_indices().skip(start_idx) {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    end_idx = Some(idx);
                    break;
                }
            }
        }
        if let Some(end) = end_idx {
            return Some(&trimmed[start_idx..=end]);
        }
    }
    None
}

fn parse_relationship_deltas(value: &Value) -> Vec<RelationshipDelta> {
    value
        .get("relationshipDeltas")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let obj = item.as_object()?;
                    Some(RelationshipDelta {
                        source: obj
                            .get("source")
                            .and_then(Value::as_str)
                            .unwrap_or("player")
                            .to_string(),
                        target: obj
                            .get("target")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        delta: obj.get("delta").and_then(Value::as_f64).unwrap_or(0.0),
                        reason: obj
                            .get("reason")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_task_state(value: &Value) -> Option<TaskState> {
    let items = value
        .get("taskState")
        .and_then(Value::as_object)
        .and_then(|obj| obj.get("items"))
        .and_then(Value::as_array)?
        .iter()
        .filter_map(|item| {
            let obj = item.as_object()?;
            let id = obj.get("id").and_then(Value::as_str)?.trim().to_string();
            if id.is_empty() {
                return None;
            }
            Some(TaskStateItem {
                id,
                title: obj
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
                stage: obj.get("stage").and_then(Value::as_u64).unwrap_or(1) as u32,
                status: obj
                    .get("status")
                    .and_then(Value::as_str)
                    .unwrap_or("active")
                    .to_string(),
                note: obj
                    .get("note")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    Some(TaskState { items })
}

fn parse_story_state(value: &Value) -> Option<StoryState> {
    let story = value.get("storyState")?.as_object()?;
    Some(StoryState {
        title: story
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        summary: story
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        tension: story
            .get("tension")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        scene_tags: story
            .get("sceneTags")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    })
}

fn parse_turn_proposal(raw: &str) -> Result<TurnStateProposal, String> {
    let json_slice =
        extract_json_object(raw).ok_or_else(|| "模型未返回合法 JSON 对象".to_string())?;
    let value: Value = serde_json::from_str(json_slice)
        .map_err(|err| format!("模型 JSON 解析失败: {err}"))?;
    let narration = value
        .get("narration")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if narration.is_empty() {
        return Err("回合 narration 为空".to_string());
    }

    let mut options = value
        .get("options")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .take(3)
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    let text = obj.get("text").and_then(Value::as_str)?.trim().to_string();
                    if text.is_empty() {
                        return None;
                    }
                    Some(DialogueOption {
                        id: format!("opt_{}", idx + 1),
                        kind: obj
                            .get("kind")
                            .and_then(Value::as_str)
                            .filter(|v| !v.trim().is_empty())
                            .unwrap_or("approach")
                            .to_string(),
                        text,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    while options.len() < 3 {
        let idx = options.len() + 1;
        options.push(DialogueOption {
            id: format!("opt_{idx}"),
            kind: "approach".to_string(),
            text: format!("继续推进当前线索（方案{idx}）"),
        });
    }

    let state_changes_preview = value
        .get("stateChangesPreview")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let event_hints = value
        .get("eventHints")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(TurnStateProposal {
        narration,
        options,
        state_changes_preview,
        event_hints,
        story_state: parse_story_state(&value),
        task_state: parse_task_state(&value),
        relationship_deltas: parse_relationship_deltas(&value),
    })
}

fn apply_ai_proposal_with_guardrails(
    snapshot: &mut SaveSnapshot,
    proposal: &TurnStateProposal,
) -> (Vec<String>, Vec<RelationshipDelta>) {
    let mut changes = Vec::new();
    let mut accepted_rel = Vec::new();

    for delta in &proposal.relationship_deltas {
        if delta.target.trim().is_empty() {
            continue;
        }
        let allowed_delta = delta
            .delta
            .clamp(-RELATIONSHIP_DELTA_LIMIT_PER_TURN, RELATIONSHIP_DELTA_LIMIT_PER_TURN);
        let prev = snapshot
            .relationships
            .get(&delta.target)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let next = (prev + allowed_delta).clamp(-100.0, 100.0);
        snapshot
            .relationships
            .insert(delta.target.clone(), Value::from(next));
        accepted_rel.push(RelationshipDelta {
            source: delta.source.clone(),
            target: delta.target.clone(),
            delta: next - prev,
            reason: delta.reason.clone(),
        });
        changes.push(format!("关系 {} -> {:.1}", delta.target, next));
    }

    if let Some(task_state) = proposal.task_state.as_ref() {
        for item in &task_state.items {
            if item.id.trim().is_empty() {
                continue;
            }
            let stage = item.stage.clamp(1, 99);
            if let Some(existing) = snapshot.quests.iter_mut().find(|quest| quest.id == item.id) {
                existing.stage = stage;
                existing.completed = item.status == "completed";
                changes.push(format!(
                    "任务 {} -> stage {} ({})",
                    existing.id, existing.stage, item.status
                ));
                continue;
            }
            if item.status == "active" || item.status == "completed" {
                snapshot.quests.push(QuestState {
                    id: item.id.clone(),
                    title: if item.title.trim().is_empty() {
                        item.id.clone()
                    } else {
                        item.title.clone()
                    },
                    stage,
                    completed: item.status == "completed",
                });
                changes.push(format!("新增任务 {}", item.id));
            }
        }
    }

    (changes, accepted_rel)
}

fn delta_from_seen_chars(full: &str, seen_chars: &mut usize) -> Option<String> {
    let total_chars = full.chars().count();
    if total_chars <= *seen_chars {
        return None;
    }
    let delta = full.chars().skip(*seen_chars).collect::<String>();
    *seen_chars = total_chars;
    if delta.is_empty() {
        None
    } else {
        Some(delta)
    }
}

fn resolve_runtime_model_config(
    paths: &AppPaths,
    snapshot: &crate::domain::SaveSnapshot,
) -> Result<ModelProviderConfig, String> {
    if snapshot.model_profile_id.trim().is_empty() {
        return Err("当前存档未绑定模型，请到 AI 设置页修复后重新创建或迁移存档".to_string());
    }
    let global_data = load_global_data(paths)?;
    let profile = global_data
        .ai_settings
        .models
        .iter()
        .find(|item| item.id == snapshot.model_profile_id)
        .ok_or_else(|| "当前存档绑定模型不存在，请到 AI 设置页修复".to_string())?;
    let api_key_missing = profile
        .api_key
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    if api_key_missing {
        return Err("当前存档绑定模型未配置 API Key，请到 AI 设置页修复".to_string());
    }

    Ok(ModelProviderConfig {
        provider_type: profile.provider_type.clone(),
        provider: profile.provider.clone(),
        base_url: profile.base_url.clone(),
        model: profile.model.clone(),
        api_key: profile.api_key.clone(),
        temperature: profile.temperature,
        max_tokens: Some(profile.max_tokens),
        timeout_ms: {
            let mut timeout_ms = profile.timeout_ms.max(30_000);
            let model_lower = profile.model.to_lowercase();
            let provider_lower = profile.provider.to_lowercase();
            if model_lower.contains("reasoner")
                || model_lower.contains("r1")
                || provider_lower.contains("deepseek")
            {
                timeout_ms = timeout_ms.max(120_000);
            }
            timeout_ms
        },
    })
}

fn evaluate_guard(snapshot: &SaveSnapshot, expr: &str) -> bool {
    // Minimal guard parser for expr like: relationship.npc_guard >= 30
    let normalized = expr.trim();
    if normalized.is_empty() {
        return true;
    }
    if let Some((left, right)) = normalized.split_once(">=") {
        let left = left.trim();
        let threshold = right.trim().parse::<f64>().unwrap_or(f64::MAX);
        if let Some(key) = left.strip_prefix("relationship.") {
            let value = snapshot
                .relationships
                .get(key)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            return value >= threshold;
        }
    }
    true
}

fn apply_event_action(
    snapshot: &mut SaveSnapshot,
    action: &crate::domain::EventAction,
    changes: &mut Vec<String>,
) {
    match action.r#type.as_str() {
        "set_variable" => {
            if let (Some(key), Some(value)) = (
                action.params.get("key").and_then(|v| v.as_str()),
                action.params.get("value").cloned(),
            ) {
                snapshot
                    .world_variables
                    .insert(key.to_string(), value.clone());
                changes.push(format!("变量 {} 已更新", key));
            }
        }
        "inc_variable" => {
            if let (Some(key), Some(inc)) = (
                action.params.get("key").and_then(|v| v.as_str()),
                action.params.get("delta").and_then(|v| v.as_f64()),
            ) {
                let next = snapshot
                    .world_variables
                    .get(key)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
                    + inc;
                snapshot
                    .world_variables
                    .insert(key.to_string(), Value::from(next));
                changes.push(format!("变量 {} 变化为 {:.1}", key, next));
            }
        }
        "update_relationship" => {
            if let (Some(npc_id), Some(delta)) = (
                action.params.get("npcId").and_then(|v| v.as_str()),
                action.params.get("delta").and_then(|v| v.as_f64()),
            ) {
                let old = snapshot
                    .relationships
                    .get(npc_id)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let next = old + delta;
                snapshot
                    .relationships
                    .insert(npc_id.to_string(), Value::from(next));
                changes.push(format!("关系 {} -> {:.1}", npc_id, next));
            }
        }
        "unlock_location" => {
            if let Some(location_id) = action.params.get("locationId").and_then(|v| v.as_str()) {
                for edge in &mut snapshot.paths {
                    if edge.from == location_id || edge.to == location_id {
                        edge.locked = false;
                    }
                }
                changes.push(format!("地点 {} 相关路径已解锁", location_id));
            }
        }
        "lock_path" => {
            if let Some(path_id) = action.params.get("pathId").and_then(|v| v.as_str()) {
                for edge in &mut snapshot.paths {
                    if edge.id == path_id {
                        edge.locked = true;
                    }
                }
                changes.push(format!("路径 {} 已锁定", path_id));
            }
        }
        "inject_quest" => {
            if let (Some(id), Some(title)) = (
                action.params.get("id").and_then(|v| v.as_str()),
                action.params.get("title").and_then(|v| v.as_str()),
            ) {
                if !snapshot.quests.iter().any(|quest| quest.id == id) {
                    snapshot.quests.push(QuestState {
                        id: id.to_string(),
                        title: title.to_string(),
                        stage: 1,
                        completed: false,
                    });
                    changes.push(format!("新任务：{}", title));
                }
            }
        }
        "advance_quest_stage" => {
            if let Some(id) = action.params.get("id").and_then(|v| v.as_str()) {
                if let Some(quest) = snapshot.quests.iter_mut().find(|quest| quest.id == id) {
                    quest.stage += 1;
                    changes.push(format!("任务 {} 进入阶段 {}", id, quest.stage));
                }
            }
        }
        "append_log" => {
            if let Some(message) = action.params.get("message").and_then(|v| v.as_str()) {
                changes.push(message.to_string());
            }
        }
        _ => {}
    }
}

fn event_match(event: &GameEvent, trigger_type: &str, context: &BTreeMap<String, Value>) -> bool {
    if event.trigger.r#type != trigger_type {
        return false;
    }
    for (key, value) in &event.trigger.params {
        if context.get(key) != Some(value) {
            return false;
        }
    }
    true
}

pub fn execute_events(
    snapshot: &mut SaveSnapshot,
    trigger_type: &str,
    context: &BTreeMap<String, Value>,
) -> Result<(Vec<String>, Vec<String>), String> {
    for event in &snapshot.events {
        validate_game_event(event)?;
    }

    let mut triggered_ids = Vec::new();
    let mut changes = Vec::new();
    let mut queue: Vec<String> = snapshot
        .events
        .iter()
        .filter(|event| event_match(event, trigger_type, context))
        .map(|event| event.id.clone())
        .collect();
    let mut visited = HashSet::new();
    let mut depth = 0usize;

    while let Some(event_id) = queue.pop() {
        if depth >= EVENT_CHAIN_MAX_DEPTH {
            break;
        }
        if visited.contains(&event_id) {
            continue;
        }
        let Some(event) = snapshot
            .events
            .iter()
            .find(|event| event.id == event_id)
            .cloned()
        else {
            continue;
        };
        let allow = event
            .guards
            .iter()
            .all(|guard| evaluate_guard(snapshot, &guard.expr));
        if !allow {
            depth += 1;
            continue;
        }

        visited.insert(event.id.clone());
        triggered_ids.push(event.id.clone());
        if !snapshot.active_event_ids.contains(&event.id) {
            snapshot.active_event_ids.push(event.id.clone());
        }

        for action in &event.actions {
            apply_event_action(snapshot, action, &mut changes);
        }
        for next_id in &event.next_event_ids {
            queue.push(next_id.clone());
        }

        depth += 1;
    }

    Ok((triggered_ids, changes))
}

pub fn apply_manual_event(
    snapshot: &mut SaveSnapshot,
    event_id: &str,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut context = BTreeMap::new();
    context.insert("eventId".to_string(), Value::String(event_id.to_string()));

    let exists = snapshot.events.iter().any(|event| event.id == event_id);
    if exists {
        let mut triggered = vec![event_id.to_string()];
        let mut changes = Vec::new();
        if let Some(event) = snapshot
            .events
            .iter()
            .find(|event| event.id == event_id)
            .cloned()
        {
            for action in &event.actions {
                apply_event_action(snapshot, action, &mut changes);
            }
            triggered.extend(event.next_event_ids);
        }
        if !snapshot.active_event_ids.contains(&event_id.to_string()) {
            snapshot.active_event_ids.push(event_id.to_string());
        }
        return Ok((triggered, changes));
    }

    execute_events(snapshot, "manual", &context)
}

fn build_turn_state_changes(turn_input: &TurnInput, event_changes: Vec<String>) -> Vec<String> {
    let mut changes = vec!["回合推进 +1".to_string()];
    if let Some(option_id) = turn_input.option_id.as_ref() {
        if option_id.contains("emotion") {
            changes.push("社交倾向提升".to_string());
        } else if option_id.contains("risk") {
            changes.push("风险压力上升".to_string());
        } else {
            changes.push("主线推进度提升".to_string());
        }
    } else if turn_input.custom_text.is_some() {
        changes.push("自定义行为触发自由演算".to_string());
    }
    changes.extend(event_changes);
    changes
}

fn persist_turn_result(
    paths: &AppPaths,
    turn_input: TurnInput,
    mut snapshot: SaveSnapshot,
    mut meta: crate::domain::SaveMeta,
    result: TurnResult,
) -> Result<TurnResult, String> {
    if turn_input.draft {
        return Ok(result);
    }

    snapshot.turn += 1;
    let log = EventLogEntry {
        turn: snapshot.turn,
        timestamp: now_iso(),
        input: turn_input.clone(),
        output: result.clone(),
        triggered_event_ids: result.triggered_event_ids.clone(),
        state_diff: result.state_diff.clone(),
    };
    validate_event_log_entry(&log)?;

    meta.current_turn = snapshot.turn;
    meta.updated_at = now_iso();

    write_snapshot(paths, &snapshot)?;
    write_meta(paths, &meta)?;
    append_ndjson(
        &paths.save_dir(&snapshot.save_id).join("events.ndjson"),
        &log,
    )?;

    Ok(result)
}

pub async fn run_turn_with_provider(
    paths: &AppPaths,
    turn_input: TurnInput,
) -> Result<TurnResult, String> {
    let mut snapshot = load_snapshot(paths, &turn_input.save_id)?;
    validate_save_snapshot(&snapshot)?;
    let meta = load_meta(paths, &turn_input.save_id)?;
    let runtime_config = resolve_runtime_model_config(paths, &snapshot)?;
    let selected = turn_input
        .custom_text
        .clone()
        .or(turn_input.option_id.clone())
        .unwrap_or_else(|| "观察周围".to_string());

    let mut event_context = BTreeMap::new();
    event_context.insert(
        "locationId".to_string(),
        Value::String(snapshot.current_location_id.clone()),
    );
    event_context.insert("actionText".to_string(), Value::String(selected.clone()));

    let (triggered_event_ids, event_changes) =
        execute_events(&mut snapshot, "on_turn_elapsed", &event_context)?;
    let (event_prompts, chapter_prompt) = load_prompt_context(paths, &meta, &snapshot);
    let recent_logs = collect_recent_logs(paths, &turn_input.save_id, 5)?;
    let world_card = load_world_card(paths, &meta);
    let prompt = build_turn_generation_context(
        &snapshot,
        &selected,
        &recent_logs,
        world_card.as_ref(),
        &chapter_prompt,
        &event_prompts,
    );
    let raw = generate_turn_json(&runtime_config, &prompt).await?;
    let proposal = parse_turn_proposal(&raw)?;
    let narration = proposal.narration.clone();

    let next_turn = snapshot.turn + 1;
    push_short_memory(&mut snapshot, format!("T{}: {}", next_turn, selected));
    snapshot.mid_term_summary = trim_by_chars(
        &format!("{}\n最新回合：{}", snapshot.mid_term_summary, narration),
        500,
    );

    let (ai_applied_changes, accepted_relationship_deltas) =
        apply_ai_proposal_with_guardrails(&mut snapshot, &proposal);
    let mut state_changes_preview = build_turn_state_changes(&turn_input, event_changes.clone());
    state_changes_preview.extend(proposal.state_changes_preview.clone());
    state_changes_preview.extend(ai_applied_changes.clone());
    state_changes_preview.dedup();

    let state_diff = json!({
        "turn": {
            "from": snapshot.turn,
            "to": snapshot.turn + 1
        },
        "stateChanges": state_changes_preview,
        "worldVariables": &snapshot.world_variables,
        "relationships": &snapshot.relationships,
        "quests": &snapshot.quests,
    });

    let result = TurnResult {
        narration,
        options: proposal.options.clone(),
        state_changes_preview,
        event_hints: triggered_event_ids
            .iter()
            .map(|id| format!("可能触发：{}", id))
            .chain(proposal.event_hints.iter().cloned())
            .collect(),
        triggered_event_ids,
        state_diff,
        story_state: proposal.story_state.clone(),
        task_state: proposal.task_state.clone(),
        relationship_deltas: accepted_relationship_deltas,
        ai_meta: Some(AiMeta {
            model: runtime_config.model.clone(),
            parser: "serde_json".to_string(),
            raw_chars: raw.chars().count(),
        }),
    };

    persist_turn_result(paths, turn_input, snapshot, meta, result)
}

pub async fn run_turn_stream_with_provider(
    paths: &AppPaths,
    turn_input: TurnInput,
    on_event: &mut (dyn FnMut(TurnStreamEvent) -> Result<(), String> + Send),
) -> Result<TurnResult, String> {
    let mut snapshot = load_snapshot(paths, &turn_input.save_id)?;
    validate_save_snapshot(&snapshot)?;
    let meta = load_meta(paths, &turn_input.save_id)?;
    let runtime_config = resolve_runtime_model_config(paths, &snapshot)?;
    let selected = turn_input
        .custom_text
        .clone()
        .or(turn_input.option_id.clone())
        .unwrap_or_else(|| "观察周围".to_string());

    let mut event_context = BTreeMap::new();
    event_context.insert(
        "locationId".to_string(),
        Value::String(snapshot.current_location_id.clone()),
    );
    event_context.insert("actionText".to_string(), Value::String(selected.clone()));
    let (triggered_event_ids, event_changes) =
        execute_events(&mut snapshot, "on_turn_elapsed", &event_context)?;
    let (event_prompts, chapter_prompt) = load_prompt_context(paths, &meta, &snapshot);
    let recent_logs = collect_recent_logs(paths, &turn_input.save_id, 5)?;
    let world_card = load_world_card(paths, &meta);
    let prompt = build_turn_generation_context(
        &snapshot,
        &selected,
        &recent_logs,
        world_card.as_ref(),
        &chapter_prompt,
        &event_prompts,
    );

    on_event(TurnStreamEvent {
        phase: "preview".to_string(),
        event_type: Some("status".to_string()),
        chunk: None,
        data: Some(json!({"message":"AI回合生成中"})),
    })?;

    let mut streamed_raw = String::new();
    let mut seen_narration_chars = 0usize;
    let mut emitted_options_preview = false;
    let mut emitted_state_preview = false;
    let mut on_piece = |piece: TurnJsonStreamPiece| -> Result<(), String> {
        match piece {
            TurnJsonStreamPiece::Reasoning(chunk) => {
                on_event(TurnStreamEvent {
                    phase: "delta".to_string(),
                    event_type: Some("status".to_string()),
                    chunk: None,
                    data: Some(json!({ "reasoning": chunk })),
                })?;
            }
            TurnJsonStreamPiece::Content(chunk) => {
                streamed_raw.push_str(&chunk);
                on_event(TurnStreamEvent {
                    phase: "delta".to_string(),
                    event_type: Some("json_delta".to_string()),
                    chunk: Some(chunk),
                    data: None,
                })?;
                if let Ok(preview) = parse_turn_proposal(&streamed_raw) {
                    if let Some(delta) =
                        delta_from_seen_chars(&preview.narration, &mut seen_narration_chars)
                    {
                        on_event(TurnStreamEvent {
                            phase: "delta".to_string(),
                            event_type: Some("narration_delta".to_string()),
                            chunk: Some(delta),
                            data: None,
                        })?;
                    }
                    if !emitted_options_preview {
                        emitted_options_preview = true;
                        on_event(TurnStreamEvent {
                            phase: "preview".to_string(),
                            event_type: Some("options_preview".to_string()),
                            chunk: None,
                            data: Some(json!({ "options": preview.options })),
                        })?;
                    }
                    if !emitted_state_preview {
                        emitted_state_preview = true;
                        on_event(TurnStreamEvent {
                            phase: "preview".to_string(),
                            event_type: Some("state_preview".to_string()),
                            chunk: None,
                            data: Some(json!({
                                "storyState": preview.story_state,
                                "taskState": preview.task_state,
                                "relationshipDeltas": preview.relationship_deltas,
                                "stateChangesPreview": preview.state_changes_preview
                            })),
                        })?;
                    }
                }
            }
        }
        Ok(())
    };

    let raw = match stream_turn_json(&runtime_config, &prompt, &mut on_piece).await {
        Ok(text) => text,
        Err(stream_err) => {
            on_event(TurnStreamEvent {
                phase: "preview".to_string(),
                event_type: Some("status".to_string()),
                chunk: None,
                data: Some(json!({
                    "message": format!("流式通道异常，已自动切换为非流式：{stream_err}")
                })),
            })?;
            generate_turn_json(&runtime_config, &prompt)
                .await
                .map_err(|fallback_err| {
                    format!(
                        "流式生成失败({stream_err})，且非流式回退也失败({fallback_err})"
                    )
                })?
        }
    };
    let proposal = parse_turn_proposal(&raw)?;
    let narration = proposal.narration.clone();

    let next_turn = snapshot.turn + 1;
    push_short_memory(&mut snapshot, format!("T{}: {}", next_turn, selected));
    snapshot.mid_term_summary = trim_by_chars(
        &format!("{}\n最新回合：{}", snapshot.mid_term_summary, narration),
        500,
    );

    let (ai_applied_changes, accepted_relationship_deltas) =
        apply_ai_proposal_with_guardrails(&mut snapshot, &proposal);
    let mut state_changes_preview = build_turn_state_changes(&turn_input, event_changes);
    state_changes_preview.extend(proposal.state_changes_preview.clone());
    state_changes_preview.extend(ai_applied_changes);
    state_changes_preview.dedup();

    let state_diff = json!({
        "turn": {
            "from": snapshot.turn,
            "to": snapshot.turn + 1
        },
        "worldVariables": &snapshot.world_variables,
        "relationships": &snapshot.relationships,
        "quests": &snapshot.quests,
    });

    let result = TurnResult {
        narration,
        options: proposal.options.clone(),
        state_changes_preview,
        event_hints: triggered_event_ids
            .iter()
            .map(|id| format!("可能触发：{}", id))
            .chain(proposal.event_hints.iter().cloned())
            .collect(),
        triggered_event_ids,
        state_diff,
        story_state: proposal.story_state.clone(),
        task_state: proposal.task_state.clone(),
        relationship_deltas: accepted_relationship_deltas,
        ai_meta: Some(AiMeta {
            model: runtime_config.model.clone(),
            parser: "serde_json".to_string(),
            raw_chars: raw.chars().count(),
        }),
    };

    on_event(TurnStreamEvent {
        phase: "final".to_string(),
        event_type: Some("status".to_string()),
        chunk: None,
        data: Some(json!({"message":"回合生成完成"})),
    })?;
    persist_turn_result(paths, turn_input, snapshot, meta, result)
}

pub fn run_enter_location_events(
    snapshot: &mut SaveSnapshot,
    location_id: &str,
) -> Result<EventResult, String> {
    let mut context = BTreeMap::new();
    context.insert(
        "locationId".to_string(),
        Value::String(location_id.to_string()),
    );
    let (triggered_ids, changes) = execute_events(snapshot, "on_enter_location", &context)?;
    Ok(EventResult {
        triggered: !triggered_ids.is_empty(),
        event_id: triggered_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "evt_on_enter_location".to_string()),
        message: if triggered_ids.is_empty() {
            "未命中地点事件".to_string()
        } else {
            format!("触发 {} 个地点事件", triggered_ids.len())
        },
        state_changes: changes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{EventAction, GuardCondition, TriggerCondition};

    fn test_snapshot() -> SaveSnapshot {
        SaveSnapshot {
            save_id: "save_1".to_string(),
            turn: 0,
            current_location_id: "loc_a".to_string(),
            player_role: "role".to_string(),
            relationships: serde_json::Map::new(),
            world_summary: "summary".to_string(),
            locations: vec![LocationNode {
                id: "loc_a".to_string(),
                name: "A".to_string(),
                x: 0.0,
                y: 0.0,
                tags: vec![],
                npc_ids: vec![],
                event_ids: vec![],
            }],
            paths: vec![],
            model_profile_id: "model_1".to_string(),
            model_label: "openai/gpt".to_string(),
            active_event_ids: vec![],
            world_variables: BTreeMap::new(),
            quests: vec![],
            events: vec![GameEvent {
                id: "evt_1".to_string(),
                name: "inc threat".to_string(),
                trigger: TriggerCondition {
                    r#type: "on_turn_elapsed".to_string(),
                    params: BTreeMap::new(),
                },
                guards: vec![GuardCondition {
                    expr: "".to_string(),
                }],
                actions: vec![EventAction {
                    r#type: "inc_variable".to_string(),
                    params: BTreeMap::from([
                        ("key".to_string(), Value::String("threat".to_string())),
                        ("delta".to_string(), Value::from(2.0)),
                    ]),
                }],
                cooldown_turns: None,
                next_event_ids: vec![],
            }],
            short_term_memory: vec![],
            mid_term_summary: String::new(),
            fact_locks: vec![],
        }
    }

    #[test]
    fn execute_events_applies_variable_increment() {
        let mut snapshot = test_snapshot();
        let context = BTreeMap::new();
        let (triggered, _changes) =
            execute_events(&mut snapshot, "on_turn_elapsed", &context).expect("execute events");
        assert_eq!(triggered, vec!["evt_1".to_string()]);
        let threat = snapshot
            .world_variables
            .get("threat")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        assert_eq!(threat, 2.0);
    }

    #[test]
    fn generate_world_from_card_v2_projects_nodes_edges() {
        let card = default_world_cards()
            .into_iter()
            .find(|item| item.id == "fantasy_realm")
            .expect("default card");
        let out = generate_world_from_card(&card, "测试者");
        assert!(!out.locations.is_empty());
        assert!(!out.paths.is_empty());
        assert_eq!(out.locations[0].id, card.map.nodes[0].id);
        assert!(
            out.paths.iter().any(|edge| {
                edge.from == "loc_gate" && edge.to == "loc_square"
                    || edge.from == "loc_square" && edge.to == "loc_gate"
            })
        );
    }
}
