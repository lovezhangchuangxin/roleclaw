use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
    #[serde(default = "default_model_max_tokens_compat_option")]
    pub max_tokens: Option<u32>,
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

fn default_model_max_tokens_compat_option() -> Option<u32> {
    Some(default_model_max_tokens_compat())
}

fn default_model_timeout_ms() -> u32 {
    25000
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
    pub worldbook: WorldBook,
    pub map: WorldMap,
    pub npcs: Vec<NpcProfile>,
    pub events: Vec<CardPromptEvent>,
    pub chapter_goals: Vec<ChapterGoal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldBook {
    pub title: String,
    pub overview: String,
    pub background: String,
    #[serde(default)]
    pub core_conflicts: Vec<String>,
    pub play_style: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorldMap {
    pub nodes: Vec<MapNode>,
    pub edges: Vec<MapEdge>,
    pub start_node_id: String,
    pub canvas: MapCanvas,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapCanvas {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapNode {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapEdge {
    pub id: String,
    pub a: String,
    pub b: String,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub unlock_conditions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NpcProfile {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub personality: Vec<String>,
    pub identity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CardPromptEvent {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChapterGoal {
    pub id: String,
    pub title: String,
    pub prompt: String,
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
    #[serde(default)]
    pub parent_save_id: Option<String>,
    #[serde(default)]
    pub forked_from_turn: Option<u32>,
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
    #[serde(default)]
    pub world_variables: BTreeMap<String, Value>,
    #[serde(default)]
    pub quests: Vec<QuestState>,
    #[serde(default)]
    pub events: Vec<GameEvent>,
    #[serde(default)]
    pub short_term_memory: Vec<String>,
    #[serde(default)]
    pub mid_term_summary: String,
    #[serde(default)]
    pub fact_locks: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuestState {
    pub id: String,
    pub title: String,
    pub stage: u32,
    #[serde(default)]
    pub completed: bool,
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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StoryState {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub tension: String,
    #[serde(default)]
    pub scene_tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskStateItem {
    pub id: String,
    pub title: String,
    pub stage: u32,
    pub status: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TaskState {
    #[serde(default)]
    pub items: Vec<TaskStateItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipDelta {
    pub source: String,
    pub target: String,
    pub delta: f64,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiMeta {
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub parser: String,
    #[serde(default)]
    pub raw_chars: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TurnStateProposal {
    #[serde(default)]
    pub narration: String,
    #[serde(default)]
    pub options: Vec<DialogueOption>,
    #[serde(default)]
    pub state_changes_preview: Vec<String>,
    #[serde(default)]
    pub event_hints: Vec<String>,
    #[serde(default)]
    pub story_state: Option<StoryState>,
    #[serde(default)]
    pub task_state: Option<TaskState>,
    #[serde(default)]
    pub relationship_deltas: Vec<RelationshipDelta>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TurnInput {
    pub save_id: String,
    pub option_id: Option<String>,
    pub custom_text: Option<String>,
    #[serde(default)]
    pub draft: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TurnResult {
    pub narration: String,
    pub options: Vec<DialogueOption>,
    pub state_changes_preview: Vec<String>,
    pub event_hints: Vec<String>,
    #[serde(default)]
    pub triggered_event_ids: Vec<String>,
    #[serde(default)]
    pub state_diff: Value,
    #[serde(default)]
    pub story_state: Option<StoryState>,
    #[serde(default)]
    pub task_state: Option<TaskState>,
    #[serde(default)]
    pub relationship_deltas: Vec<RelationshipDelta>,
    #[serde(default)]
    pub ai_meta: Option<AiMeta>,
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
pub struct EventLogPage {
    pub items: Vec<EventLogEntry>,
    pub next_cursor: Option<u32>,
    pub total: usize,
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
    #[serde(default)]
    pub state_changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayTurn {
    pub turn: u32,
    pub input: TurnInput,
    pub output: TurnResult,
    pub triggered_event_ids: Vec<String>,
    pub state_diff: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayResult {
    pub save_id: String,
    pub until_turn: u32,
    pub total_turns: u32,
    pub turns: Vec<ReplayTurn>,
    pub consistency: ReplayConsistency,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReplayConsistency {
    pub snapshot_turn: u32,
    pub log_last_turn: u32,
    pub is_monotonic: bool,
    pub matches_snapshot: bool,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConnectivityResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiGenerateWorldCardInput {
    pub prompt: String,
    pub model_profile_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameSettings {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_message_speed")]
    pub message_speed: String,
    #[serde(default = "default_font_scale")]
    pub font_scale: f32,
    #[serde(default = "default_ui_zoom")]
    pub ui_zoom: f32,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_theme() -> String {
    "default".to_string()
}

fn default_message_speed() -> String {
    "normal".to_string()
}

fn default_font_scale() -> f32 {
    1.0
}

fn default_ui_zoom() -> f32 {
    1.0
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlobalGameData {
    pub game_settings: GameSettings,
    pub ai_settings: AiSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerCondition {
    pub r#type: String,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GuardCondition {
    pub expr: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventAction {
    pub r#type: String,
    #[serde(default)]
    pub params: BTreeMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameEvent {
    pub id: String,
    pub name: String,
    pub trigger: TriggerCondition,
    #[serde(default)]
    pub guards: Vec<GuardCondition>,
    #[serde(default)]
    pub actions: Vec<EventAction>,
    #[serde(default)]
    pub cooldown_turns: Option<u32>,
    #[serde(default)]
    pub next_event_ids: Vec<String>,
}
