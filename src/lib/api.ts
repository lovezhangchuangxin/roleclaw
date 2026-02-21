import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AiModelProfile,
  AiSettings,
  ConnectivityResult,
  CreateSaveConfig,
  GlobalGameData,
  MoveResult,
  SaveBundle,
  SaveMeta,
  TurnInput,
  TurnResult,
  WorldCard,
  WorldInit,
} from "@/types";

export async function listSaves(): Promise<SaveMeta[]> {
  return invoke("list_saves");
}

export async function loadSave(saveId: string): Promise<SaveBundle> {
  return invoke("load_save", { saveId });
}

export async function deleteSave(saveId: string): Promise<void> {
  return invoke("delete_save", { saveId });
}

export async function listWorldCards(): Promise<WorldCard[]> {
  return invoke("list_world_cards");
}

export async function generateWorld(
  worldCardId: string,
  playerRole: string,
): Promise<WorldInit> {
  return invoke("generate_world", {
    input: { worldCardId, playerRole },
  });
}

export async function createSave(config: CreateSaveConfig): Promise<SaveMeta> {
  return invoke("create_save", { config });
}

export async function runTurn(turnInput: TurnInput): Promise<TurnResult> {
  return invoke("run_turn", { turnInput });
}

type TurnStreamPhase = "start" | "chunk" | "end";

interface TurnStreamEventPayload {
  streamId: string;
  phase: TurnStreamPhase;
  chunk?: string;
}

function newStreamId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `stream_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

export async function runTurnStream(
  turnInput: TurnInput,
  onChunk: (chunk: string) => void,
): Promise<TurnResult> {
  const streamId = newStreamId();
  const unlisten = await listen<TurnStreamEventPayload>(
    "turn_stream_chunk",
    (event) => {
      const payload = event.payload;
      if (!payload || payload.streamId !== streamId) {
        return;
      }
      if (payload.phase === "chunk" && payload.chunk) {
        onChunk(payload.chunk);
      }
    },
  );

  try {
    return await invoke("run_turn_stream", { turnInput, streamId });
  } finally {
    unlisten();
  }
}

export async function moveToLocation(
  saveId: string,
  locationId: string,
): Promise<MoveResult> {
  return invoke("move_to_location", { saveId, locationId });
}

export async function listAiModels(): Promise<AiSettings> {
  return invoke("list_ai_models");
}

export async function upsertAiModel(
  profile: AiModelProfile,
): Promise<AiModelProfile> {
  return invoke("upsert_ai_model", { profile });
}

export async function deleteAiModel(modelId: string): Promise<void> {
  return invoke("delete_ai_model", { modelId });
}

export async function setDefaultAiModel(modelId: string): Promise<AiSettings> {
  return invoke("set_default_ai_model", { modelId });
}

export async function testModelProvider(
  config: AiModelProfile,
): Promise<ConnectivityResult> {
  return invoke("test_model_provider", { config });
}

export async function importWorldCard(file: string): Promise<WorldCard> {
  return invoke("import_world_card", { file });
}

export async function exportWorldCard(
  cardId: string,
  file: string,
): Promise<void> {
  return invoke("export_world_card", { cardId, file });
}

export async function getGlobalGameData(): Promise<GlobalGameData> {
  return invoke("get_global_game_data");
}

export async function updateGlobalGameData(
  data: GlobalGameData,
): Promise<GlobalGameData> {
  return invoke("update_global_game_data", { data });
}
