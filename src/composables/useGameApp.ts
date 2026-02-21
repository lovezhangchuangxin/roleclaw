import { computed, ref } from "vue";
import {
  createSave,
  deleteAiModel,
  deleteSave,
  exportWorldCard,
  getGlobalGameData,
  generateWorld,
  importWorldCard,
  listSaves,
  listWorldCards,
  loadSave,
  moveToLocation,
  runTurnStream,
  setDefaultAiModel,
  testModelProvider,
  updateGlobalGameData,
  upsertAiModel,
} from "@/lib/api";
import { normalizeError } from "@/lib/errors";
import type {
  AiModelProfile,
  CreateSaveConfig,
  DialogueOption,
  GameSettings,
  SaveBundle,
  SaveMeta,
  TurnResult,
  WorldCard,
} from "@/types";

function defaultAiModelDraft(): AiModelProfile {
  return {
    id: "",
    providerType: "openai_compatible",
    provider: "OpenAI",
    baseUrl: "https://api.openai.com/v1",
    model: "gpt-4.1",
    apiKey: "",
    temperature: 0.7,
    timeoutMs: 25000,
    updatedAt: "",
  };
}

export type ViewMode =
  | "menu"
  | "new"
  | "game"
  | "cards"
  | "saves"
  | "ai-settings"
  | "settings";

export function useGameApp() {
  const view = ref<ViewMode>("menu");
  const errorMsg = ref("");
  const modelCheckMsg = ref("");
  const modelCheckOk = ref<boolean | null>(null);
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
    modelProfileId: "",
  });
  const gameSettings = ref<GameSettings>({
    theme: "default",
    messageSpeed: "normal",
  });

  const aiModels = ref<AiModelProfile[]>([]);
  const defaultModelId = ref<string | null>(null);
  const editingModelId = ref<string | null>(null);
  const aiDraft = ref<AiModelProfile>(defaultAiModelDraft());

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

  function applyAiSettings(
    models: AiModelProfile[],
    defaultId?: string | null,
  ) {
    aiModels.value = models;
    defaultModelId.value = defaultId ?? null;

    const nextDefault =
      defaultId && models.some((item) => item.id === defaultId)
        ? defaultId
        : models[0]?.id;
    newSave.value.modelProfileId = nextDefault ?? "";

    if (editingModelId.value) {
      const editing = models.find((item) => item.id === editingModelId.value);
      if (editing) {
        aiDraft.value = { ...editing };
        return;
      }
    }

    if (models.length > 0) {
      aiDraft.value = { ...models[0] };
      editingModelId.value = models[0].id;
    } else {
      resetAiDraft();
    }
  }

  async function refreshHome() {
    errorMsg.value = "";
    try {
      const [saveList, cardList, globalData] = await Promise.all([
        listSaves(),
        listWorldCards(),
        getGlobalGameData(),
      ]);
      saves.value = saveList;
      worldCards.value = cardList;
      gameSettings.value = globalData.gameSettings;
      applyAiSettings(
        globalData.aiSettings.models,
        globalData.aiSettings.defaultModelId,
      );
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

  function selectAiModel(id: string) {
    const model = aiModels.value.find((item) => item.id === id);
    if (!model) {
      errorMsg.value = "模型不存在";
      return;
    }
    editingModelId.value = id;
    aiDraft.value = { ...model };
    modelCheckMsg.value = "";
    modelCheckOk.value = null;
  }

  function resetAiDraft() {
    editingModelId.value = null;
    aiDraft.value = defaultAiModelDraft();
    modelCheckMsg.value = "";
    modelCheckOk.value = null;
  }

  async function testAiDraft() {
    errorMsg.value = "";
    modelCheckMsg.value = "";
    modelCheckOk.value = null;
    try {
      const result = await testModelProvider(aiDraft.value);
      modelCheckMsg.value = result.message;
      modelCheckOk.value = true;
    } catch (err) {
      modelCheckMsg.value = normalizeError(err);
      modelCheckOk.value = false;
    }
  }

  async function saveAiModel() {
    errorMsg.value = "";
    modelCheckMsg.value = "";
    try {
      const saved = await upsertAiModel({
        ...aiDraft.value,
        id: editingModelId.value ?? aiDraft.value.id,
      });
      await refreshHome();
      selectAiModel(saved.id);
    } catch (err) {
      setError(err);
    }
  }

  async function removeAiModel(modelId: string) {
    errorMsg.value = "";
    modelCheckMsg.value = "";
    try {
      await deleteAiModel(modelId);
      await refreshHome();
      if (editingModelId.value === modelId) {
        resetAiDraft();
      }
    } catch (err) {
      setError(err);
    }
  }

  async function markDefaultAiModel(modelId: string) {
    errorMsg.value = "";
    try {
      const settings = await setDefaultAiModel(modelId);
      applyAiSettings(settings.models, settings.defaultModelId);
    } catch (err) {
      setError(err);
    }
  }

  async function createNewSave() {
    errorMsg.value = "";
    try {
      if (!newSave.value.modelProfileId) {
        throw new Error("请先在 AI 设置里配置并选择一个默认模型");
      }
      const worldInit = await generateWorld(
        newSave.value.worldCardId,
        newSave.value.playerRole,
      );
      const meta = await createSave({ ...newSave.value, worldInit });
      await refreshHome();
      await openSave(meta.id, true);
    } catch (err) {
      setError(err);
    }
  }

  async function saveGlobalGameData() {
    errorMsg.value = "";
    try {
      await updateGlobalGameData({
        gameSettings: gameSettings.value,
        aiSettings: {
          models: aiModels.value,
          defaultModelId: defaultModelId.value,
        },
      });
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
      narrationText.value = "";
      const result = await runTurnStream(
        { saveId: activeSave.value.meta.id, optionId },
        (chunk) => {
          narrationText.value += chunk;
        },
      );
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
      narrationText.value = "";
      const result = await runTurnStream(
        {
          saveId: activeSave.value.meta.id,
          customText: customInput.value.trim(),
        },
        (chunk) => {
          narrationText.value += chunk;
        },
      );
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
      stateChanges.value = [
        result.message,
        ...result.triggeredEventIds.map((id) => `触发 ${id}`),
      ];
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
    modelCheckOk,
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
    gameSettings,
    aiModels,
    defaultModelId,
    editingModelId,
    aiDraft,
    reachableLocations,
    goMenu,
    setView,
    refreshHome,
    openSave,
    removeSave,
    selectAiModel,
    resetAiDraft,
    testAiDraft,
    saveAiModel,
    removeAiModel,
    markDefaultAiModel,
    saveGlobalGameData,
    createNewSave,
    submitOption,
    submitCustom,
    move,
    importCardFromText,
    exportCard,
  };
}
