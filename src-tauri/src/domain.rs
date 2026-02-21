use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelProviderConfig {
    pub provider_type: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    #[serde(default = "default_model_temperature")]
    pub temperature: f32,
    #[serde(default = "default_model_max_tokens_compat")]
    pub max_tokens: u32,
    #[serde(default = "default_model_timeout_ms")]
    pub timeout_ms: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiModelProfile {
    pub id: String,
    pub provider_type: String,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    #[serde(default = "default_model_temperature")]
    pub temperature: f32,
    #[serde(default = "default_model_max_tokens_compat")]
    pub max_tokens: u32,
    #[serde(default = "default_model_timeout_ms")]
    pub timeout_ms: u32,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSettings {
    pub models: Vec<AiModelProfile>,
    pub default_model_id: Option<String>,
}

fn default_model_temperature() -> f32 {
    0.7
}

fn default_model_max_tokens_compat() -> u32 {
    // Compatibility field. Runtime max tokens are controlled by scene strategy.
    100000
}

fn default_model_timeout_ms() -> u32 {
    25000
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldRule {
    pub id: String,
    pub title: String,
    pub content: String,
    pub priority: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterArchetype {
    pub id: String,
    pub name: String,
    pub traits: Vec<String>,
    pub motivation: String,
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LocationNode {
    pub id: String,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub tags: Vec<String>,
    pub npc_ids: Vec<String>,
    pub event_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PathEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    pub locked: bool,
    pub conditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldCard {
    pub id: String,
    pub name: String,
    pub schema_version: String,
    pub content_version: u32,
    pub genre: String,
    pub tone: String,
    pub rules: Vec<WorldRule>,
    pub location_pool: Vec<LocationNode>,
    pub archetype_pool: Vec<CharacterArchetype>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldInit {
    pub world_summary: String,
    pub main_npcs: Vec<CharacterArchetype>,
    pub locations: Vec<LocationNode>,
    pub paths: Vec<PathEdge>,
    pub quest_hooks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveMeta {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub world_card_id: String,
    pub current_turn: u32,
    pub player_role: String,
    #[serde(default)]
    pub model_profile_id: String,
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveSnapshot {
    pub save_id: String,
    pub turn: u32,
    pub current_location_id: String,
    pub player_role: String,
    pub relationships: serde_json::Map<String, Value>,
    pub world_summary: String,
    pub locations: Vec<LocationNode>,
    pub paths: Vec<PathEdge>,
    #[serde(default)]
    pub model_profile_id: String,
    #[serde(default)]
    pub model_label: String,
    pub active_event_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SaveBundle {
    pub meta: SaveMeta,
    pub snapshot: SaveSnapshot,
    pub recent_logs: Vec<EventLogEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateSaveConfig {
    pub save_name: String,
    pub player_role: String,
    pub world_card_id: String,
    pub model_profile_id: String,
    pub world_init: Option<WorldInit>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenerateWorldInput {
    pub world_card_id: String,
    pub player_role: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DialogueOption {
    pub id: String,
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TurnInput {
    pub save_id: String,
    pub option_id: Option<String>,
    pub custom_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TurnResult {
    pub narration: String,
    pub options: Vec<DialogueOption>,
    pub state_changes_preview: Vec<String>,
    pub event_hints: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventLogEntry {
    pub turn: u32,
    pub timestamp: String,
    pub input: TurnInput,
    pub output: TurnResult,
    pub triggered_event_ids: Vec<String>,
    pub state_diff: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MoveResult {
    pub moved: bool,
    pub current_location_id: String,
    pub message: String,
    pub triggered_event_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerEventPayload {
    pub save_id: String,
    pub event_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventResult {
    pub triggered: bool,
    pub event_id: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameSettings {
    pub theme: String,
    pub message_speed: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalGameData {
    pub game_settings: GameSettings,
    pub ai_settings: AiSettings,
}
