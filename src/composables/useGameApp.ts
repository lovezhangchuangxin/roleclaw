import { computed, ref } from "vue";
import {
  createSave,
  deleteAiModel,
  deleteSave,
  deleteWorldCard,
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
  StoryState,
  TaskState,
  RelationshipDelta,
  TurnStreamPayload,
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
  const streamingNarrationText = ref("");
  const streamingStructuredPreview = ref<{
    storyState: StoryState | null;
    taskState: TaskState | null;
    relationshipDeltas: RelationshipDelta[];
    options: DialogueOption[];
    stateChangesPreview: string[];
  }>({
    storyState: null,
    taskState: null,
    relationshipDeltas: [],
    options: [],
    stateChangesPreview: [],
  });
  const turnStreamingStatus = ref<"idle" | "running" | "error">("idle");
  const turnStreamingHint = ref("");
  const turnStreamingHintPending = ref("");
  const turnStreamingHintLines = ref<string[]>([]);
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
  const modelImportText = ref("");

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
        const lastLog = bundle.recentLogs[bundle.recentLogs.length - 1];
        if (lastLog?.output) {
          narrationText.value = lastLog.output.narration || bundle.snapshot.worldSummary;
          stateChanges.value = lastLog.output.stateChangesPreview ?? ["存档已加载"];
          options.value = lastLog.output.options ?? [];
          streamingStructuredPreview.value = {
            storyState: lastLog.output.storyState ?? null,
            taskState: lastLog.output.taskState ?? null,
            relationshipDeltas: lastLog.output.relationshipDeltas ?? [],
            options: lastLog.output.options ?? [],
            stateChangesPreview: lastLog.output.stateChangesPreview ?? [],
          };
        } else {
          narrationText.value = bundle.snapshot.worldSummary;
          stateChanges.value = ["存档已加载"];
          options.value = [];
          if (bundle.snapshot.turn === 0) {
            await generateOpeningScene(bundle.meta.id);
          }
        }
      }
      turnStreamingStatus.value = "idle";
      streamingNarrationText.value = "";
    } catch (err) {
      setError(err);
    }
  }

  async function generateOpeningScene(saveId: string) {
    try {
      resetTurnStreaming();
      narrationText.value = "";
      stateChanges.value = ["正在生成开场场景..."];
      turnStreamingStatus.value = "running";
      const result = await runTurnStream(
        {
          saveId,
          draft: true,
          customText:
            "请生成本次冒险的开场场景，描述当前局面，并给出三个互斥可执行选项。",
        },
        onTurnStreamEvent,
      );
      narrationText.value = result.narration;
      stateChanges.value = result.stateChangesPreview.length
        ? result.stateChangesPreview
        : ["开场场景已生成"];
      options.value = result.options;
      streamingStructuredPreview.value = {
        storyState: result.storyState ?? null,
        taskState: result.taskState ?? null,
        relationshipDeltas: result.relationshipDeltas ?? [],
        options: result.options,
        stateChangesPreview: result.stateChangesPreview,
      };
      turnStreamingStatus.value = "idle";
    } catch (err) {
      turnStreamingStatus.value = "error";
      setError(err);
      // Compatibility fallback when opening generation fails.
      if (!narrationText.value.trim()) {
        narrationText.value = "开场场景生成失败，请重试一次或手动输入第四选项开始。";
      }
      if (options.value.length === 0) {
        options.value = [
          { id: "opt_1", kind: "approach", text: "观察周围并整理当前局势" },
          { id: "opt_2", kind: "social", text: "接近附近关键人物并试探交流" },
          { id: "opt_3", kind: "risk", text: "主动探索可疑区域寻找突破口" },
        ];
      }
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

  function normalizeImportModel(input: unknown, fallbackId: string): AiModelProfile {
    if (!input || typeof input !== "object") {
      throw new Error("模型配置项必须是对象");
    }
    const draft = input as Partial<AiModelProfile>;
    const providerType =
      draft.providerType === "openai_compatible" ? draft.providerType : "openai_compatible";
    const provider = typeof draft.provider === "string" ? draft.provider.trim() : "";
    const baseUrl = typeof draft.baseUrl === "string" ? draft.baseUrl.trim() : "";
    const model = typeof draft.model === "string" ? draft.model.trim() : "";
    if (!provider || !baseUrl || !model) {
      throw new Error("模型配置缺少必填字段：provider/baseUrl/model");
    }
    const temperatureRaw = Number(draft.temperature);
    const timeoutRaw = Number(draft.timeoutMs);
    const maxTokensRaw = draft.maxTokens == null ? undefined : Number(draft.maxTokens);
    return {
      id: typeof draft.id === "string" && draft.id.trim() ? draft.id.trim() : fallbackId,
      providerType,
      provider,
      baseUrl,
      model,
      apiKey: typeof draft.apiKey === "string" ? draft.apiKey : "",
      temperature: Number.isFinite(temperatureRaw) ? temperatureRaw : 0.7,
      maxTokens:
        maxTokensRaw == null || Number.isNaN(maxTokensRaw) ? undefined : Math.max(1, Math.floor(maxTokensRaw)),
      timeoutMs: Number.isFinite(timeoutRaw) ? Math.max(1000, Math.floor(timeoutRaw)) : 25000,
      updatedAt: typeof draft.updatedAt === "string" ? draft.updatedAt : "",
    };
  }

  function exportAiModels(modelIds: string[] = []): string {
    errorMsg.value = "";
    const picked = modelIds.length
      ? aiModels.value.filter((item) => modelIds.includes(item.id))
      : aiModels.value;
    const payload = {
      schemaVersion: "roleclaw.ai-models.v1",
      exportedAt: new Date().toISOString(),
      defaultModelId: defaultModelId.value,
      models: picked.map((item) => ({ ...item })),
    };
    return JSON.stringify(payload, null, 2);
  }

  async function importAiModelsFromText() {
    if (!modelImportText.value.trim()) {
      throw new Error("请先粘贴模型配置 JSON");
    }
    errorMsg.value = "";
    const raw = JSON.parse(modelImportText.value.trim()) as unknown;
    let sourceModels: unknown[] = [];
    let sourceDefaultId: string | null = null;
    if (Array.isArray(raw)) {
      sourceModels = raw;
    } else if (raw && typeof raw === "object") {
      const obj = raw as { models?: unknown; defaultModelId?: unknown };
      if (Array.isArray(obj.models)) {
        sourceModels = obj.models;
      } else {
        sourceModels = [raw];
      }
      if (typeof obj.defaultModelId === "string" && obj.defaultModelId.trim()) {
        sourceDefaultId = obj.defaultModelId.trim();
      }
    } else {
      throw new Error("导入内容必须是对象或数组");
    }

    if (sourceModels.length === 0) {
      throw new Error("未解析到任何模型配置");
    }

    const normalized = sourceModels.map((item, idx) =>
      normalizeImportModel(item, `import_${Date.now()}_${idx + 1}`),
    );
    const savedModels: AiModelProfile[] = [];
    for (const model of normalized) {
      const saved = await upsertAiModel(model);
      savedModels.push(saved);
    }

    await refreshHome();
    if (sourceDefaultId) {
      const defaultIdx = normalized.findIndex((item) => item.id === sourceDefaultId);
      if (defaultIdx >= 0 && savedModels[defaultIdx]) {
        await markDefaultAiModel(savedModels[defaultIdx].id);
      }
    }
    return savedModels.length;
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

  const REASONING_LINE_TARGET_CHARS = 28;
  const REASONING_LINE_MIN_CHARS = 16;
  const REASONING_MAX_LINES = 2;

  function normalizeReasoningChunk(input: string): string {
    return input.replace(/\s+/g, " ").trim();
  }

  function splitReasoningLine(pending: string): { line: string; rest: string } | null {
    if (!pending.trim()) {
      return null;
    }
    const text = pending.trim();
    if (text.length < REASONING_LINE_MIN_CHARS) {
      return null;
    }

    const punct = /[。！？；.!?;]/g;
    let match: RegExpExecArray | null = null;
    let bestEnd = -1;
    while ((match = punct.exec(text)) !== null) {
      const end = match.index + 1;
      if (end >= REASONING_LINE_MIN_CHARS && end <= REASONING_LINE_TARGET_CHARS + 8) {
        bestEnd = end;
      }
    }
    if (bestEnd > 0) {
      return {
        line: text.slice(0, bestEnd).trim(),
        rest: text.slice(bestEnd).trim(),
      };
    }

    if (text.length >= REASONING_LINE_TARGET_CHARS) {
      return {
        line: text.slice(0, REASONING_LINE_TARGET_CHARS).trim(),
        rest: text.slice(REASONING_LINE_TARGET_CHARS).trim(),
      };
    }
    return null;
  }

  function flushReasoningHint(force = false) {
    let pending = turnStreamingHintPending.value;
    while (true) {
      const next = splitReasoningLine(pending);
      if (!next) {
        break;
      }
      if (next.line) {
        turnStreamingHintLines.value.push(next.line);
      }
      pending = next.rest;
    }

    if (force && pending.trim()) {
      turnStreamingHintLines.value.push(pending.trim());
      pending = "";
    }

    if (turnStreamingHintLines.value.length > REASONING_MAX_LINES) {
      turnStreamingHintLines.value = turnStreamingHintLines.value.slice(
        turnStreamingHintLines.value.length - REASONING_MAX_LINES,
      );
    }

    turnStreamingHintPending.value = pending;
    turnStreamingHint.value = turnStreamingHintLines.value.length
      ? `AI 思考中：${turnStreamingHintLines.value.join(" ")}`
      : "";
  }

  async function applyTurnResult(result: TurnResult) {
    narrationText.value = result.narration;
    stateChanges.value = result.stateChangesPreview;
    options.value = result.options;
    streamingStructuredPreview.value = {
      storyState: result.storyState ?? null,
      taskState: result.taskState ?? null,
      relationshipDeltas: result.relationshipDeltas ?? [],
      options: result.options,
      stateChangesPreview: result.stateChangesPreview,
    };
    streamingNarrationText.value = result.narration;
    turnStreamingStatus.value = "idle";
    if (activeSave.value) {
      await openSave(activeSave.value.meta.id, false);
    }
  }

  function resetTurnStreaming() {
    streamingNarrationText.value = "";
    turnStreamingHint.value = "";
    turnStreamingHintPending.value = "";
    turnStreamingHintLines.value = [];
    streamingStructuredPreview.value = {
      storyState: null,
      taskState: null,
      relationshipDeltas: [],
      options: [],
      stateChangesPreview: [],
    };
  }

  function applyStreamPreviewFromJsonChunk() {
    try {
      const nextRaw = streamingNarrationText.value;
      const parsed = parsePartialJson(nextRaw, ALL) as Record<string, unknown>;
      const narration = parsed.narration;
      if (typeof narration === "string") {
        narrationText.value = narration;
      }
      const optionsRaw = parsed.options;
      if (Array.isArray(optionsRaw)) {
        const parsedOptions = optionsRaw
          .map((item, idx) => {
            if (!item || typeof item !== "object") return null;
            const record = item as Record<string, unknown>;
            const text = typeof record.text === "string" ? record.text : "";
            if (!text.trim()) return null;
            return {
              id: `opt_${idx + 1}`,
              kind: typeof record.kind === "string" ? record.kind : "approach",
              text,
            } as DialogueOption;
          })
          .filter((item): item is DialogueOption => Boolean(item));
        if (parsedOptions.length > 0) {
          streamingStructuredPreview.value.options = parsedOptions;
          options.value = parsedOptions;
        }
      }
      if (parsed.storyState && typeof parsed.storyState === "object") {
        streamingStructuredPreview.value.storyState = parsed.storyState as StoryState;
      }
      if (parsed.taskState && typeof parsed.taskState === "object") {
        streamingStructuredPreview.value.taskState = parsed.taskState as TaskState;
      }
      if (Array.isArray(parsed.relationshipDeltas)) {
        streamingStructuredPreview.value.relationshipDeltas =
          parsed.relationshipDeltas as RelationshipDelta[];
      }
      if (Array.isArray(parsed.stateChangesPreview)) {
        streamingStructuredPreview.value.stateChangesPreview =
          parsed.stateChangesPreview.filter((v): v is string => typeof v === "string");
        stateChanges.value = streamingStructuredPreview.value.stateChangesPreview;
      }
    } catch {
      // Ignore partial parse errors while stream is still incomplete.
    }
  }

  function onTurnStreamEvent(payload: TurnStreamPayload) {
    if (payload.eventType === "status") {
      if (payload.phase === "start" || payload.phase === "preview" || payload.phase === "delta") {
        turnStreamingStatus.value = "running";
      }
      if (payload.phase === "final" || payload.phase === "end") {
        turnStreamingStatus.value = "idle";
        flushReasoningHint(true);
        turnStreamingHint.value = "";
      }
      const reasoning = payload.data?.reasoning;
      if (typeof reasoning === "string" && reasoning.trim()) {
        const normalized = normalizeReasoningChunk(reasoning);
        if (normalized) {
          turnStreamingHintPending.value = `${turnStreamingHintPending.value} ${normalized}`.trim();
          flushReasoningHint(false);
        }
      }
      return;
    }
    if (payload.eventType === "error") {
      turnStreamingStatus.value = "error";
      const msg = payload.data?.message;
      if (typeof msg === "string" && msg.trim()) {
        errorMsg.value = msg;
      }
      return;
    }
    if (payload.eventType === "narration_delta" && payload.chunk) {
      narrationText.value += payload.chunk;
      return;
    }
    if (payload.eventType === "json_delta" && payload.chunk) {
      streamingNarrationText.value += payload.chunk;
      applyStreamPreviewFromJsonChunk();
      return;
    }
    if (payload.eventType === "options_preview" && payload.data) {
      const raw = payload.data.options;
      if (Array.isArray(raw)) {
        const next = raw.filter((item): item is DialogueOption => {
          return Boolean(item && typeof item === "object");
        });
        if (next.length > 0) {
          streamingStructuredPreview.value.options = next;
          options.value = next;
        }
      }
      return;
    }
    if (payload.eventType === "state_preview" && payload.data) {
      const stateData = payload.data;
      if (stateData.storyState && typeof stateData.storyState === "object") {
        streamingStructuredPreview.value.storyState = stateData.storyState as StoryState;
      }
      if (stateData.taskState && typeof stateData.taskState === "object") {
        streamingStructuredPreview.value.taskState = stateData.taskState as TaskState;
      }
      if (Array.isArray(stateData.relationshipDeltas)) {
        streamingStructuredPreview.value.relationshipDeltas =
          stateData.relationshipDeltas as RelationshipDelta[];
      }
      if (Array.isArray(stateData.stateChangesPreview)) {
        streamingStructuredPreview.value.stateChangesPreview =
          stateData.stateChangesPreview as string[];
      }
      stateChanges.value = streamingStructuredPreview.value.stateChangesPreview;
    }
  }

  async function submitOption(optionId: string) {
    if (!activeSave.value || turnStreamingStatus.value === "running") {
      return;
    }
    errorMsg.value = "";
    try {
      resetTurnStreaming();
      narrationText.value = "";
      turnStreamingStatus.value = "running";
      const result = await runTurnStream(
        { saveId: activeSave.value.meta.id, optionId },
        onTurnStreamEvent,
      );
      await applyTurnResult(result);
    } catch (err) {
      turnStreamingStatus.value = "error";
      turnStreamingHint.value = "";
      setError(err);
    }
  }

  async function submitCustom() {
    if (
      !activeSave.value ||
      turnStreamingStatus.value === "running" ||
      !customInput.value.trim()
    ) {
      return;
    }
    errorMsg.value = "";
    try {
      resetTurnStreaming();
      narrationText.value = "";
      turnStreamingStatus.value = "running";
      const result = await runTurnStream(
        {
          saveId: activeSave.value.meta.id,
          customText: customInput.value.trim(),
        },
        onTurnStreamEvent,
      );
      customInput.value = "";
      await applyTurnResult(result);
    } catch (err) {
      turnStreamingStatus.value = "error";
      turnStreamingHint.value = "";
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

  async function removeWorldCard(cardId: string) {
    errorMsg.value = "";
    try {
      await deleteWorldCard(cardId);
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
    streamingNarrationText,
    streamingStructuredPreview,
    turnStreamingStatus,
    turnStreamingHint,
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
    modelImportText,
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
    exportAiModels,
    importAiModelsFromText,
    saveGlobalGameData,
    createNewSave,
    submitOption,
    submitCustom,
    move,
    importCardFromText,
    exportCard,
    duplicateCard,
    removeWorldCard,
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
