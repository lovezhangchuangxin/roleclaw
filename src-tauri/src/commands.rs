use crate::domain::{
    ConnectivityResult, CreateSaveConfig, EventResult, GameSettings, GenerateWorldInput, GlobalGameData,
    MoveResult, SaveBundle, SaveMeta, SaveSnapshot, TriggerEventPayload, TurnInput, TurnResult, WorldCard,
    WorldInit,
};
use crate::error::AppError;
use crate::game::{default_world_cards, generate_world_from_card, run_turn_stream_with_provider, run_turn_with_provider};
use crate::llm::test_provider_connectivity;
use crate::storage::{
    collect_recent_logs, load_global_data, load_meta, load_snapshot, now_id, now_iso, read_json, write_global_data,
    write_json, write_meta, write_snapshot, AppPaths,
};
use std::fs;
use std::path::Path;
use tauri::{AppHandle, Emitter, Window};
use serde::Serialize;

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

fn default_global_game_data() -> GlobalGameData {
    GlobalGameData {
        game_settings: GameSettings {
            theme: "default".to_string(),
            message_speed: "normal".to_string(),
        },
        ai_settings: crate::domain::ModelProviderConfig {
            provider: "openai_compatible".to_string(),
            provider_name: "OpenAI Compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4.1".to_string(),
            api_key: None,
            temperature: 0.7,
            max_tokens: 100000,
            timeout_ms: 25000,
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
        Ok(data) => Ok(data),
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
    write_global_data(&paths, &data).map_err(map_storage_err)?;
    Ok(data)
}

#[tauri::command]
pub fn import_world_card(file: String, app: AppHandle) -> ApiResult<WorldCard> {
    let card: WorldCard = if file.trim_start().starts_with('{') {
        serde_json::from_str(&file).map_err(|e| AppError::validation(format!("invalid world card JSON: {e}")))?
    } else {
        read_json(Path::new(&file)).map_err(map_storage_err)?
    };
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let out_path = paths.world_cards_dir.join(format!("{}.json", card.id.clone()));
    write_json(&out_path, &card).map_err(map_storage_err)?;
    Ok(card)
}

#[tauri::command]
pub fn export_world_card(card_id: String, file: String, app: AppHandle) -> ApiResult<()> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let source = paths.world_cards_dir.join(format!("{card_id}.json"));
    if !source.exists() {
        return Err(AppError::not_found(format!("world card not found: {card_id}")));
    }
    let card: WorldCard = read_json(&source).map_err(map_storage_err)?;
    write_json(Path::new(&file), &card).map_err(map_storage_err)
}

#[tauri::command]
pub fn generate_world(input: GenerateWorldInput, app: AppHandle) -> ApiResult<WorldInit> {
    let paths = AppPaths::from_app(&app).map_err(map_storage_err)?;
    let card: WorldCard = read_json(&paths.world_cards_dir.join(format!("{}.json", input.world_card_id)))
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
    if config.model_config.provider != "openai_compatible" {
        return Err(AppError::validation("provider 仅支持 openai_compatible"));
    }
    let save_id = now_id("save");
    fs::create_dir_all(paths.save_dir(&save_id)).map_err(|e| AppError::storage(format!("failed to create save dir: {e}")))?;

    let world_init = if let Some(w) = config.world_init.clone() {
        w
    } else {
        let card: WorldCard =
            read_json(&paths.world_cards_dir.join(format!("{}.json", config.world_card_id))).map_err(map_storage_err)?;
        generate_world_from_card(&card, &config.player_role)
    };
    let starting_location = world_init
        .locations
        .first()
        .map(|v| v.id.clone())
        .unwrap_or_else(|| "loc_origin".to_string());
    let timestamp = now_iso();
    let meta = SaveMeta {
        id: save_id.clone(),
        name: config.save_name,
        created_at: timestamp.clone(),
        updated_at: timestamp,
        world_card_id: config.world_card_id,
        current_turn: 0,
        player_role: config.player_role.clone(),
        provider: config.model_config.provider.clone(),
        model: config.model_config.model.clone(),
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
        model_config: crate::domain::ModelProviderConfig {
            api_key: None,
            ..config.model_config
        },
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
pub fn move_to_location(save_id: String, location_id: String, app: AppHandle) -> ApiResult<MoveResult> {
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
    run_turn_with_provider(&paths, turn_input).await.map_err(AppError::provider)
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
    config: crate::domain::ModelProviderConfig,
) -> Result<ConnectivityResult, AppError> {
    if config.provider.trim().is_empty()
        || config.model.trim().is_empty()
        || config.base_url.trim().is_empty()
        || config.provider_name.trim().is_empty()
    {
        return Err(AppError::validation(
            "provider/providerName/baseUrl/model 均不能为空",
        ));
    }
    if config.provider != "openai_compatible" {
        return Err(AppError::validation("当前仅支持 openai_compatible 协议"));
    }
    let _ = config
        .api_key
        .clone()
        .filter(|key| !key.trim().is_empty())
        .ok_or_else(|| AppError::validation("apiKey 不能为空"))?;

    let message = test_provider_connectivity(&config)
        .await
        .map_err(AppError::provider)?;

    Ok(ConnectivityResult {
        ok: true,
        message,
    })
}
