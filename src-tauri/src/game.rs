use crate::domain::{
    CharacterArchetype, DialogueOption, EventLogEntry, PathEdge, TurnInput, TurnResult, WorldCard, WorldInit, WorldRule,
};
use crate::llm::{generate_narration, stream_narration};
use crate::storage::{append_ndjson, load_meta, load_snapshot, now_iso, write_meta, write_snapshot, AppPaths};
use serde_json::json;

pub fn default_world_cards() -> Vec<WorldCard> {
    vec![
        WorldCard {
            id: "fantasy_realm".to_string(),
            name: "破碎王座".to_string(),
            schema_version: "1.0.0".to_string(),
            content_version: 1,
            genre: "fantasy".to_string(),
            tone: "epic-dark".to_string(),
            rules: vec![WorldRule {
                id: "rule_1".to_string(),
                title: "王国法则".to_string(),
                content: "王都秩序由三大议会和王室共同维持。".to_string(),
                priority: 100,
            }],
            location_pool: vec![
                crate::domain::LocationNode {
                    id: "loc_gate".to_string(),
                    name: "北门".to_string(),
                    x: 120.0,
                    y: 140.0,
                    tags: vec!["city".to_string(), "checkpoint".to_string()],
                    npc_ids: vec!["npc_guard".to_string()],
                    event_ids: vec!["evt_enter_gate".to_string()],
                },
                crate::domain::LocationNode {
                    id: "loc_square".to_string(),
                    name: "钟楼广场".to_string(),
                    x: 320.0,
                    y: 220.0,
                    tags: vec!["city".to_string(), "crowded".to_string()],
                    npc_ids: vec!["npc_bard".to_string()],
                    event_ids: vec!["evt_square_rumor".to_string()],
                },
                crate::domain::LocationNode {
                    id: "loc_tower".to_string(),
                    name: "旧法师塔".to_string(),
                    x: 520.0,
                    y: 120.0,
                    tags: vec!["mystic".to_string(), "danger".to_string()],
                    npc_ids: vec!["npc_mage".to_string()],
                    event_ids: vec!["evt_tower_lock".to_string()],
                },
            ],
            archetype_pool: vec![
                CharacterArchetype {
                    id: "npc_guard".to_string(),
                    name: "守卫长卡恩".to_string(),
                    traits: vec!["警惕".to_string(), "讲规则".to_string()],
                    motivation: "守住北门".to_string(),
                    secret: Some("曾见过王室密令".to_string()),
                },
                CharacterArchetype {
                    id: "npc_bard".to_string(),
                    name: "吟游诗人米拉".to_string(),
                    traits: vec!["健谈".to_string(), "圆滑".to_string()],
                    motivation: "收集流言换取报酬".to_string(),
                    secret: Some("与地下组织有联络".to_string()),
                },
            ],
        },
        WorldCard {
            id: "cyber_city".to_string(),
            name: "霓虹深井".to_string(),
            schema_version: "1.0.0".to_string(),
            content_version: 1,
            genre: "cyberpunk".to_string(),
            tone: "gritty-noir".to_string(),
            rules: vec![WorldRule {
                id: "rule_1".to_string(),
                title: "算力即权力".to_string(),
                content: "城市由四家算力财团分区统治，数据即法律。".to_string(),
                priority: 100,
            }],
            location_pool: vec![
                crate::domain::LocationNode {
                    id: "loc_dock".to_string(),
                    name: "灰港接入站".to_string(),
                    x: 130.0,
                    y: 260.0,
                    tags: vec!["port".to_string(), "black-market".to_string()],
                    npc_ids: vec!["npc_broker".to_string()],
                    event_ids: vec![],
                },
                crate::domain::LocationNode {
                    id: "loc_tower".to_string(),
                    name: "主核塔".to_string(),
                    x: 430.0,
                    y: 120.0,
                    tags: vec!["corp".to_string(), "restricted".to_string()],
                    npc_ids: vec!["npc_exec".to_string()],
                    event_ids: vec![],
                },
            ],
            archetype_pool: vec![CharacterArchetype {
                id: "npc_broker".to_string(),
                name: "中间人 R-9".to_string(),
                traits: vec!["理性".to_string(), "逐利".to_string()],
                motivation: "撮合高风险交易".to_string(),
                secret: Some("掌握一份泄露密钥".to_string()),
            }],
        },
    ]
}

pub fn generate_world_from_card(card: &WorldCard, player_role: &str) -> WorldInit {
    let locations = if card.location_pool.is_empty() {
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
        card.location_pool.clone()
    };

    let mut paths: Vec<PathEdge> = Vec::new();
    if locations.len() > 1 {
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
            "你进入了《{}》世界。你的身份是“{}”。世界基调：{}。",
            card.name, player_role, card.tone
        ),
        main_npcs: card.archetype_pool.clone(),
        locations,
        paths,
        quest_hooks: vec![
            "调查第一条异动线索".to_string(),
            "建立与关键 NPC 的初始信任".to_string(),
            "找出影响当前区域的核心冲突".to_string(),
        ],
    }
}

fn build_turn_options() -> Vec<DialogueOption> {
    vec![
        DialogueOption {
            id: "opt_plot_1".to_string(),
            kind: "plot".to_string(),
            text: "追问关键线索，推动主线".to_string(),
        },
        DialogueOption {
            id: "opt_emotion_1".to_string(),
            kind: "emotion".to_string(),
            text: "尝试建立信任并交换信息".to_string(),
        },
        DialogueOption {
            id: "opt_risk_1".to_string(),
            kind: "risk".to_string(),
            text: "冒险试探禁区，获取高价值情报".to_string(),
        },
    ]
}

fn build_state_changes(turn_input: &TurnInput) -> Vec<String> {
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
    changes
}

fn build_turn_prompt(location: &str, player_role: &str, selected: &str) -> String {
    format!(
        "你正在进行文字 RPG。\n玩家身份：{player_role}\n当前位置：{location}\n玩家本回合行为：{selected}\n请输出 120-260 字的中文叙事，体现环境反馈、NPC反应、潜在线索。"
    )
}

fn persist_turn_result(
    paths: &AppPaths,
    turn_input: TurnInput,
    mut snapshot: crate::domain::SaveSnapshot,
    mut meta: crate::domain::SaveMeta,
    result: TurnResult,
) -> Result<TurnResult, String> {
    snapshot.turn += 1;
    let log = EventLogEntry {
        turn: snapshot.turn,
        timestamp: now_iso(),
        input: turn_input.clone(),
        output: result.clone(),
        triggered_event_ids: vec!["evt_generic_turn".to_string()],
        state_diff: json!({
            "turn": {
                "from": snapshot.turn - 1,
                "to": snapshot.turn
            }
        }),
    };
    meta.current_turn = snapshot.turn;
    meta.updated_at = now_iso();

    write_snapshot(paths, &snapshot)?;
    write_meta(paths, &meta)?;
    append_ndjson(&paths.save_dir(&snapshot.save_id).join("events.ndjson"), &log)?;

    Ok(result)
}

pub async fn run_turn_with_provider(paths: &AppPaths, turn_input: TurnInput) -> Result<TurnResult, String> {
    let snapshot = load_snapshot(paths, &turn_input.save_id)?;
    let meta = load_meta(paths, &turn_input.save_id)?;
    let selected = turn_input
        .custom_text
        .clone()
        .or(turn_input.option_id.clone())
        .unwrap_or_else(|| "观察周围".to_string());

    let narration = generate_narration(
        &snapshot.model_config,
        &build_turn_prompt(&snapshot.current_location_id, &snapshot.player_role, &selected),
    )
    .await?;
    let result = TurnResult {
        narration,
        options: build_turn_options(),
        state_changes_preview: build_state_changes(&turn_input),
        event_hints: vec!["可能触发：地点关联事件".to_string()],
    };

    persist_turn_result(paths, turn_input, snapshot, meta, result)
}

pub async fn run_turn_stream_with_provider(
    paths: &AppPaths,
    turn_input: TurnInput,
    on_chunk: &mut (dyn FnMut(&str) -> Result<(), String> + Send),
) -> Result<TurnResult, String> {
    let snapshot = load_snapshot(paths, &turn_input.save_id)?;
    let meta = load_meta(paths, &turn_input.save_id)?;
    let selected = turn_input
        .custom_text
        .clone()
        .or(turn_input.option_id.clone())
        .unwrap_or_else(|| "观察周围".to_string());

    let narration = stream_narration(
        &snapshot.model_config,
        &build_turn_prompt(&snapshot.current_location_id, &snapshot.player_role, &selected),
        on_chunk,
    )
    .await?;

    let result = TurnResult {
        narration,
        options: build_turn_options(),
        state_changes_preview: build_state_changes(&turn_input),
        event_hints: vec!["可能触发：地点关联事件".to_string()],
    };
    persist_turn_result(paths, turn_input, snapshot, meta, result)
}
