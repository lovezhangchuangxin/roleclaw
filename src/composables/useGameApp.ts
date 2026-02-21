import { computed, ref } from "vue";
import {
  createSave,
  deleteAiModel,
  deleteSave,
  exportWorldCard,
  forkSave,
  generateWorldCardWithAiStream,
  getGlobalGameData,
  generateWorld,
  importWorldCard,
  listEvents,
  listSaves,
  listWorldCards,
  loadSave,
  moveToLocation,
  replaySave,
  runTurnStream,
  setDefaultAiModel,
  testModelProvider,
  updateGlobalGameData,
  upsertAiModel,
} from "@/lib/api";
import { normalizeError } from "@/lib/errors";
import { parse as parsePartialJson, ALL } from "partial-json";
import type {
  AiModelProfile,
  CreateSaveConfig,
  DialogueOption,
  EventLogEntry,
  GameSettings,
  NpcProfile,
  ReplayResult,
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
  | "settings"
  | "replay";

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
  const aiWorldCardPrompt = ref("");
  const aiWorldCardGenerating = ref(false);
  const aiGeneratedWorldCard = ref<WorldCard | null>(null);
  const aiWorldCardStreamText = ref("");
  const aiWorldCardStreamParsedOk = ref(false);
  const replayPreview = ref<EventLogEntry[]>([]);
  const replayResult = ref<ReplayResult | null>(null);
  const replaySelectedTurn = ref<number | null>(null);
  const replayNextCursor = ref<number | null>(null);
  const replayHasMore = ref(false);
  const replayLoading = ref(false);

  const saves = ref<SaveMeta[]>([]);
  const worldCards = ref<WorldCard[]>([]);
  const activeSave = ref<SaveBundle | null>(null);

  const newSave = ref<CreateSaveConfig>({
    saveName: "新冒险",
    playerRole: "主角",
    worldCardId: "",
    modelProfileId: "",
  });
  const gameSettings = ref<GameSettings>({
    theme: "default",
    messageSpeed: "normal",
    fontScale: 1,
    uiZoom: 1,
    logLevel: "info",
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

  async function saveEditedCard(card: WorldCard) {
    errorMsg.value = "";
    try {
      const normalizedCard: WorldCard = {
        ...card,
        worldbook: {
          ...card.worldbook,
          coreConflicts: card.worldbook.coreConflicts.filter((line) => line.trim()),
        },
        map: {
          ...card.map,
          nodes: card.map.nodes.map((node) => ({
            ...node,
            tags: node.tags.filter((tag) => tag.trim()),
          })),
          edges: card.map.edges.map((edge) => ({
            ...edge,
            unlockConditions: edge.unlockConditions.filter((line) => line.trim()),
          })),
        },
        npcs: card.npcs.map((arch: NpcProfile) => ({
          ...arch,
          personality: arch.personality.filter((trait) => trait.trim()),
        })),
        events: card.events.filter(
          (event) => event.id.trim() && event.name.trim() && event.prompt.trim(),
        ),
        chapterGoals: card.chapterGoals.filter(
          (goal) => goal.id.trim() && goal.title.trim() && goal.prompt.trim(),
        ),
      };
      await importWorldCard(JSON.stringify(normalizedCard));
      await refreshHome();
    } catch (err) {
      setError(err);
    }
  }

  async function generateCardDraftWithAi(prompt: string) {
    if (!prompt.trim()) {
      errorMsg.value = "请输入用于生成世界卡的提示词";
      return;
    }
    errorMsg.value = "";
    aiWorldCardGenerating.value = true;
    aiWorldCardStreamText.value = "";
    aiWorldCardStreamParsedOk.value = false;
    try {
      const card = await generateWorldCardWithAiStream(
        prompt.trim(),
        (chunk) => {
          aiWorldCardStreamText.value += chunk;
          try {
            const partial = parsePartialJson(aiWorldCardStreamText.value, ALL) as Record<
              string,
              unknown
            >;
            aiWorldCardStreamParsedOk.value = typeof partial === "object" && partial !== null;
          } catch {
            aiWorldCardStreamParsedOk.value = false;
          }
        },
        defaultModelId.value ?? undefined,
      );
      aiGeneratedWorldCard.value = card;
      aiWorldCardPrompt.value = prompt;
      aiWorldCardStreamParsedOk.value = true;
    } catch (err) {
      setError(err);
    } finally {
      aiWorldCardGenerating.value = false;
    }
  }

  async function duplicateCard(cardId: string) {
    const source = worldCards.value.find((card) => card.id === cardId);
    if (!source) {
      errorMsg.value = "世界卡不存在";
      return;
    }
    const clone = {
      ...source,
      id: `${source.id}_copy_${Date.now()}`,
      name: `${source.name} 副本`,
      contentVersion: source.contentVersion + 1,
    };
    errorMsg.value = "";
    try {
      await importWorldCard(JSON.stringify(clone));
      await refreshHome();
    } catch (err) {
      setError(err);
    }
  }

  async function refreshReplayConsistency() {
    if (!activeSave.value) {
      return;
    }
    errorMsg.value = "";
    try {
      const replay = await replaySave(activeSave.value.meta.id);
      replayResult.value = replay;
    } catch (err) {
      setError(err);
    }
  }

  async function loadReplayTimeline(reset = true) {
    if (!activeSave.value) {
      return;
    }
    if (replayLoading.value) {
      return;
    }
    if (!reset && !replayHasMore.value) {
      return;
    }
    replayLoading.value = true;
    errorMsg.value = "";
    try {
      const page = await listEvents(
        activeSave.value.meta.id,
        reset ? undefined : replayNextCursor.value ?? undefined,
      );
      if (reset) {
        replayPreview.value = page.items;
      } else {
        replayPreview.value = [...page.items, ...replayPreview.value];
      }
      replayNextCursor.value = page.nextCursor ?? null;
      replayHasMore.value = page.nextCursor != null;
    } catch (err) {
      setError(err);
    } finally {
      replayLoading.value = false;
    }
  }

  async function openReplayView() {
    if (!activeSave.value) {
      errorMsg.value = "请先加载一个存档";
      return;
    }
    view.value = "replay";
    await Promise.all([refreshReplayConsistency(), loadReplayTimeline(true)]);
    if (replayPreview.value.length) {
      replaySelectedTurn.value =
        replayPreview.value[replayPreview.value.length - 1].turn;
    }
  }

  async function refreshReplayData() {
    await Promise.all([refreshReplayConsistency(), loadReplayTimeline(true)]);
  }

  async function forkAtTurn(turn: number) {
    if (!activeSave.value) {
      return;
    }
    const name = `${activeSave.value.meta.name} 分叉 T${turn}`;
    errorMsg.value = "";
    try {
      const meta = await forkSave(activeSave.value.meta.id, turn, name);
      await refreshHome();
      await openSave(meta.id, true);
    } catch (err) {
      setError(err);
    }
  }

  async function forkActiveSave() {
    if (!activeSave.value) {
      return;
    }
    const fromTurn = activeSave.value.snapshot.turn;
    const name = `${activeSave.value.meta.name} 分叉 T${fromTurn}`;
    errorMsg.value = "";
    try {
      const meta = await forkSave(activeSave.value.meta.id, fromTurn, name);
      await refreshHome();
      await openSave(meta.id, true);
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
    aiWorldCardPrompt,
    aiWorldCardGenerating,
    aiGeneratedWorldCard,
    aiWorldCardStreamText,
    aiWorldCardStreamParsedOk,
    replayPreview,
    replayResult,
    replaySelectedTurn,
    replayNextCursor,
    replayHasMore,
    replayLoading,
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
    duplicateCard,
    saveEditedCard,
    generateCardDraftWithAi,
    refreshReplayConsistency,
    loadReplayTimeline,
    refreshReplayData,
    openReplayView,
    forkAtTurn,
    forkActiveSave,
  };
}
