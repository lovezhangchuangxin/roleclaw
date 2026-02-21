import { computed, ref } from "vue";
import {
  createSave,
  deleteSave,
  exportWorldCard,
  generateWorld,
  importWorldCard,
  listSaves,
  listWorldCards,
  loadSave,
  moveToLocation,
  runTurn,
  testModelProvider,
} from "@/lib/api";
import { normalizeError } from "@/lib/errors";
import type {
  CreateSaveConfig,
  DialogueOption,
  SaveBundle,
  SaveMeta,
  TurnResult,
  WorldCard,
} from "@/types";

export type ViewMode = "menu" | "new" | "game" | "cards" | "saves" | "ai-settings" | "settings";

export function useGameApp() {
  const view = ref<ViewMode>("menu");
  const errorMsg = ref("");
  const modelCheckMsg = ref("");
  const narrationText = ref("欢迎来到 RoleClaw。先创建一个存档开始冒险。");
  const stateChanges = ref<string[]>([]);
  const options = ref<DialogueOption[]>([]);
  const customInput = ref("");
  const cardImportText = ref("");
  const cardExportPath = ref("");

  const saves = ref<SaveMeta[]>([]);
  const worldCards = ref<WorldCard[]>([]);
  const activeSave = ref<SaveBundle | null>(null);

  const newSave = ref<CreateSaveConfig>({
    saveName: "新冒险",
    playerRole: "流浪调查员",
    worldCardId: "",
    modelConfig: {
      provider: "openai",
      model: "gpt-4.1",
      temperature: 0.7,
      maxTokens: 900,
      timeoutMs: 25000,
    },
  });

  const reachableLocations = computed(() => {
    if (!activeSave.value) {
      return [];
    }
    const snapshot = activeSave.value.snapshot;
    const current = snapshot.currentLocationId;
    const connectedIds = new Set<string>();
    for (const edge of snapshot.paths) {
      if (edge.locked) {
        continue;
      }
      if (edge.from === current) {
        connectedIds.add(edge.to);
      }
      if (edge.to === current) {
        connectedIds.add(edge.from);
      }
    }
    return snapshot.locations.filter((loc) => connectedIds.has(loc.id));
  });

  function setError(err: unknown) {
    errorMsg.value = normalizeError(err);
  }

  function goMenu() {
    view.value = "menu";
  }

  function setView(next: ViewMode) {
    view.value = next;
  }

  async function refreshHome() {
    errorMsg.value = "";
    try {
      const [saveList, cardList] = await Promise.all([listSaves(), listWorldCards()]);
      saves.value = saveList;
      worldCards.value = cardList;
      if (!newSave.value.worldCardId && cardList.length > 0) {
        newSave.value.worldCardId = cardList[0].id;
      }
    } catch (err) {
      setError(err);
    }
  }

  async function openSave(saveId: string, resetScene = true) {
    errorMsg.value = "";
    try {
      const bundle = await loadSave(saveId);
      activeSave.value = bundle;
      view.value = "game";
      if (resetScene) {
        narrationText.value = bundle.snapshot.worldSummary;
        stateChanges.value = ["存档已加载"];
        options.value = [
          { id: "opt_plot_1", kind: "plot", text: "追问关键线索" },
          { id: "opt_emotion_1", kind: "emotion", text: "尝试建立信任" },
          { id: "opt_risk_1", kind: "risk", text: "冒险试探未知区域" },
        ];
      }
    } catch (err) {
      setError(err);
    }
  }

  async function removeSave(saveId: string) {
    errorMsg.value = "";
    try {
      await deleteSave(saveId);
      await refreshHome();
    } catch (err) {
      setError(err);
    }
  }

  async function checkModel() {
    errorMsg.value = "";
    modelCheckMsg.value = "";
    try {
      const result = await testModelProvider(newSave.value.modelConfig);
      modelCheckMsg.value = result.message;
    } catch (err) {
      setError(err);
    }
  }

  async function createNewSave() {
    errorMsg.value = "";
    try {
      const worldInit = await generateWorld(newSave.value.worldCardId, newSave.value.playerRole);
      const meta = await createSave({ ...newSave.value, worldInit });
      await refreshHome();
      await openSave(meta.id, true);
    } catch (err) {
      setError(err);
    }
  }

  async function applyTurnResult(result: TurnResult) {
    narrationText.value = result.narration;
    stateChanges.value = result.stateChangesPreview;
    options.value = result.options;
    if (activeSave.value) {
      await openSave(activeSave.value.meta.id, false);
    }
  }

  async function submitOption(optionId: string) {
    if (!activeSave.value) {
      return;
    }
    errorMsg.value = "";
    try {
      const result = await runTurn({ saveId: activeSave.value.meta.id, optionId });
      await applyTurnResult(result);
    } catch (err) {
      setError(err);
    }
  }

  async function submitCustom() {
    if (!activeSave.value || !customInput.value.trim()) {
      return;
    }
    errorMsg.value = "";
    try {
      const result = await runTurn({
        saveId: activeSave.value.meta.id,
        customText: customInput.value.trim(),
      });
      customInput.value = "";
      await applyTurnResult(result);
    } catch (err) {
      setError(err);
    }
  }

  async function move(locationId: string) {
    if (!activeSave.value) {
      return;
    }
    errorMsg.value = "";
    try {
      const result = await moveToLocation(activeSave.value.meta.id, locationId);
      stateChanges.value = [result.message, ...result.triggeredEventIds.map((id) => `触发 ${id}`)];
      await openSave(activeSave.value.meta.id, false);
    } catch (err) {
      setError(err);
    }
  }

  async function importCardFromText() {
    if (!cardImportText.value.trim()) {
      errorMsg.value = "请输入世界卡 JSON";
      return;
    }
    errorMsg.value = "";
    try {
      await importWorldCard(cardImportText.value.trim());
      cardImportText.value = "";
      await refreshHome();
    } catch (err) {
      setError(err);
    }
  }

  async function exportCard(cardId: string) {
    if (!cardExportPath.value.trim()) {
      errorMsg.value = "请输入导出路径";
      return;
    }
    errorMsg.value = "";
    try {
      await exportWorldCard(cardId, cardExportPath.value.trim());
    } catch (err) {
      setError(err);
    }
  }

  return {
    view,
    errorMsg,
    modelCheckMsg,
    narrationText,
    stateChanges,
    options,
    customInput,
    cardImportText,
    cardExportPath,
    saves,
    worldCards,
    activeSave,
    newSave,
    reachableLocations,
    goMenu,
    setView,
    refreshHome,
    openSave,
    removeSave,
    checkModel,
    createNewSave,
    submitOption,
    submitCustom,
    move,
    importCardFromText,
    exportCard,
  };
}
