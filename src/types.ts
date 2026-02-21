export type Provider = "openai" | "claude";

export interface ModelProviderConfig {
  provider: Provider;
  model: string;
  temperature: number;
  maxTokens: number;
  timeoutMs: number;
}

export interface WorldRule {
  id: string;
  title: string;
  content: string;
  priority: number;
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

export interface WorldCard {
  id: string;
  name: string;
  schemaVersion: string;
  contentVersion: number;
  genre: string;
  tone: string;
  rules: WorldRule[];
  locationPool: LocationNode[];
  archetypePool: CharacterArchetype[];
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
  modelConfig: ModelProviderConfig;
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
  provider: string;
  model: string;
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
  modelConfig: ModelProviderConfig;
  activeEventIds: string[];
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
}

export interface TurnResult {
  narration: string;
  options: DialogueOption[];
  stateChangesPreview: string[];
  eventHints: string[];
}

export interface EventLogEntry {
  turn: number;
  timestamp: string;
  input: TurnInput;
  output: TurnResult;
  triggeredEventIds: string[];
  stateDiff: Record<string, unknown>;
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
