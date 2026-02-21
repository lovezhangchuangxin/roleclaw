export type ProviderType = "openai_compatible";

export interface ModelProviderConfig {
  providerType: ProviderType;
  provider: string;
  baseUrl: string;
  model: string;
  apiKey?: string;
  temperature: number;
  maxTokens?: number;
  timeoutMs: number;
}

export interface AiModelProfile {
  id: string;
  providerType: ProviderType;
  provider: string;
  baseUrl: string;
  model: string;
  apiKey?: string;
  temperature: number;
  maxTokens?: number;
  timeoutMs: number;
  updatedAt: string;
}

export interface AiSettings {
  models: AiModelProfile[];
  defaultModelId?: string | null;
}

export interface CharacterArchetype {
  id: string;
  name: string;
  traits: string[];
  motivation: string;
  secret?: string;
}

export interface LocationNode {
  id: string;
  name: string;
  x: number;
  y: number;
  tags: string[];
  npcIds: string[];
  eventIds: string[];
}

export interface PathEdge {
  id: string;
  from: string;
  to: string;
  locked: boolean;
  conditions: string[];
}

export interface WorldBook {
  title: string;
  overview: string;
  background: string;
  coreConflicts: string[];
  playStyle: string;
}

export interface MapCanvas {
  width: number;
  height: number;
}

export interface MapNode {
  id: string;
  name: string;
  description: string;
  tags: string[];
  x: number;
  y: number;
}

export interface MapEdge {
  id: string;
  a: string;
  b: string;
  locked: boolean;
  unlockConditions: string[];
}

export interface WorldMap {
  nodes: MapNode[];
  edges: MapEdge[];
  startNodeId: string;
  canvas: MapCanvas;
}

export interface NpcProfile {
  id: string;
  name: string;
  personality: string[];
  identity: string;
}

export interface CardEvent {
  id: string;
  name: string;
  prompt: string;
}

export interface ChapterGoal {
  id: string;
  title: string;
  prompt: string;
}

export interface WorldCard {
  id: string;
  name: string;
  schemaVersion: string;
  contentVersion: number;
  worldbook: WorldBook;
  map: WorldMap;
  npcs: NpcProfile[];
  events: CardEvent[];
  chapterGoals: ChapterGoal[];
}

export interface WorldInit {
  worldSummary: string;
  mainNpcs: CharacterArchetype[];
  locations: LocationNode[];
  paths: PathEdge[];
  questHooks: string[];
}

export interface CreateSaveConfig {
  saveName: string;
  playerRole: string;
  worldCardId: string;
  modelProfileId: string;
  worldInit?: WorldInit;
}

export interface SaveMeta {
  id: string;
  name: string;
  createdAt: string;
  updatedAt: string;
  worldCardId: string;
  currentTurn: number;
  playerRole: string;
  modelProfileId: string;
  provider: string;
  model: string;
  parentSaveId?: string;
  forkedFromTurn?: number;
}

export interface QuestState {
  id: string;
  title: string;
  stage: number;
  completed: boolean;
}

export interface TriggerCondition {
  type: string;
  params: Record<string, unknown>;
}

export interface GuardCondition {
  expr: string;
}

export interface EventAction {
  type:
    | "set_variable"
    | "inc_variable"
    | "unlock_location"
    | "lock_path"
    | "update_relationship"
    | "inject_quest"
    | "advance_quest_stage"
    | "append_log";
  params: Record<string, unknown>;
}

export interface GameEvent {
  id: string;
  name: string;
  trigger: TriggerCondition;
  guards: GuardCondition[];
  actions: EventAction[];
  cooldownTurns?: number;
  nextEventIds?: string[];
}

export interface SaveSnapshot {
  saveId: string;
  turn: number;
  currentLocationId: string;
  playerRole: string;
  relationships: Record<string, unknown>;
  worldSummary: string;
  locations: LocationNode[];
  paths: PathEdge[];
  modelProfileId: string;
  modelLabel: string;
  activeEventIds: string[];
  worldVariables: Record<string, unknown>;
  quests: QuestState[];
  events: GameEvent[];
  shortTermMemory: string[];
  midTermSummary: string;
  factLocks: string[];
}

export interface DialogueOption {
  id: string;
  kind: "plot" | "emotion" | "risk" | "custom" | string;
  text: string;
}

export interface TurnInput {
  saveId: string;
  optionId?: string;
  customText?: string;
  draft?: boolean;
}

export interface TurnResult {
  narration: string;
  options: DialogueOption[];
  stateChangesPreview: string[];
  eventHints: string[];
  triggeredEventIds: string[];
  stateDiff: Record<string, unknown>;
}

export interface EventLogEntry {
  turn: number;
  timestamp: string;
  input: TurnInput;
  output: TurnResult;
  triggeredEventIds: string[];
  stateDiff: Record<string, unknown>;
}

export interface EventLogPage {
  items: EventLogEntry[];
  nextCursor?: number;
  total: number;
}

export interface ReplayTurn {
  turn: number;
  input: TurnInput;
  output: TurnResult;
  triggeredEventIds: string[];
  stateDiff: Record<string, unknown>;
}

export interface ReplayConsistency {
  snapshotTurn: number;
  logLastTurn: number;
  isMonotonic: boolean;
  matchesSnapshot: boolean;
  warnings: string[];
}

export interface ReplayResult {
  saveId: string;
  untilTurn: number;
  totalTurns: number;
  turns: ReplayTurn[];
  consistency: ReplayConsistency;
}

export interface SaveBundle {
  meta: SaveMeta;
  snapshot: SaveSnapshot;
  recentLogs: EventLogEntry[];
}

export interface MoveResult {
  moved: boolean;
  currentLocationId: string;
  message: string;
  triggeredEventIds: string[];
}

export interface ConnectivityResult {
  ok: boolean;
  message: string;
}

export interface GameSettings {
  theme: "default" | "fantasy" | "terminal" | "archive";
  messageSpeed: "slow" | "normal" | "fast";
  fontScale: number;
  uiZoom: number;
  logLevel: "error" | "warn" | "info" | "debug";
}

export interface GlobalGameData {
  gameSettings: GameSettings;
  aiSettings: AiSettings;
}
