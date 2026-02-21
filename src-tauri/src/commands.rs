use crate::domain::{
    AiModelProfile, AiSettings, ConnectivityResult, CreateSaveConfig, EventResult, GameSettings,
    GenerateWorldInput, GlobalGameData, ModelProviderConfig, MoveResult, SaveBundle, SaveMeta,
    SaveSnapshot, TriggerEventPayload, TurnInput, TurnResult, WorldCard, WorldInit,
};
use crate::error::AppError;
use crate::game::{
    default_world_cards, generate_world_from_card, run_turn_stream_with_provider,
    run_turn_with_provider,
};
use crate::llm::test_provider_connectivity;
use crate::storage::{
    collect_recent_logs, load_global_data, load_meta, load_snapshot, now_id, now_iso, read_json,
    write_global_data, write_json, write_meta, write_snapshot, AppPaths,
};
use serde::Serialize;
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter, Window};

type ApiResult<T> = Result<T, AppError>;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct TurnStreamEventPayload {
    stream_id: String,
    phase: String,
    chunk: Option<String>,
}

fn map_storage_err(err: String) -> AppError {
    AppError::storage(err)
}

fn normalize_ai_settings(ai_settings: &mut AiSettings) {
    // Ensure model ids are usable for edit/delete operations.
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

fn profile_to_runtime_config(profile: &AiModelProfile) -> ModelProviderConfig {
    ModelProviderConfig {
        provider_type: profile.provider_type.clone(),
        provider: profile.provider.clone(),
        base_url: profile.base_url.clone(),
        model: profile.model.clone(),
        api_key: profile.api_key.clone(),
        temperature: profile.temperature,
        max_tokens: profile.max_tokens,
        timeout_ms: profile.timeout_ms,
    }
}

fn default_global_game_data() -> GlobalGameData {
    GlobalGameData {
        game_settings: GameSettings {
            theme: "default".to_string(),
            message_speed: "normal".to_string(),
        },
        ai_settings: AiSettings {
            default_model_id: None,
            models: Vec::new(),
        },
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

    // Prefer explicit id update, fallback to natural-key match to avoid accidental duplicates.
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
pub fn generate_world(input: GenerateWorldInput, app: AppHandle) -> ApiResult<WorldInit> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let card: WorldCard = read_json(
        &paths
            .world_cards_dir
            .join(format!("{}.json", input.world_card_id)),
    )
    .map_err(map_storage_err)?;
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
    let save_id = now_id("save");
    fs::create_dir_all(paths.save_dir(&save_id))
        .map_err(|e| AppError::storage(format!("failed to create save dir: {e}")))?;

    let world_init = if let Some(w) = config.world_init.clone() {
        w
    } else {
        let card: WorldCard = read_json(
            &paths
                .world_cards_dir
                .join(format!("{}.json", config.world_card_id)),
        )
        .map_err(map_storage_err)?;
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
        world_card_id: config.world_card_id,
        current_turn: 0,
        player_role: config.player_role.clone(),
        model_profile_id: profile.id.clone(),
        provider: profile.provider.clone(),
        model: profile.model.clone(),
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
    write_snapshot(&paths, &snapshot).map_err(map_storage_err)?;
    Ok(MoveResult {
        moved: true,
        current_location_id: location_id,
        message: "已移动到新地点".to_string(),
        triggered_event_ids: vec!["evt_on_enter_location".to_string()],
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
        chunk: None,
    };
    let _ = window.emit("turn_stream_chunk", begin);
    let mut emitter = |chunk: &str| -> Result<(), String> {
        let payload = TurnStreamEventPayload {
            stream_id: stream_id.clone(),
            phase: "chunk".to_string(),
            chunk: Some(chunk.to_string()),
        };
        window
            .emit("turn_stream_chunk", payload)
            .map_err(|e| format!("emit stream chunk failed: {e}"))
    };

    let result = run_turn_stream_with_provider(&paths, turn_input, &mut emitter)
        .await
        .map_err(AppError::provider)?;

    let end = TurnStreamEventPayload {
        stream_id,
        phase: "end".to_string(),
        chunk: None,
    };
    let _ = window.emit("turn_stream_chunk", end);
    Ok(result)
}

#[tauri::command]
pub fn trigger_event(payload: TriggerEventPayload, app: AppHandle) -> ApiResult<EventResult> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let mut snapshot = load_snapshot(&paths, &payload.save_id).map_err(map_storage_err)?;
    if !snapshot.active_event_ids.contains(&payload.event_id) {
        snapshot.active_event_ids.push(payload.event_id.clone());
    }
    write_snapshot(&paths, &snapshot).map_err(map_storage_err)?;
    Ok(EventResult {
        triggered: true,
        event_id: payload.event_id,
        message: "事件已记录（MVP stub）".to_string(),
    })
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
    if config.provider_type != "openai_compatible" {
        return Err(AppError::validation("当前仅支持 openai_compatible 协议"));
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
