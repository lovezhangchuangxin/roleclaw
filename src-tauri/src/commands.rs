use crate::domain::{
    AiGenerateWorldCardInput, AiModelProfile, AiSettings, ConnectivityResult, CreateSaveConfig,
    EventLogEntry, EventLogPage, EventResult, GameSettings, GenerateWorldInput, GlobalGameData,
    MoveResult, ReplayConsistency, ReplayResult, ReplayTurn, SaveBundle, SaveMeta, SaveSnapshot,
    TriggerEventPayload, TurnInput, TurnResult, WorldCard, WorldInit,
};
use crate::error::AppError;
use crate::game::{
    apply_manual_event, default_world_cards, generate_world_from_card, project_card_events,
    run_enter_location_events, run_turn_stream_with_provider, run_turn_with_provider, TurnStreamEvent,
    seed_events_for_world,
};
use crate::llm::{generate_world_card_json, stream_world_card_json, test_provider_connectivity};
use crate::storage::{
    collect_recent_logs, list_events_page, load_all_logs, load_global_data, load_meta,
    load_snapshot, now_id, now_iso, read_json, write_global_data, write_json, write_meta,
    write_snapshot, AppPaths,
};
use crate::validate::validate_world_card;
use serde::Serialize;
use serde_json::Value;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter, Window};

type ApiResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TurnStreamEventPayload {
    stream_id: String,
    phase: String,
    event_type: Option<String>,
    chunk: Option<String>,
    data: Option<Value>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WorldCardStreamEventPayload {
    stream_id: String,
    phase: String,
    chunk: Option<String>,
}

fn map_storage_err(err: String) -> AppError {
    AppError::storage(err)
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

fn build_world_card_prompt(user_prompt: &str) -> String {
    format!(
        "请基于以下玩家意图，直接生成完整 WorldCard v2 JSON：\n{}\n\n要求：\
1) schemaVersion 固定为 \"2.0.0\"，contentVersion=1。\
2) 至少生成 4 个地图节点、3 条边、3 个 NPC、3 个事件、3 个章节目标。\
3) map.canvas 推荐 width=900,height=560。\
4) id 使用小写字母/数字/下划线，保证唯一。\
5) 仅输出 JSON 对象。",
        user_prompt.trim()
    )
}

fn parse_generated_world_card(raw: &str) -> Result<WorldCard, AppError> {
    let json_slice = extract_json_object(raw)
        .ok_or_else(|| AppError::provider("模型未返回合法 JSON 对象，请重试"))?;
    let value: Value = serde_json::from_str(json_slice)
        .map_err(|e| AppError::provider(format!("模型 JSON 解析失败: {e}")))?;
    let card = normalize_ai_world_card_value(value)?;
    validate_world_card(&card).map_err(AppError::validation)?;
    Ok(card)
}

fn normalize_ai_world_card_value(value: Value) -> Result<WorldCard, AppError> {
    let root = value
        .as_object()
        .ok_or_else(|| AppError::provider("模型返回的 JSON 根节点必须是对象"))?;

    let name = root
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("AI 世界卡")
        .trim()
        .to_string();
    let mut id = root
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("ai_world_card")
        .trim()
        .to_string();
    if id.is_empty() {
        id = "ai_world_card".to_string();
    }

    let schema_version = root
        .get("schemaVersion")
        .and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or("2.0.0")
        .to_string();
    let content_version = root
        .get("contentVersion")
        .and_then(Value::as_u64)
        .map(|v| v as u32)
        .unwrap_or(1);

    let worldbook = match root.get("worldbook") {
        Some(Value::String(text)) => crate::domain::WorldBook {
            title: name.clone(),
            overview: text.clone(),
            background: String::new(),
            core_conflicts: Vec::new(),
            play_style: String::new(),
        },
        Some(Value::Object(obj)) => crate::domain::WorldBook {
            title: obj
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(&name)
                .to_string(),
            overview: obj
                .get("overview")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            background: obj
                .get("background")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            core_conflicts: obj
                .get("coreConflicts")
                .and_then(Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(Value::as_str)
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
            play_style: obj
                .get("playStyle")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        },
        _ => crate::domain::WorldBook {
            title: name.clone(),
            overview: String::new(),
            background: String::new(),
            core_conflicts: Vec::new(),
            play_style: String::new(),
        },
    };

    let map_obj = root.get("map").and_then(Value::as_object);
    let canvas_obj = map_obj.and_then(|m| m.get("canvas")).and_then(Value::as_object);
    let canvas = crate::domain::MapCanvas {
        width: canvas_obj
            .and_then(|c| c.get("width"))
            .and_then(Value::as_u64)
            .map(|v| v as u32)
            .unwrap_or(900),
        height: canvas_obj
            .and_then(|c| c.get("height"))
            .and_then(Value::as_u64)
            .map(|v| v as u32)
            .unwrap_or(560),
    };

    let mut nodes = map_obj
        .and_then(|m| m.get("nodes"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    let id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("loc_{}", idx + 1));
                    let name = obj
                        .get("name")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("地点{}", idx + 1));
                    let description = obj
                        .get("description")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let tags = obj
                        .get("tags")
                        .and_then(Value::as_array)
                        .map(|tags| {
                            tags.iter()
                                .filter_map(Value::as_str)
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    let x = obj
                        .get("x")
                        .and_then(Value::as_f64)
                        .map(|v| v as f32)
                        .unwrap_or(120.0 + idx as f32 * 120.0);
                    let y = obj
                        .get("y")
                        .and_then(Value::as_f64)
                        .map(|v| v as f32)
                        .unwrap_or(120.0 + idx as f32 * 80.0);
                    Some(crate::domain::MapNode {
                        id,
                        name,
                        description,
                        tags,
                        x,
                        y,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if nodes.is_empty() {
        nodes.push(crate::domain::MapNode {
            id: "loc_start".to_string(),
            name: "起始地点".to_string(),
            description: String::new(),
            tags: Vec::new(),
            x: 180.0,
            y: 180.0,
        });
    }

    let node_ids = nodes.iter().map(|n| n.id.clone()).collect::<Vec<_>>();
    let mut edges = map_obj
        .and_then(|m| m.get("edges"))
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    let a = obj.get("a").and_then(Value::as_str)?.to_string();
                    let b = obj.get("b").and_then(Value::as_str)?.to_string();
                    let id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("edge_{}_{}_{}", a, b, idx + 1));
                    let locked = obj.get("locked").and_then(Value::as_bool).unwrap_or(false);
                    let unlock_conditions = obj
                        .get("unlockConditions")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_str)
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    Some(crate::domain::MapEdge {
                        id,
                        a,
                        b,
                        locked,
                        unlock_conditions,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    // If AI returned no edges while multiple nodes exist, auto-connect linearly.
    if edges.is_empty() && nodes.len() > 1 {
        for i in 0..(nodes.len() - 1) {
            edges.push(crate::domain::MapEdge {
                id: format!("edge_auto_{}_{}", nodes[i].id, nodes[i + 1].id),
                a: nodes[i].id.clone(),
                b: nodes[i + 1].id.clone(),
                locked: false,
                unlock_conditions: Vec::new(),
            });
        }
    }

    let start_node_id = map_obj
        .and_then(|m| m.get("startNodeId"))
        .and_then(Value::as_str)
        .filter(|id| node_ids.iter().any(|n| n == id))
        .map(|v| v.to_string())
        .unwrap_or_else(|| nodes[0].id.clone());

    let npcs = root
        .get("npcs")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    let id = obj
                        .get("id")
                        .and_then(Value::as_str)
                        .filter(|v| !v.trim().is_empty())
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("npc_{}", idx + 1));
                    let name = obj
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let identity = obj
                        .get("identity")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let personality = obj
                        .get("personality")
                        .and_then(Value::as_array)
                        .map(|arr| {
                            arr.iter()
                                .filter_map(Value::as_str)
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    Some(crate::domain::NpcProfile {
                        id,
                        name,
                        personality,
                        identity,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let events = root
        .get("events")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    Some(crate::domain::CardPromptEvent {
                        id: obj
                            .get("id")
                            .and_then(Value::as_str)
                            .filter(|v| !v.trim().is_empty())
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| format!("evt_{}", idx + 1)),
                        name: obj
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        prompt: obj
                            .get("prompt")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let chapter_goals = root
        .get("chapterGoals")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    let obj = item.as_object()?;
                    Some(crate::domain::ChapterGoal {
                        id: obj
                            .get("id")
                            .and_then(Value::as_str)
                            .filter(|v| !v.trim().is_empty())
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| format!("chapter_{}", idx + 1)),
                        title: obj
                            .get("title")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        prompt: obj
                            .get("prompt")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(WorldCard {
        id,
        name,
        schema_version,
        content_version,
        worldbook,
        map: crate::domain::WorldMap {
            nodes,
            edges,
            start_node_id,
            canvas,
        },
        npcs,
        events,
        chapter_goals,
    })
}

fn normalize_ai_settings(ai_settings: &mut AiSettings) {
    let mut used_ids = std::collections::HashSet::new();
    for (idx, model) in ai_settings.models.iter_mut().enumerate() {
        let missing = model.id.trim().is_empty();
        let duplicated = used_ids.contains(&model.id);
        if missing || duplicated {
            model.id = format!("{}_{}", now_id("model"), idx);
        }
        used_ids.insert(model.id.clone());
    }

    if ai_settings.models.is_empty() {
        ai_settings.default_model_id = None;
        return;
    }
    let default_exists = ai_settings
        .default_model_id
        .as_ref()
        .map(|id| ai_settings.models.iter().any(|m| &m.id == id))
        .unwrap_or(false);
    if !default_exists {
        ai_settings.default_model_id = ai_settings.models.first().map(|m| m.id.clone());
    }
}

fn profile_to_runtime_config(profile: &AiModelProfile) -> crate::domain::ModelProviderConfig {
    crate::domain::ModelProviderConfig {
        provider_type: profile.provider_type.clone(),
        provider: profile.provider.clone(),
        base_url: profile.base_url.clone(),
        model: profile.model.clone(),
        api_key: profile.api_key.clone(),
        temperature: profile.temperature,
        max_tokens: Some(profile.max_tokens),
        timeout_ms: profile.timeout_ms,
    }
}

fn profile_to_world_card_runtime_config(
    profile: &AiModelProfile,
) -> crate::domain::ModelProviderConfig {
    let mut config = profile_to_runtime_config(profile);
    if config.timeout_ms < 90_000 {
        config.timeout_ms = 90_000;
    }
    config
}

fn default_global_game_data() -> GlobalGameData {
    GlobalGameData {
        game_settings: GameSettings {
            theme: "default".to_string(),
            message_speed: "normal".to_string(),
            font_scale: 1.0,
            ui_zoom: 1.0,
            log_level: "info".to_string(),
        },
        ai_settings: AiSettings {
            default_model_id: None,
            models: Vec::new(),
        },
    }
}

fn build_replay_consistency(logs: &[EventLogEntry], snapshot_turn: u32) -> ReplayConsistency {
    let log_last_turn = logs.last().map(|item| item.turn).unwrap_or(0);
    let mut warnings = Vec::new();
    let mut is_monotonic = true;
    let mut prev = 0u32;
    for row in logs {
        if row.turn <= prev {
            is_monotonic = false;
            warnings.push(format!(
                "事件日志回合非严格递增：turn={} 出现在 turn={} 之后",
                row.turn, prev
            ));
            break;
        }
        prev = row.turn;
    }
    if log_last_turn != snapshot_turn {
        warnings.push(format!(
            "日志末回合({}) 与快照回合({}) 不一致",
            log_last_turn, snapshot_turn
        ));
    }
    ReplayConsistency {
        snapshot_turn,
        log_last_turn,
        is_monotonic,
        matches_snapshot: log_last_turn == snapshot_turn,
        warnings,
    }
}

pub fn ensure_default_world_cards(paths: &AppPaths) -> Result<(), String> {
    for card in default_world_cards() {
        let path = paths.world_cards_dir.join(format!("{}.json", card.id));
        if !path.exists() {
            write_json(&path, &card)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn list_world_cards(app: AppHandle) -> ApiResult<Vec<WorldCard>> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    crate::storage::list_world_cards(&paths).map_err(map_storage_err)
}

#[tauri::command]
pub fn get_global_game_data(app: AppHandle) -> ApiResult<GlobalGameData> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    match load_global_data(&paths) {
        Ok(mut data) => {
            normalize_ai_settings(&mut data.ai_settings);
            write_global_data(&paths, &data).map_err(map_storage_err)?;
            Ok(data)
        }
        Err(_) => {
            let data = default_global_game_data();
            write_global_data(&paths, &data).map_err(map_storage_err)?;
            Ok(data)
        }
    }
}

#[tauri::command]
pub fn update_global_game_data(data: GlobalGameData, app: AppHandle) -> ApiResult<GlobalGameData> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut next = data;
    normalize_ai_settings(&mut next.ai_settings);
    write_global_data(&paths, &next).map_err(map_storage_err)?;
    Ok(next)
}

#[tauri::command]
pub fn list_ai_models(app: AppHandle) -> ApiResult<AiSettings> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);
    write_global_data(&paths, &data).map_err(map_storage_err)?;
    Ok(data.ai_settings)
}

#[tauri::command]
pub fn upsert_ai_model(mut profile: AiModelProfile, app: AppHandle) -> ApiResult<AiModelProfile> {
    let id_empty = profile.id.trim().is_empty();
    if profile.provider_type.trim().is_empty()
        || profile.provider.trim().is_empty()
        || profile.base_url.trim().is_empty()
        || profile.model.trim().is_empty()
    {
        return Err(AppError::validation(
            "providerType/provider/baseUrl/model 均不能为空",
        ));
    }
    if profile.provider_type != "openai_compatible" {
        return Err(AppError::validation("当前仅支持 openai_compatible 协议"));
    }

    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);
    profile.updated_at = now_iso();

    let mut target_id = profile.id.clone();
    if id_empty {
        if let Some(existing) = data.ai_settings.models.iter().find(|item| {
            item.provider_type == profile.provider_type
                && item.provider == profile.provider
                && item.model == profile.model
                && item.base_url.trim_end_matches('/') == profile.base_url.trim_end_matches('/')
        }) {
            target_id = existing.id.clone();
        } else {
            target_id = now_id("model");
        }
    }
    profile.id = target_id.clone();

    if let Some(existing) = data
        .ai_settings
        .models
        .iter_mut()
        .find(|item| item.id == target_id)
    {
        *existing = profile.clone();
    } else {
        data.ai_settings.models.push(profile.clone());
    }
    if data.ai_settings.default_model_id.is_none() {
        data.ai_settings.default_model_id = Some(profile.id.clone());
    }
    write_global_data(&paths, &data).map_err(map_storage_err)?;
    Ok(profile)
}

#[tauri::command]
pub fn delete_ai_model(model_id: String, app: AppHandle) -> ApiResult<()> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);
    let before = data.ai_settings.models.len();
    data.ai_settings.models.retain(|item| item.id != model_id);
    if before == data.ai_settings.models.len() {
        return Err(AppError::not_found(format!("model not found: {model_id}")));
    }
    if data.ai_settings.models.is_empty() {
        data.ai_settings.default_model_id = None;
    } else if data.ai_settings.default_model_id.as_ref() == Some(&model_id) {
        data.ai_settings.default_model_id =
            data.ai_settings.models.first().map(|item| item.id.clone());
    }
    write_global_data(&paths, &data).map_err(map_storage_err)?;
    Ok(())
}

#[tauri::command]
pub fn set_default_ai_model(model_id: String, app: AppHandle) -> ApiResult<AiSettings> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);
    if !data
        .ai_settings
        .models
        .iter()
        .any(|item| item.id == model_id)
    {
        return Err(AppError::not_found(format!("model not found: {model_id}")));
    }
    data.ai_settings.default_model_id = Some(model_id);
    write_global_data(&paths, &data).map_err(map_storage_err)?;
    Ok(data.ai_settings)
}

#[tauri::command]
pub fn import_world_card(file: String, app: AppHandle) -> ApiResult<WorldCard> {
    let card: WorldCard = if file.trim_start().starts_with('{') {
        serde_json::from_str(&file)
            .map_err(|e| AppError::validation(format!("invalid world card JSON: {e}")))?
    } else {
        read_json(Path::new(&file)).map_err(map_storage_err)?
    };
    validate_world_card(&card).map_err(AppError::validation)?;
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let out_path = paths
        .world_cards_dir
        .join(format!("{}.json", card.id.clone()));
    write_json(&out_path, &card).map_err(map_storage_err)?;
    Ok(card)
}

#[tauri::command]
pub fn export_world_card(card_id: String, file: String, app: AppHandle) -> ApiResult<()> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let source = paths.world_cards_dir.join(format!("{card_id}.json"));
    if !source.exists() {
        return Err(AppError::not_found(format!(
            "world card not found: {card_id}"
        )));
    }
    let card: WorldCard = read_json(&source).map_err(map_storage_err)?;
    write_json(Path::new(&file), &card).map_err(map_storage_err)
}

#[tauri::command]
pub fn delete_world_card(card_id: String, app: AppHandle) -> ApiResult<()> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let target = paths.world_cards_dir.join(format!("{card_id}.json"));
    if !target.exists() {
        return Err(AppError::not_found(format!(
            "world card not found: {card_id}"
        )));
    }
    fs::remove_file(&target)
        .map_err(|e| AppError::storage(format!("failed to delete world card: {e}")))
}

#[tauri::command]
pub fn generate_world(input: GenerateWorldInput, app: AppHandle) -> ApiResult<WorldInit> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let card: WorldCard = read_json(
        &paths
            .world_cards_dir
            .join(format!("{}.json", input.world_card_id)),
    )
    .map_err(map_storage_err)?;
    validate_world_card(&card).map_err(AppError::validation)?;
    Ok(generate_world_from_card(&card, &input.player_role))
}

#[tauri::command]
pub fn create_save(config: CreateSaveConfig, app: AppHandle) -> ApiResult<SaveMeta> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    if config.save_name.trim().is_empty() {
        return Err(AppError::validation("saveName 不能为空"));
    }
    if config.player_role.trim().is_empty() {
        return Err(AppError::validation("playerRole 不能为空"));
    }
    if config.model_profile_id.trim().is_empty() {
        return Err(AppError::validation("modelProfileId 不能为空"));
    }
    let world_card_id = config.world_card_id.clone();
    let save_id = now_id("save");
    fs::create_dir_all(paths.save_dir(&save_id))
        .map_err(|e| AppError::storage(format!("failed to create save dir: {e}")))?;

    let world_init = if let Some(w) = config.world_init.clone() {
        w
    } else {
        let card: WorldCard = read_json(
            &paths
                .world_cards_dir
                .join(format!("{}.json", world_card_id)),
        )
        .map_err(map_storage_err)?;
        validate_world_card(&card).map_err(AppError::validation)?;
        generate_world_from_card(&card, &config.player_role)
    };
    let starting_location = world_init
        .locations
        .first()
        .map(|v| v.id.clone())
        .unwrap_or_else(|| "loc_origin".to_string());
    let timestamp = now_iso();
    let mut global_data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut global_data.ai_settings);
    let profile = global_data
        .ai_settings
        .models
        .iter()
        .find(|item| item.id == config.model_profile_id)
        .ok_or_else(|| AppError::validation("所选 AI 模型不存在，请在 AI 设置中重新选择"))?;

    let meta = SaveMeta {
        id: save_id.clone(),
        name: config.save_name,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        world_card_id: world_card_id.clone(),
        current_turn: 0,
        player_role: config.player_role.clone(),
        model_profile_id: profile.id.clone(),
        provider: profile.provider.clone(),
        model: profile.model.clone(),
        parent_save_id: None,
        forked_from_turn: None,
    };
    let seeded_events = if let Ok(card) = read_json::<WorldCard>(
        &paths
            .world_cards_dir
            .join(format!("{}.json", world_card_id)),
    ) {
        let projected = project_card_events(&card);
        if projected.is_empty() {
            seed_events_for_world(&world_init.locations)
        } else {
            projected
        }
    } else {
        seed_events_for_world(&world_init.locations)
    };
    let snapshot = SaveSnapshot {
        save_id: save_id.clone(),
        turn: 0,
        current_location_id: starting_location,
        player_role: config.player_role,
        relationships: serde_json::Map::new(),
        world_summary: world_init.world_summary,
        locations: world_init.locations,
        paths: world_init.paths,
        model_profile_id: profile.id.clone(),
        model_label: format!("{}/{}", profile.provider, profile.model),
        active_event_ids: Vec::new(),
        world_variables: std::collections::BTreeMap::new(),
        quests: Vec::new(),
        events: seeded_events,
        short_term_memory: Vec::new(),
        mid_term_summary: String::new(),
        fact_locks: Vec::new(),
    };
    write_meta(&paths, &meta).map_err(map_storage_err)?;
    write_snapshot(&paths, &snapshot).map_err(map_storage_err)?;
    Ok(meta)
}

#[tauri::command]
pub fn list_saves(app: AppHandle) -> ApiResult<Vec<SaveMeta>> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    crate::storage::list_saves(&paths).map_err(map_storage_err)
}

#[tauri::command]
pub fn load_save(save_id: String, app: AppHandle) -> ApiResult<SaveBundle> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let save_path = paths.save_dir(&save_id);
    if !save_path.exists() {
        return Err(AppError::not_found(format!("save not found: {save_id}")));
    }
    let meta = load_meta(&paths, &save_id).map_err(map_storage_err)?;
    let snapshot = load_snapshot(&paths, &save_id).map_err(map_storage_err)?;
    let recent_logs = collect_recent_logs(&paths, &save_id, 20).map_err(map_storage_err)?;
    Ok(SaveBundle {
        meta,
        snapshot,
        recent_logs,
    })
}

#[tauri::command]
pub fn delete_save(save_id: String, app: AppHandle) -> ApiResult<()> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let dir = paths.save_dir(&save_id);
    if !dir.exists() {
        return Err(AppError::not_found(format!("save not found: {save_id}")));
    }
    fs::remove_dir_all(&dir).map_err(|e| AppError::storage(format!("failed to delete save: {e}")))
}

#[tauri::command]
pub fn exit_app(app: AppHandle) -> ApiResult<()> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn move_to_location(
    save_id: String,
    location_id: String,
    app: AppHandle,
) -> ApiResult<MoveResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut snapshot = load_snapshot(&paths, &save_id).map_err(map_storage_err)?;
    let current = snapshot.current_location_id.clone();
    let reachable = snapshot.paths.iter().any(|edge| {
        !edge.locked
            && ((edge.from == current && edge.to == location_id)
                || (edge.from == location_id && edge.to == current))
    });
    if !reachable {
        return Ok(MoveResult {
            moved: false,
            current_location_id: current,
            message: "该地点当前不可达".to_string(),
            triggered_event_ids: Vec::new(),
        });
    }

    snapshot.current_location_id = location_id.clone();
    let event_result =
        run_enter_location_events(&mut snapshot, &location_id).map_err(AppError::provider)?;
    write_snapshot(&paths, &snapshot).map_err(map_storage_err)?;
    Ok(MoveResult {
        moved: true,
        current_location_id: location_id,
        message: event_result.message,
        triggered_event_ids: if event_result.triggered {
            vec![event_result.event_id]
        } else {
            Vec::new()
        },
    })
}

#[tauri::command]
pub async fn run_turn(turn_input: TurnInput, app: AppHandle) -> ApiResult<TurnResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    run_turn_with_provider(&paths, turn_input)
        .await
        .map_err(AppError::provider)
}

#[tauri::command]
pub async fn run_turn_stream(
    turn_input: TurnInput,
    stream_id: String,
    window: Window,
    app: AppHandle,
) -> ApiResult<TurnResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let begin = TurnStreamEventPayload {
        stream_id: stream_id.clone(),
        phase: "start".to_string(),
        event_type: Some("status".to_string()),
        chunk: None,
        data: Some(serde_json::json!({ "message": "stream-start" })),
    };
    let _ = window.emit("turn_stream_chunk", begin);
    let mut emitter = |event: TurnStreamEvent| -> Result<(), String> {
        let payload = TurnStreamEventPayload {
            stream_id: stream_id.clone(),
            phase: event.phase,
            event_type: event.event_type,
            chunk: event.chunk,
            data: event.data,
        };
        window
            .emit("turn_stream_chunk", payload)
            .map_err(|e| format!("emit stream chunk failed: {e}"))
    };

    let result = match run_turn_stream_with_provider(&paths, turn_input, &mut emitter).await {
        Ok(value) => value,
        Err(err) => {
            let error_payload = TurnStreamEventPayload {
                stream_id: stream_id.clone(),
                phase: "error".to_string(),
                event_type: Some("error".to_string()),
                chunk: None,
                data: Some(serde_json::json!({ "message": err })),
            };
            let _ = window.emit("turn_stream_chunk", error_payload);
            return Err(AppError::provider(err));
        }
    };

    let end = TurnStreamEventPayload {
        stream_id,
        phase: "end".to_string(),
        event_type: Some("status".to_string()),
        chunk: None,
        data: Some(serde_json::json!({ "message": "stream-end" })),
    };
    let _ = window.emit("turn_stream_chunk", end);
    Ok(result)
}

#[tauri::command]
pub fn trigger_event(payload: TriggerEventPayload, app: AppHandle) -> ApiResult<EventResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut snapshot = load_snapshot(&paths, &payload.save_id).map_err(map_storage_err)?;
    let (triggered_ids, changes) =
        apply_manual_event(&mut snapshot, &payload.event_id).map_err(AppError::provider)?;
    write_snapshot(&paths, &snapshot).map_err(map_storage_err)?;
    Ok(EventResult {
        triggered: !triggered_ids.is_empty(),
        event_id: triggered_ids
            .first()
            .cloned()
            .unwrap_or_else(|| payload.event_id.clone()),
        message: if triggered_ids.is_empty() {
            "未命中事件规则".to_string()
        } else {
            format!("事件执行完成（{}）", triggered_ids.join(","))
        },
        state_changes: changes,
    })
}

#[tauri::command]
pub fn list_events(
    save_id: String,
    cursor: Option<u32>,
    app: AppHandle,
) -> ApiResult<EventLogPage> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    list_events_page(&paths, &save_id, cursor, 30).map_err(map_storage_err)
}

#[tauri::command]
pub fn replay_save(
    save_id: String,
    until_turn: Option<u32>,
    app: AppHandle,
) -> ApiResult<ReplayResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let all_logs = load_all_logs(&paths, &save_id).map_err(map_storage_err)?;
    let snapshot = load_snapshot(&paths, &save_id).map_err(map_storage_err)?;
    let total_turns = all_logs.last().map(|item| item.turn).unwrap_or(0);
    let stop = until_turn.unwrap_or(total_turns);
    let consistency = build_replay_consistency(&all_logs, snapshot.turn);
    let turns = all_logs
        .into_iter()
        .filter(|row| row.turn <= stop)
        .map(|row| ReplayTurn {
            turn: row.turn,
            input: row.input,
            output: row.output,
            triggered_event_ids: row.triggered_event_ids,
            state_diff: row.state_diff,
        })
        .collect::<Vec<_>>();

    Ok(ReplayResult {
        save_id,
        until_turn: stop,
        total_turns,
        turns,
        consistency,
    })
}

#[tauri::command]
pub fn fork_save(
    save_id: String,
    from_turn: u32,
    new_name: String,
    app: AppHandle,
) -> ApiResult<SaveMeta> {
    if new_name.trim().is_empty() {
        return Err(AppError::validation("newName 不能为空"));
    }

    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let source_meta = load_meta(&paths, &save_id).map_err(map_storage_err)?;
    let mut source_snapshot = load_snapshot(&paths, &save_id).map_err(map_storage_err)?;
    let all_logs = load_all_logs(&paths, &save_id).map_err(map_storage_err)?;

    let prefix_logs: Vec<_> = all_logs
        .iter()
        .filter(|row| row.turn <= from_turn)
        .cloned()
        .collect();
    let effective_turn = prefix_logs.last().map(|row| row.turn).unwrap_or(0);
    source_snapshot.turn = effective_turn;

    let new_save_id = now_id("save");
    fs::create_dir_all(paths.save_dir(&new_save_id))
        .map_err(|e| AppError::storage(format!("failed to create fork save dir: {e}")))?;

    let now = now_iso();
    let meta = SaveMeta {
        id: new_save_id.clone(),
        name: new_name,
        created_at: now.clone(),
        updated_at: now,
        world_card_id: source_meta.world_card_id,
        current_turn: effective_turn,
        player_role: source_meta.player_role,
        model_profile_id: source_meta.model_profile_id,
        provider: source_meta.provider,
        model: source_meta.model,
        parent_save_id: Some(save_id),
        forked_from_turn: Some(from_turn),
    };

    source_snapshot.save_id = new_save_id.clone();
    write_meta(&paths, &meta).map_err(map_storage_err)?;
    write_snapshot(&paths, &source_snapshot).map_err(map_storage_err)?;

    let events_path = paths.save_dir(&new_save_id).join("events.ndjson");
    for row in prefix_logs {
        crate::storage::append_ndjson(&events_path, &row).map_err(map_storage_err)?;
    }

    Ok(meta)
}

#[tauri::command]
pub async fn test_model_provider(
    config: crate::domain::AiModelProfile,
) -> Result<ConnectivityResult, AppError> {
    if config.provider_type.trim().is_empty()
        || config.provider.trim().is_empty()
        || config.model.trim().is_empty()
        || config.base_url.trim().is_empty()
    {
        return Err(AppError::validation(
            "providerType/provider/baseUrl/model 均不能为空",
        ));
    }
    let _ = config
        .api_key
        .clone()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| AppError::validation("apiKey 不能为空"))?;

    let message = test_provider_connectivity(&profile_to_runtime_config(&config))
        .await
        .map_err(AppError::provider)?;

    Ok(ConnectivityResult { ok: true, message })
}

#[tauri::command]
pub async fn generate_world_card_with_ai(
    input: AiGenerateWorldCardInput,
    app: AppHandle,
) -> Result<WorldCard, AppError> {
    if input.prompt.trim().is_empty() {
        return Err(AppError::validation("prompt 不能为空"));
    }

    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);

    let target_model_id = input
        .model_profile_id
        .as_ref()
        .and_then(|id| if id.trim().is_empty() { None } else { Some(id.clone()) })
        .or_else(|| data.ai_settings.default_model_id.clone())
        .or_else(|| data.ai_settings.models.first().map(|item| item.id.clone()))
        .ok_or_else(|| AppError::validation("未找到可用 AI 模型，请先到 AI 设置新增模型"))?;

    let profile = data
        .ai_settings
        .models
        .iter()
        .find(|item| item.id == target_model_id)
        .ok_or_else(|| AppError::validation("指定 AI 模型不存在，请重新选择"))?;
    eprintln!(
        "[world-card-ai] using model profile id={} providerType={} provider={} model={} baseUrl={}",
        profile.id, profile.provider_type, profile.provider, profile.model, profile.base_url
    );
    let api_key_missing = profile
        .api_key
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    if api_key_missing {
        return Err(AppError::validation("当前 AI 模型缺少 API Key"));
    }

    let prompt = build_world_card_prompt(&input.prompt);
    eprintln!(
        "[world-card-ai] prompt length={} chars",
        input.prompt.chars().count()
    );

    let raw = generate_world_card_json(&profile_to_world_card_runtime_config(profile), &prompt)
        .await
        .map_err(AppError::provider)?;
    eprintln!(
        "[world-card-ai] model raw response length={} chars",
        raw.chars().count()
    );
    parse_generated_world_card(&raw)
}

#[tauri::command]
pub async fn generate_world_card_with_ai_stream(
    input: AiGenerateWorldCardInput,
    stream_id: String,
    window: Window,
    app: AppHandle,
) -> Result<WorldCard, AppError> {
    if input.prompt.trim().is_empty() {
        return Err(AppError::validation("prompt 不能为空"));
    }

    let begin = WorldCardStreamEventPayload {
        stream_id: stream_id.clone(),
        phase: "start".to_string(),
        chunk: None,
    };
    let _ = window.emit("world_card_stream_chunk", begin);

    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut data = load_global_data(&paths).unwrap_or_else(|_| default_global_game_data());
    normalize_ai_settings(&mut data.ai_settings);
    let target_model_id = input
        .model_profile_id
        .as_ref()
        .and_then(|id| if id.trim().is_empty() { None } else { Some(id.clone()) })
        .or_else(|| data.ai_settings.default_model_id.clone())
        .or_else(|| data.ai_settings.models.first().map(|item| item.id.clone()))
        .ok_or_else(|| AppError::validation("未找到可用 AI 模型，请先到 AI 设置新增模型"))?;
    let profile = data
        .ai_settings
        .models
        .iter()
        .find(|item| item.id == target_model_id)
        .ok_or_else(|| AppError::validation("指定 AI 模型不存在，请重新选择"))?;
    eprintln!(
        "[world-card-ai-stream] using model profile id={} providerType={} provider={} model={} baseUrl={}",
        profile.id, profile.provider_type, profile.provider, profile.model, profile.base_url
    );
    let api_key_missing = profile
        .api_key
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    if api_key_missing {
        return Err(AppError::validation("当前 AI 模型缺少 API Key"));
    }

    let prompt = build_world_card_prompt(&input.prompt);
    eprintln!(
        "[world-card-ai-stream] prompt length={} chars",
        input.prompt.chars().count()
    );
    let mut emitter = |chunk: &str| -> Result<(), String> {
        let payload = WorldCardStreamEventPayload {
            stream_id: stream_id.clone(),
            phase: "chunk".to_string(),
            chunk: Some(chunk.to_string()),
        };
        window
            .emit("world_card_stream_chunk", payload)
            .map_err(|e| format!("emit world-card stream chunk failed: {e}"))
    };
    let raw = stream_world_card_json(
        &profile_to_world_card_runtime_config(profile),
        &prompt,
        &mut emitter,
    )
        .await
        .map_err(AppError::provider)?;
    eprintln!(
        "[world-card-ai-stream] model raw response length={} chars",
        raw.chars().count()
    );
    let end = WorldCardStreamEventPayload {
        stream_id,
        phase: "end".to_string(),
        chunk: None,
    };
    let _ = window.emit("world_card_stream_chunk", end);
    parse_generated_world_card(&raw)
}

#[cfg(test)]
mod tests {
    use super::build_replay_consistency;
    use crate::domain::{DialogueOption, EventLogEntry, TurnInput, TurnResult};
    use serde_json::json;

    fn mk_log(turn: u32) -> EventLogEntry {
        EventLogEntry {
            turn,
            timestamp: "2026-02-21T00:00:00Z".to_string(),
            input: TurnInput {
                save_id: "save_1".to_string(),
                option_id: Some("opt_plot_1".to_string()),
                custom_text: None,
                draft: false,
            },
            output: TurnResult {
                narration: "n".to_string(),
                options: vec![DialogueOption {
                    id: "opt_plot_1".to_string(),
                    kind: "plot".to_string(),
                    text: "t".to_string(),
                }],
                state_changes_preview: vec![],
                event_hints: vec![],
                triggered_event_ids: vec![],
                state_diff: json!({}),
                story_state: None,
                task_state: None,
                relationship_deltas: vec![],
                ai_meta: None,
            },
            triggered_event_ids: vec![],
            state_diff: json!({}),
        }
    }

    #[test]
    fn replay_consistency_detects_mismatch() {
        let logs = vec![mk_log(1), mk_log(2)];
        let out = build_replay_consistency(&logs, 3);
        assert!(!out.matches_snapshot);
        assert!(!out.warnings.is_empty());
    }

    #[test]
    fn replay_consistency_detects_non_monotonic() {
        let logs = vec![mk_log(1), mk_log(1)];
        let out = build_replay_consistency(&logs, 1);
        assert!(!out.is_monotonic);
    }
}
