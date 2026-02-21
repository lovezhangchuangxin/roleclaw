import { invoke } from "@tauri-apps/api/core";
import type {
  ConnectivityResult,
  CreateSaveConfig,
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

export async function generateWorld(worldCardId: string, playerRole: string): Promise<WorldInit> {
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

export async function moveToLocation(saveId: string, locationId: string): Promise<MoveResult> {
  return invoke("move_to_location", { saveId, locationId });
}

export async function testModelProvider(config: CreateSaveConfig["modelConfig"]): Promise<ConnectivityResult> {
  return invoke("test_model_provider", { config });
}

export async function importWorldCard(file: string): Promise<WorldCard> {
  return invoke("import_world_card", { file });
}

export async function exportWorldCard(cardId: string, file: string): Promise<void> {
  return invoke("export_world_card", { cardId, file });
}
