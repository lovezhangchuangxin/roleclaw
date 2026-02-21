use crate::domain::{
    CardPromptEvent, CharacterArchetype, DialogueOption, EventAction, EventLogEntry, EventResult,
    GameEvent, LocationNode, MapCanvas, MapEdge, MapNode, ModelProviderConfig, NpcProfile,
    PathEdge, QuestState, SaveSnapshot, TriggerCondition, TurnInput, TurnResult, WorldBook,
    WorldCard, WorldInit, WorldMap,
};
use crate::llm::{generate_narration, stream_narration};
use crate::storage::{
    append_ndjson, load_global_data, load_meta, load_snapshot, now_iso, read_json, write_meta,
    write_snapshot, AppPaths,
};
use crate::validate::{validate_event_log_entry, validate_game_event, validate_save_snapshot};
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashSet};

const EVENT_CHAIN_MAX_DEPTH: usize = 6;

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

fn build_turn_options(snapshot: &SaveSnapshot) -> Vec<DialogueOption> {
    let loc_name = snapshot
        .locations
        .iter()
        .find(|loc| loc.id == snapshot.current_location_id)
        .map(|loc| loc.name.as_str())
        .unwrap_or("当前位置");
    vec![
        DialogueOption {
            id: "opt_plot_1".to_string(),
            kind: "plot".to_string(),
            text: format!("围绕{}追查主线线索", loc_name),
        },
        DialogueOption {
            id: "opt_emotion_1".to_string(),
            kind: "emotion".to_string(),
            text: "尝试与关键 NPC 建立信任".to_string(),
        },
        DialogueOption {
            id: "opt_risk_1".to_string(),
            kind: "risk".to_string(),
            text: "冒险探索高风险区域以获取突破".to_string(),
        },
    ]
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

fn build_turn_prompt(
    snapshot: &SaveSnapshot,
    selected: &str,
    chapter_prompt: &str,
    event_prompts: &[String],
) -> String {
    let system_layer = "[System]\n你是 AI RPG 引擎。仅输出叙事文本。保持事实一致，不违背事实锁。";
    let world_layer = format!(
        "[World]\nsummary={}\nvars={}\nfacts={}",
        trim_by_chars(&snapshot.world_summary, 450),
        snapshot.world_variables.len(),
        snapshot.fact_locks.join(" | ")
    );
    let save_layer = format!(
        "[Save]\nturn={}\nlocation={}\nmemory={}\nmid={}",
        snapshot.turn,
        snapshot.current_location_id,
        snapshot.short_term_memory.join(" || "),
        trim_by_chars(&snapshot.mid_term_summary, 240)
    );
    let turn_layer = format!(
        "[Turn]\nplayerRole={}\nplayerAction={}\nchapterGoal={}\neventPrompts={}\n请输出 120-260 字中文叙事，体现环境反馈、NPC反应和可执行线索。",
        snapshot.player_role,
        selected,
        chapter_prompt,
        event_prompts.join(" || ")
    );

    // Token budget fallback: hard trim full prompt to avoid uncontrolled growth.
    trim_by_chars(
        &format!(
            "{}\n{}\n{}\n{}",
            system_layer, world_layer, save_layer, turn_layer
        ),
        3600,
    )
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
        timeout_ms: profile.timeout_ms,
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
    let prompt = build_turn_prompt(&snapshot, &selected, &chapter_prompt, &event_prompts);
    let narration = generate_narration(&runtime_config, &prompt).await?;

    let next_turn = snapshot.turn + 1;
    push_short_memory(&mut snapshot, format!("T{}: {}", next_turn, selected));
    snapshot.mid_term_summary = trim_by_chars(
        &format!("{}\n最新回合：{}", snapshot.mid_term_summary, narration),
        500,
    );

    let state_changes_preview = build_turn_state_changes(&turn_input, event_changes.clone());
    let state_diff = json!({
        "turn": {
            "from": snapshot.turn,
            "to": snapshot.turn + 1
        },
        "stateChanges": state_changes_preview,
        "worldVariables": snapshot.world_variables,
    });

    let result = TurnResult {
        narration,
        options: build_turn_options(&snapshot),
        state_changes_preview,
        event_hints: triggered_event_ids
            .iter()
            .map(|id| format!("可能触发：{}", id))
            .collect(),
        triggered_event_ids,
        state_diff,
    };

    persist_turn_result(paths, turn_input, snapshot, meta, result)
}

pub async fn run_turn_stream_with_provider(
    paths: &AppPaths,
    turn_input: TurnInput,
    on_chunk: &mut (dyn FnMut(&str) -> Result<(), String> + Send),
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

    let narration = stream_narration(
        &runtime_config,
        &build_turn_prompt(&snapshot, &selected, &chapter_prompt, &event_prompts),
        on_chunk,
    )
    .await?;

    let next_turn = snapshot.turn + 1;
    push_short_memory(&mut snapshot, format!("T{}: {}", next_turn, selected));

    let state_changes_preview = build_turn_state_changes(&turn_input, event_changes);
    let state_diff = json!({
        "turn": {
            "from": snapshot.turn,
            "to": snapshot.turn + 1
        },
        "worldVariables": snapshot.world_variables,
    });

    let result = TurnResult {
        narration,
        options: build_turn_options(&snapshot),
        state_changes_preview,
        event_hints: triggered_event_ids
            .iter()
            .map(|id| format!("可能触发：{}", id))
            .collect(),
        triggered_event_ids,
        state_diff,
    };
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
