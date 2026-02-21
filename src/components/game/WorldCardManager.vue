<template>
  <div class="panel w-full">
    <h2 class="panel-title">世界卡管理（V2）</h2>
    <p class="mb-4 text-sm game-text-muted">编辑世界观、地点地图、固定 NPC、事件 Prompt 与章节目标。保存时会自动合并为 JSON。</p>

    <div class="mb-4 flex flex-wrap items-center gap-2">
      <button class="btn btn-primary" @click="createNewCard">新建世界卡（表单）</button>
      <details class="advanced-box">
        <summary class="advanced-summary">AI 生成草稿</summary>
        <div class="mt-2 grid gap-2">
          <textarea
            :value="aiWorldCardPrompt"
            class="input h-24 w-full"
            placeholder="输入你的创意：世界观、风格、核心冲突、地点和角色方向..."
            @input="$emit('update:aiWorldCardPrompt', ($event.target as HTMLTextAreaElement).value)"
          />
          <div class="flex items-center gap-2">
            <button class="btn" :disabled="aiWorldCardGenerating" @click="$emit('generateCardByAi', aiWorldCardPrompt)">
              {{ aiWorldCardGenerating ? "AI 生成中..." : "生成世界卡草稿" }}
            </button>
            <span class="text-xs game-text-muted">生成后会自动填入当前编辑表单，可继续手动调整再保存。</span>
          </div>
          <div v-if="aiWorldCardStreamText" class="grid gap-1">
            <div class="flex items-center justify-between text-xs game-text-muted">
              <span>流式 JSON 预览（SSE）</span>
              <span>
                {{
                  aiWorldCardGenerating
                    ? aiWorldCardStreamParsedOk
                      ? "流式解析中（已可预览）"
                      : "流式接收中"
                    : aiWorldCardStreamParsedOk
                      ? "生成完成（结构可解析）"
                      : "生成完成（结构仍不完整）"
                }}
              </span>
            </div>
            <textarea :value="aiWorldCardStreamText" class="input h-28 w-full font-mono text-xs" readonly />
          </div>
        </div>
      </details>
      <details class="advanced-box">
        <summary class="advanced-summary">高级：JSON 导入/导出</summary>
        <div class="mt-2 grid gap-3 md:grid-cols-2">
          <div>
            <h3 class="mb-2 text-sm font-medium">导入（粘贴 JSON）</h3>
            <textarea
              :value="cardImportText"
              class="input h-36 w-full"
              placeholder='{"id":"my_card","schemaVersion":"2.0.0", ...}'
              @input="$emit('update:cardImportText', ($event.target as HTMLTextAreaElement).value)"
            />
            <button class="btn mt-2" @click="$emit('importCard')">导入世界卡</button>
          </div>
          <div>
            <h3 class="mb-2 text-sm font-medium">导出目标路径</h3>
            <input
              :value="cardExportPath"
              class="input w-full"
              placeholder="/tmp/world-card-v2.json"
              @input="$emit('update:cardExportPath', ($event.target as HTMLInputElement).value)"
            />
            <p class="mt-2 text-xs game-text-muted">点击列表中的“导出”会输出到该路径。</p>
          </div>
        </div>
      </details>
    </div>

    <div class="mb-4 grid gap-4 lg:grid-cols-[0.9fr_1.6fr]">
      <div>
        <div class="mb-2 flex items-center justify-between gap-2">
          <h3 class="font-medium">世界卡列表</h3>
          <input v-model="search" class="input w-44" placeholder="搜索名称/ID" />
        </div>
        <div class="space-y-2">
          <button
            v-for="card in filteredCards"
            :key="card.id"
            class="card-row"
            :class="{ 'card-row-active': selectedCardId === card.id }"
            @click="selectCard(card.id)"
          >
            <span>
              <b>{{ card.name }}</b>
              <small class="game-text-muted block">{{ card.id }} · schema {{ card.schemaVersion }}</small>
            </span>
            <span class="flex gap-2">
              <span class="mini-btn" @click.stop="$emit('duplicateCard', card.id)">复制</span>
              <span class="mini-btn" @click.stop="$emit('exportCard', card.id)">导出</span>
            </span>
          </button>
          <p v-if="filteredCards.length === 0" class="text-sm game-text-muted">暂无匹配世界卡。</p>
        </div>
      </div>

      <div v-if="draftCard" class="space-y-4">
        <div class="flex items-center justify-between gap-2">
          <h3 class="font-medium">可视化编辑</h3>
          <div class="flex gap-2">
            <button class="btn" @click="resetDraft">重置</button>
            <button class="btn btn-primary" @click="emitSave">保存世界卡</button>
          </div>
        </div>

        <div class="section">
          <h4 class="section-title">基础信息</h4>
          <div class="grid gap-3 md:grid-cols-2">
            <label class="field">
              <span>ID</span>
              <input v-model="draftCard.id" class="input" />
            </label>
            <label class="field">
              <span>名称</span>
              <input v-model="draftCard.name" class="input" />
            </label>
            <label class="field">
              <span>Schema 版本</span>
              <input v-model="draftCard.schemaVersion" class="input" />
            </label>
            <label class="field">
              <span>内容版本</span>
              <input v-model.number="draftCard.contentVersion" class="input" type="number" min="1" />
            </label>
          </div>
        </div>

        <div class="section">
          <h4 class="section-title">世界观</h4>
          <div class="grid gap-3">
            <label class="field">
              <span>世界标题</span>
              <input v-model="draftCard.worldbook.title" class="input" />
            </label>
            <label class="field">
              <span>世界介绍</span>
              <textarea v-model="draftCard.worldbook.overview" class="input h-20" />
            </label>
            <label class="field">
              <span>游戏背景</span>
              <textarea v-model="draftCard.worldbook.background" class="input h-24" />
            </label>
            <label class="field">
              <span>核心冲突（逗号分隔）</span>
              <input :value="draftCard.worldbook.coreConflicts.join(',')" class="input" @input="setCoreConflicts(($event.target as HTMLInputElement).value)" />
            </label>
            <label class="field">
              <span>玩法风格</span>
              <input v-model="draftCard.worldbook.playStyle" class="input" />
            </label>
          </div>
        </div>

        <div class="section">
          <h4 class="section-title">地点地图</h4>
          <WorldCardMapEditor v-model="draftCard.map" />
        </div>

        <div class="section">
          <div class="mb-2 flex items-center justify-between">
            <h4 class="section-title mb-0">固定 NPC（简版）</h4>
            <button class="btn" @click="addNpc">新增 NPC</button>
          </div>
          <div class="space-y-2 max-h-72 overflow-auto pr-1">
            <div v-for="(npc, idx) in draftCard.npcs" :key="npc.id + idx" class="edit-stack">
              <label class="field">
                <span>姓名</span>
                <input v-model="npc.name" class="input" placeholder="NPC 名称" />
              </label>
              <label class="field">
                <span>性格（逗号分隔）</span>
                <input :value="npc.personality.join(',')" class="input" placeholder="冷静,敏锐" @input="setNpcPersonality(idx, ($event.target as HTMLInputElement).value)" />
              </label>
              <label class="field">
                <span>身份</span>
                <input v-model="npc.identity" class="input" placeholder="身份/作用" />
              </label>
              <button class="mini-btn mini-btn-danger" @click="removeNpc(idx)">删除 NPC</button>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="mb-2 flex items-center justify-between">
            <h4 class="section-title mb-0">固定事件（Prompt）</h4>
            <button class="btn" @click="addEvent">新增事件</button>
          </div>
          <div class="space-y-2 max-h-80 overflow-auto pr-1">
            <div v-for="(event, idx) in draftCard.events" :key="event.id + idx" class="edit-stack">
              <label class="field">
                <span>事件标题</span>
                <input v-model="event.name" class="input" placeholder="事件名" />
              </label>
              <label class="field">
                <span>事件 Prompt（将喂给 AI 推进剧情）</span>
                <textarea v-model="event.prompt" class="input h-24" placeholder="描述该事件在什么条件下应该推动怎样的剧情" />
              </label>
              <button class="mini-btn mini-btn-danger" @click="removeEvent(idx)">删除事件</button>
            </div>
          </div>
        </div>

        <div class="section">
          <div class="mb-2 flex items-center justify-between">
            <h4 class="section-title mb-0">章节目标（顺序推进）</h4>
            <button class="btn" @click="addChapter">新增章节目标</button>
          </div>
          <div class="space-y-2 max-h-80 overflow-auto pr-1">
            <div v-for="(goal, idx) in draftCard.chapterGoals" :key="goal.id + idx" class="edit-stack">
              <label class="field">
                <span>章节标题</span>
                <input v-model="goal.title" class="input" placeholder="章节标题" />
              </label>
              <label class="field">
                <span>章节 Prompt（告诉 AI 当前剧情阶段）</span>
                <textarea v-model="goal.prompt" class="input h-24" placeholder="当前章节剧情走向、目标、氛围" />
              </label>
              <button class="mini-btn mini-btn-danger" @click="removeChapter(idx)">删除章节</button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { CardEvent, ChapterGoal, NpcProfile, WorldCard } from "@/types";
import WorldCardMapEditor from "@/components/game/WorldCardMapEditor.vue";

const props = defineProps<{
  worldCards: WorldCard[];
  cardImportText: string;
  cardExportPath: string;
  aiWorldCardPrompt: string;
  aiWorldCardGenerating: boolean;
  aiGeneratedCard: WorldCard | null;
  aiWorldCardStreamText: string;
  aiWorldCardStreamParsedOk: boolean;
}>();

const emit = defineEmits<{
  importCard: [];
  exportCard: [cardId: string];
  duplicateCard: [cardId: string];
  saveEditedCard: [card: WorldCard];
  "update:cardImportText": [value: string];
  "update:cardExportPath": [value: string];
  "update:aiWorldCardPrompt": [value: string];
  generateCardByAi: [prompt: string];
}>();

const search = ref("");
const selectedCardId = ref<string>("");
const draftCard = ref<WorldCard | null>(null);

const filteredCards = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) {
    return props.worldCards;
  }
  return props.worldCards.filter((card) => {
    return card.name.toLowerCase().includes(keyword) || card.id.toLowerCase().includes(keyword);
  });
});

function cloneCard(card: WorldCard): WorldCard {
  return JSON.parse(JSON.stringify(card)) as WorldCard;
}

function selectCard(cardId: string) {
  selectedCardId.value = cardId;
  const card = props.worldCards.find((item) => item.id === cardId);
  draftCard.value = card ? cloneCard(card) : null;
}

function createNewCard() {
  selectedCardId.value = "";
  const now = Date.now();
  draftCard.value = {
    id: `card_${now}`,
    name: "新世界卡",
    schemaVersion: "2.0.0",
    contentVersion: 1,
    worldbook: {
      title: "新世界",
      overview: "",
      background: "",
      coreConflicts: [],
      playStyle: "",
    },
    map: {
      nodes: [
        {
          id: `loc_${now}`,
          name: "起始地点",
          description: "",
          tags: [],
          x: 180,
          y: 180,
        },
      ],
      edges: [],
      startNodeId: `loc_${now}`,
      canvas: {
        width: 900,
        height: 560,
      },
    },
    npcs: [],
    events: [],
    chapterGoals: [],
  };
}

function resetDraft() {
  if (!selectedCardId.value) {
    createNewCard();
    return;
  }
  selectCard(selectedCardId.value);
}

function emitSave() {
  if (!draftCard.value) return;
  emit("saveEditedCard", cloneCard(draftCard.value));
}

function splitCsv(value: string): string[] {
  return value.split(",").map((item) => item.trim()).filter(Boolean);
}

function setCoreConflicts(value: string) {
  if (!draftCard.value) return;
  draftCard.value.worldbook.coreConflicts = splitCsv(value);
}

function addNpc() {
  if (!draftCard.value) return;
  draftCard.value.npcs.push({
    id: `npc_${Date.now()}`,
    name: "",
    personality: [],
    identity: "",
  } satisfies NpcProfile);
}

function removeNpc(index: number) {
  draftCard.value?.npcs.splice(index, 1);
}

function setNpcPersonality(index: number, value: string) {
  if (!draftCard.value) return;
  draftCard.value.npcs[index].personality = splitCsv(value);
}

function addEvent() {
  if (!draftCard.value) return;
  draftCard.value.events.push({
    id: `evt_${Date.now()}`,
    name: "",
    prompt: "",
  } satisfies CardEvent);
}

function removeEvent(index: number) {
  draftCard.value?.events.splice(index, 1);
}

function addChapter() {
  if (!draftCard.value) return;
  draftCard.value.chapterGoals.push({
    id: `chapter_${Date.now()}`,
    title: "",
    prompt: "",
  } satisfies ChapterGoal);
}

function removeChapter(index: number) {
  draftCard.value?.chapterGoals.splice(index, 1);
}

watch(
  () => props.aiGeneratedCard,
  (card) => {
    if (!card) return;
    selectedCardId.value = "";
    draftCard.value = cloneCard(card);
  },
  { deep: true },
);

watch(
  () => props.worldCards,
  (cards) => {
    if (cards.length === 0) {
      selectedCardId.value = "";
      draftCard.value = null;
      return;
    }
    if (!selectedCardId.value || !cards.some((item) => item.id === selectedCardId.value)) {
      selectCard(cards[0].id);
      return;
    }
    selectCard(selectedCardId.value);
  },
  { immediate: true, deep: true },
);
</script>

<style scoped>
.panel {
  border: 1px solid var(--game-panel-border);
  background: var(--game-panel-bg);
  border-radius: var(--radius-lg);
  padding: 1rem;
}

.panel-title {
  font-size: 1rem;
  font-weight: 600;
  margin-bottom: 0.75rem;
}

.input {
  border: 1px solid var(--game-input-border);
  border-radius: var(--radius-md);
  background: var(--game-input-bg);
  padding: 0.5rem 0.625rem;
}

.btn {
  border: 1px solid var(--game-btn-border);
  border-radius: var(--radius-md);
  padding: 0.4rem 0.7rem;
  font-size: 0.875rem;
  background: var(--game-btn-bg);
  color: var(--game-btn-text);
}

.btn-primary {
  background: var(--game-btn-primary-bg);
  color: var(--game-btn-primary-text);
}

.game-text-muted {
  color: var(--game-text-muted);
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.35rem;
  font-size: 0.875rem;
}

.card-row {
  width: 100%;
  display: flex;
  justify-content: space-between;
  gap: 0.6rem;
  text-align: left;
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: var(--game-input-bg);
  padding: 0.55rem;
}

.card-row-active {
  border-color: var(--game-btn-primary-bg);
  box-shadow: 0 0 0 1px var(--game-btn-primary-bg);
}

.mini-btn {
  border: 1px solid var(--game-btn-border);
  border-radius: 999px;
  padding: 0.1rem 0.45rem;
  font-size: 0.72rem;
  cursor: pointer;
  background: var(--game-btn-bg);
}

.mini-btn-danger {
  color: var(--game-danger-text);
}

.section {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.75rem;
}

.advanced-box {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.45rem 0.6rem;
}

.advanced-summary {
  cursor: pointer;
  font-size: 0.84rem;
}

.section-title {
  font-size: 0.9rem;
  font-weight: 600;
  margin-bottom: 0.5rem;
}

.edit-stack {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.5rem;
  display: grid;
  gap: 0.35rem;
}
</style>
