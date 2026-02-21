<template>
  <div class="app-root min-h-screen p-6">
    <div v-if="view === 'menu'" class="start-screen">
      <div class="start-stack">
        <div class="start-title-wrap">
          <p class="start-kicker">AI ROLEPLAYING GAME</p>
          <h1 class="start-title">RoleClaw</h1>
        </div>
        <GameSettingsMenu title="主菜单" subtitle="选择你的下一步行动" @select="handleMenuSelect" />
      </div>
    </div>

    <template v-else>
      <header class="mb-6 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold">RoleClaw AI RPG</h1>
          <p class="text-sm text-muted-foreground">叙事驱动 · 世界卡生成 · 3+1 对话</p>
        </div>
        <div class="flex gap-2">
          <button class="btn" @click="setView('menu')">主菜单</button>
          <button class="btn" @click="refreshHome">刷新</button>
        </div>
      </header>

      <p v-if="errorMsg" class="error-banner mb-4 rounded p-2 text-sm">
        {{ errorMsg }}
      </p>

      <section v-if="view === 'new'" class="panel max-w-3xl">
        <h2 class="panel-title">开始游戏</h2>
        <p v-if="!defaultModelId" class="mb-3 text-sm text-muted-foreground">
          尚未设置默认 AI 模型。请先前往“AI设置”新增模型并设为默认。
        </p>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="field">
            <span>存档名称</span>
            <input v-model="newSave.saveName" class="input" />
          </label>
          <label class="field">
            <span>玩家角色</span>
            <input v-model="newSave.playerRole" class="input" />
          </label>
          <label class="field md:col-span-2">
            <span>世界卡</span>
            <select v-model="newSave.worldCardId" class="input">
              <option v-for="card in worldCards" :key="card.id" :value="card.id">
                {{ card.name }} ({{ card.genre }} / {{ card.tone }})
              </option>
            </select>
          </label>
          <label class="field md:col-span-2">
            <span>AI模型</span>
            <select v-model="newSave.modelProfileId" class="input">
              <option v-for="model in aiModels" :key="model.id" :value="model.id">
                {{ model.provider }}/{{ model.model }}
              </option>
            </select>
          </label>
        </div>
        <div class="mt-4 flex gap-2">
          <button class="btn" @click="setView('menu')">返回</button>
          <button class="btn btn-primary" :disabled="!defaultModelId" @click="createNewSave">生成世界并开始</button>
        </div>
      </section>

      <section v-if="view === 'saves'" class="panel max-w-4xl">
        <h2 class="panel-title">存档管理</h2>
        <div class="space-y-2">
          <div v-for="save in saves" :key="save.id" class="rounded border p-2">
            <div class="flex items-center justify-between gap-2">
              <div>
                <p class="font-medium">{{ save.name }}</p>
                <p class="text-xs text-muted-foreground">
                  回合 {{ save.currentTurn }} · {{ save.provider }} / {{ save.model }}
                </p>
              </div>
              <div class="flex gap-2">
                <button class="btn" @click="openSave(save.id)">载入</button>
                <button class="btn" @click="removeSave(save.id)">删除</button>
              </div>
            </div>
          </div>
          <p v-if="saves.length === 0" class="text-sm text-muted-foreground">暂无存档。</p>
        </div>
      </section>

      <section v-if="view === 'ai-settings'" class="panel w-full ai-settings-panel">
        <h2 class="panel-title">AI设置</h2>
        <p class="text-sm text-muted-foreground">当前协议类型仅支持 OpenAI Compatible。</p>

        <div class="ai-layout mt-4">
          <div class="ai-block">
            <div class="ai-block-head">
              <h3 class="ai-block-title">{{ editingModelId ? "编辑模型" : "新建模型" }}</h3>
              <p class="text-xs text-muted-foreground">左侧维护模型连接信息，保存后立即生效。</p>
            </div>
            <div class="grid gap-4 md:grid-cols-2">
              <label class="field">
                <span>协议类型</span>
                <input class="input" value="OpenAI Compatible" disabled />
              </label>
              <label class="field">
                <span>Provider 名称</span>
                <input v-model="aiDraft.provider" class="input" placeholder="例如 OpenAI / DeepSeek" />
              </label>
              <label class="field md:col-span-2">
                <span>Base URL</span>
                <input v-model="aiDraft.baseUrl" class="input" placeholder="例如 https://api.openai.com/v1" />
              </label>
              <label class="field md:col-span-2">
                <span>API Key</span>
                <input v-model="aiDraft.apiKey" class="input" type="password" placeholder="sk-..." />
              </label>
              <label class="field md:col-span-2">
                <span>模型名</span>
                <input v-model="aiDraft.model" class="input" />
              </label>
              <label class="field">
                <span>Temperature</span>
                <input v-model.number="aiDraft.temperature" class="input" type="number" step="0.1" />
              </label>
              <label class="field">
                <span>Timeout(ms)</span>
                <input v-model.number="aiDraft.timeoutMs" class="input" type="number" step="100" />
              </label>
            </div>

            <div class="mt-4 flex flex-wrap gap-2">
              <button class="btn" @click="testAiDraft">连通性测试</button>
              <button class="btn btn-primary" @click="saveAiModel">保存模型</button>
              <button class="btn" @click="resetAiDraft">重置为新建</button>
            </div>

            <div v-if="modelCheckMsg" class="mt-3 model-check-msg"
              :class="modelCheckOk ? 'model-check-success' : 'model-check-fail'">
              <p class="text-sm">{{ modelCheckMsg }}</p>
              <button v-if="modelCheckOk === false" class="btn model-check-copy-btn" @click="copyModelCheckError">
                {{ copiedModelCheck ? "已复制" : "复制失败原因" }}
              </button>
            </div>
          </div>

          <div class="ai-block">
            <div class="ai-block-head">
              <h3 class="ai-block-title">已配置模型</h3>
              <p class="text-xs text-muted-foreground">共 {{ aiModels.length }} 个 · 默认 {{ defaultModelId || "未设置" }}</p>
            </div>
            <div class="ai-list">
              <div v-for="model in aiModels" :key="model.id" class="ai-list-item"
                :class="{ 'ai-list-item-active': editingModelId === model.id }" @click="selectAiModel(model.id)">
                <div class="ai-list-main">
                  <p class="font-medium">{{ model.provider }}/{{ model.model }}</p>
                  <p class="text-xs text-muted-foreground">{{ model.providerType }}</p>
                  <p class="text-xs text-muted-foreground truncate">{{ model.baseUrl }}</p>
                </div>
                <div class="ai-list-actions">
                  <span v-if="defaultModelId === model.id" class="ai-default-badge">默认</span>
                  <button class="ai-action-btn" @click.stop="markDefaultAiModel(model.id)">设为默认</button>
                  <button class="ai-action-btn ai-action-btn-danger"
                    @click.stop="confirmRemoveAiModel(model.id)">删除</button>
                </div>
              </div>
              <div v-if="aiModels.length === 0" class="ai-empty">
                <p class="text-sm text-muted-foreground">当前还没有配置任何 AI 模型。</p>
                <p class="text-xs text-muted-foreground">请在左侧填写参数并点击“保存模型”。</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section v-if="view === 'settings'" class="settings-shell">
        <div class="panel settings-panel">
          <div class="settings-head">
            <h2 class="panel-title mb-0">游戏设置</h2>
            <p class="text-sm text-muted-foreground">主题切换会即时生效，当前存档与全局设置会自动同步。</p>
          </div>

          <div class="settings-grid">
            <label class="field">
              <span>主题</span>
              <Select v-model="gameSettings.theme">
                <SelectTrigger class="w-full settings-select-trigger">
                  <SelectValue placeholder="选择主题" />
                </SelectTrigger>
                <SelectContent class="settings-select-content">
                  <SelectItem v-for="theme in themeOptions" :key="theme.value" :value="theme.value"
                    class="settings-select-item">
                    <div class="settings-select-theme" :data-game-theme="theme.value">
                      <span class="settings-select-theme-name">{{ theme.label }}</span>
                      <span class="settings-select-swatch-row">
                        <span class="settings-select-swatch settings-select-swatch-panel" />
                        <span class="settings-select-swatch settings-select-swatch-primary" />
                        <span class="settings-select-swatch settings-select-swatch-accent" />
                      </span>
                    </div>
                  </SelectItem>
                </SelectContent>
              </Select>
            </label>

            <label class="field">
              <span>消息速度</span>
              <Select v-model="gameSettings.messageSpeed">
                <SelectTrigger class="w-full settings-select-trigger">
                  <SelectValue placeholder="选择速度" />
                </SelectTrigger>
                <SelectContent class="settings-select-content">
                  <SelectItem v-for="speed in speedOptions" :key="speed.value" :value="speed.value"
                    class="settings-select-item">
                    {{ speed.label }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </label>
          </div>

          <div class="theme-preview-grid">
            <button v-for="theme in themeOptions" :key="theme.value" type="button" class="theme-preview-card"
              :data-game-theme="theme.value"
              :class="{ 'theme-preview-card-active': gameSettings.theme === theme.value }"
              @click="gameSettings.theme = theme.value">
              <div class="theme-preview-surface">
                <div class="theme-preview-header">
                  <span class="theme-preview-dot" />
                  <span class="theme-preview-dot" />
                  <span class="theme-preview-dot" />
                </div>
                <div class="theme-preview-lines">
                  <span class="theme-preview-line" />
                  <span class="theme-preview-line theme-preview-line-short" />
                </div>
                <div class="theme-preview-cta">Action</div>
              </div>
              <div class="theme-preview-meta">
                <p class="theme-preview-name">{{ theme.label }}</p>
                <p class="theme-preview-desc">{{ theme.description }}</p>
              </div>
            </button>
          </div>

          <div class="mt-5 flex justify-end">
            <button class="btn btn-primary" @click="saveGlobalGameData">保存游戏设置</button>
          </div>
        </div>
      </section>

      <section v-if="view === 'cards'">
        <WorldCardManager :world-cards="worldCards" :card-import-text="cardImportText"
          :card-export-path="cardExportPath" @update:card-import-text="cardImportText = $event"
          @update:card-export-path="cardExportPath = $event" @import-card="importCardFromText"
          @export-card="exportCard" />
      </section>

      <section v-if="view === 'game' && activeSave" class="grid gap-4 lg:grid-cols-[1.2fr_1.8fr_1fr]">
        <div class="panel">
          <h2 class="panel-title">地图</h2>
          <GameMapCanvas :snapshot="activeSave.snapshot" :reachable-locations="reachableLocations" @move="move" />
        </div>

        <div class="panel">
          <h2 class="panel-title">叙事与对话</h2>
          <p class="mb-3 text-sm leading-6">{{ narrationText }}</p>

          <div class="space-y-2">
            <button v-for="opt in options" :key="opt.id" class="btn w-full text-left" @click="submitOption(opt.id)">
              [{{ opt.kind }}] {{ opt.text }}
            </button>
          </div>

          <div class="mt-3 flex gap-2">
            <input v-model="customInput" class="input flex-1" placeholder="输入你的自定义第四选项..." />
            <button class="btn btn-primary" @click="submitCustom">提交</button>
          </div>
        </div>

        <div class="panel">
          <h2 class="panel-title">状态</h2>
          <p class="text-sm">存档：{{ activeSave.meta.name }}</p>
          <p class="text-sm">回合：{{ activeSave.snapshot.turn }}</p>
          <p class="text-sm">角色：{{ activeSave.snapshot.playerRole }}</p>
          <p class="text-sm">模型：{{ activeSave.snapshot.modelLabel || activeSave.meta.model }}</p>

          <h3 class="mt-4 font-medium">最近变化</h3>
          <ul class="mt-2 list-disc pl-5 text-sm">
            <li v-for="line in stateChanges" :key="line">{{ line }}</li>
          </ul>
        </div>
      </section>

      <div v-if="showInGameMenu && view === 'game'" class="overlay">
        <GameSettingsMenu title="游戏菜单" subtitle="按 Esc 关闭菜单并继续游戏" :show-close="true" @select="handleMenuSelect"
          @close="showInGameMenu = false" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import GameMapCanvas from "@/components/game/GameMapCanvas.vue";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import WorldCardManager from "@/components/game/WorldCardManager.vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameApp } from "@/composables/useGameApp";

const {
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
} = useGameApp();

const showInGameMenu = ref(false);
const copiedModelCheck = ref(false);
const themeOptions = [
  { value: "default", label: "默认", description: "清晰平衡，通用阅读" },
  { value: "fantasy", label: "沉浸幻想", description: "暖色羊皮卷氛围" },
  { value: "terminal", label: "科幻终端", description: "高对比霓虹控制台" },
  { value: "archive", label: "古典档案", description: "旧纸档案式质感" },
] as const;
const speedOptions = [
  { value: "slow", label: "慢" },
  { value: "normal", label: "中" },
  { value: "fast", label: "快" },
] as const;

function handleMenuSelect(action: "start" | "saves" | "ai" | "cards" | "settings" | "exit") {
  if (action === "exit") {
    exitGame();
    return;
  }

  showInGameMenu.value = false;
  if (action === "start") {
    if (!defaultModelId.value) {
      errorMsg.value = "无法开始游戏：当前没有默认 AI 模型。请先到 AI 设置新增模型并设为默认。";
      setView("ai-settings");
      return;
    }
    setView("new");
  } else if (action === "saves") {
    setView("saves");
  } else if (action === "ai") {
    setView("ai-settings");
  } else if (action === "cards") {
    setView("cards");
  } else if (action === "settings") {
    setView("settings");
  }
}

async function exitGame() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch {
    window.close();
  }
}

async function confirmRemoveAiModel(modelId: string) {
  const ok = window.confirm("确认删除该 AI 模型吗？");
  if (!ok) {
    return;
  }
  await removeAiModel(modelId);
}

async function copyModelCheckError() {
  if (!modelCheckMsg.value || modelCheckOk.value !== false) {
    return;
  }
  try {
    await navigator.clipboard.writeText(modelCheckMsg.value);
    copiedModelCheck.value = true;
    setTimeout(() => {
      copiedModelCheck.value = false;
    }, 1200);
  } catch {
    copiedModelCheck.value = false;
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") {
    return;
  }
  if (view.value !== "game") {
    return;
  }
  showInGameMenu.value = !showInGameMenu.value;
}

function applyTheme(theme: string) {
  document.documentElement.setAttribute("data-game-theme", theme);
}

watch(
  () => gameSettings.value.theme,
  async (next, prev) => {
    applyTheme(next);
    if (prev && prev !== next) {
      await saveGlobalGameData();
    }
  }
);

watch(modelCheckMsg, () => {
  copiedModelCheck.value = false;
});

onMounted(async () => {
  await refreshHome();
  applyTheme(gameSettings.value.theme);
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<style scoped>
.app-root {
  background:
    radial-gradient(circle at 10% 0%, var(--game-bg-layer-1), transparent 28%),
    radial-gradient(circle at 90% 90%, var(--game-bg-layer-2), transparent 30%);
}

.start-screen {
  min-height: calc(100vh - 3rem);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.start-screen::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  opacity: 0.45;
  background-image:
    linear-gradient(to right, var(--game-panel-border) 1px, transparent 1px),
    linear-gradient(to bottom, var(--game-panel-border) 1px, transparent 1px);
  background-size: 28px 28px;
  mask-image: radial-gradient(circle at 50% 40%, black 30%, transparent 90%);
}

.start-stack {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  animation: menu-reveal 520ms ease-out 120ms both;
}

.start-title-wrap {
  text-align: center;
  animation: title-reveal 700ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

.start-title-wrap::after {
  content: "";
  display: block;
  width: clamp(140px, 28vw, 260px);
  height: 16px;
  margin: 0.5rem auto 0;
  border-radius: 999px;
  background: radial-gradient(circle, var(--game-bg-layer-1), transparent 70%);
  filter: blur(6px);
  opacity: 0.8;
  animation: title-glow 2.6s ease-in-out 1s infinite;
}

.start-kicker {
  font-size: 0.72rem;
  letter-spacing: 0.22em;
  color: var(--game-menu-kicker);
}

.start-title {
  font-size: clamp(2.2rem, 5vw, 3.4rem);
  line-height: 1.05;
  font-weight: 800;
  letter-spacing: 0.04em;
}

@keyframes title-reveal {
  from {
    opacity: 0;
    transform: translateY(14px) scale(0.985);
    filter: blur(2px);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
    filter: blur(0);
  }
}

@keyframes menu-reveal {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes title-glow {

  0%,
  100% {
    opacity: 0.65;
    transform: scaleX(0.96);
  }

  50% {
    opacity: 1;
    transform: scaleX(1.03);
  }
}

.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--game-overlay-bg);
  backdrop-filter: blur(2px);
  z-index: 40;
}

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

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  font-size: 0.875rem;
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

.btn:hover {
  background: var(--game-btn-hover-bg);
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--game-btn-primary-bg);
  color: var(--game-btn-primary-text);
}

.btn-primary:hover {
  background: var(--game-btn-primary-hover-bg);
}

.error-banner {
  border: 1px solid var(--game-error-border);
  background: var(--game-error-bg);
  color: var(--game-error-text);
}

.ai-layout {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(420px, 1.7fr) minmax(360px, 1fr);
  flex: 1;
  min-height: 0;
}

@media (max-width: 960px) {
  .ai-layout {
    grid-template-columns: 1fr;
  }
}

.ai-block {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-lg);
  padding: 1rem;
  background: color-mix(in oklab, var(--game-panel-bg) 92%, white 8%);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.ai-settings-panel {
  min-height: calc(100dvh - 11rem);
  display: flex;
  flex-direction: column;
}

.ai-block-head {
  margin-bottom: 0.75rem;
}

.ai-block-title {
  font-weight: 600;
}

.model-check-msg {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.7rem 0.75rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.model-check-success {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 55%, var(--game-panel-border) 45%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 14%, transparent);
}

.model-check-fail {
  border-color: var(--game-error-border);
  background: color-mix(in oklab, var(--game-error-bg) 75%, transparent);
}

.model-check-copy-btn {
  white-space: nowrap;
  padding: 0.2rem 0.56rem;
  font-size: 0.76rem;
  border-color: var(--game-panel-border);
  background: color-mix(in oklab, var(--game-panel-bg) 92%, transparent);
  color: var(--game-btn-text);
}

.model-check-copy-btn:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 35%, var(--game-panel-border) 65%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
}

.ai-list {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  flex: 1;
  min-height: 220px;
  overflow: auto;
  padding-right: 0.2rem;
}

.ai-list-item {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: color-mix(in oklab, var(--game-panel-bg) 96%, transparent);
  padding: 0.68rem 0.72rem;
  width: 100%;
  text-align: left;
  display: flex;
  justify-content: space-between;
  gap: 0.8rem;
  transition: border-color 140ms ease, background 140ms ease;
}

.ai-list-item:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 26%, var(--game-panel-border) 74%);
  background: color-mix(in oklab, var(--game-panel-bg) 90%, white 10%);
}

.ai-list-item-active {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 48%, var(--game-panel-border) 52%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
}

.ai-list-main {
  min-width: 0;
}

.ai-list-actions {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-content: flex-start;
  min-width: 182px;
}

.ai-action-btn {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: color-mix(in oklab, var(--game-panel-bg) 94%, transparent);
  color: var(--game-btn-text);
  font-size: 0.76rem;
  line-height: 1;
  padding: 0.34rem 0.52rem;
  transition: border-color 120ms ease, background 120ms ease;
}

.ai-action-btn:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 38%, var(--game-panel-border) 62%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 8%, var(--game-panel-bg) 92%);
}

.ai-action-btn-danger:hover {
  border-color: color-mix(in oklab, var(--game-error-border) 60%, var(--game-panel-border) 40%);
  background: color-mix(in oklab, var(--game-error-bg) 25%, var(--game-panel-bg) 75%);
}

.ai-default-badge {
  border: 1px solid color-mix(in oklab, var(--game-btn-primary-bg) 45%, var(--game-panel-border) 55%);
  color: color-mix(in oklab, var(--game-btn-text) 82%, var(--game-btn-primary-text) 18%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
  border-radius: 999px;
  padding: 0.1rem 0.46rem;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.ai-empty {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.8rem;
  background: color-mix(in oklab, var(--game-panel-bg) 94%, white 6%);
}

.settings-shell {
  min-height: calc(100dvh - 11rem);
  display: flex;
  align-items: center;
  justify-content: center;
}

.settings-panel {
  width: min(980px, 100%);
  padding: 1.1rem;
}

.settings-head {
  margin-bottom: 1rem;
}

.settings-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.settings-select-trigger {
  border-color: var(--game-input-border);
  background: var(--game-input-bg);
}

.settings-select-content {
  border-color: var(--game-panel-border);
  background: var(--game-panel-bg);
  color: var(--game-btn-text);
}

.settings-select-item {
  color: var(--game-btn-text);
}

.settings-select-theme {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.settings-select-theme-name {
  font-size: 0.84rem;
}

.settings-select-swatch-row {
  display: inline-flex;
  gap: 0.25rem;
}

.settings-select-swatch {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 999px;
  border: 1px solid color-mix(in oklab, var(--game-panel-border) 78%, transparent);
}

.settings-select-swatch-panel {
  background: var(--game-panel-bg);
}

.settings-select-swatch-primary {
  background: var(--game-btn-primary-bg);
}

.settings-select-swatch-accent {
  background: var(--game-bg-layer-1);
}

.theme-preview-grid {
  margin-top: 1rem;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
}

.theme-preview-card {
  border: 1px solid var(--game-panel-border);
  border-radius: calc(var(--radius-lg) + 2px);
  padding: 0.68rem;
  background: color-mix(in oklab, var(--game-panel-bg) 96%, transparent);
  text-align: left;
  transition: border-color 140ms ease, transform 140ms ease, box-shadow 140ms ease;
}

.theme-preview-card:hover {
  transform: translateY(-2px);
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 46%, var(--game-panel-border) 54%);
  box-shadow: 0 10px 22px color-mix(in oklab, var(--game-overlay-bg) 25%, transparent);
}

.theme-preview-card-active {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 62%, var(--game-panel-border) 38%);
  box-shadow: 0 0 0 2px color-mix(in oklab, var(--game-btn-primary-bg) 24%, transparent);
}

.theme-preview-surface {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background:
    linear-gradient(180deg, color-mix(in oklab, var(--game-panel-bg) 88%, transparent), color-mix(in oklab, var(--game-bg-layer-1) 35%, transparent));
  padding: 0.5rem;
}

.theme-preview-header {
  display: flex;
  gap: 0.2rem;
  margin-bottom: 0.45rem;
}

.theme-preview-dot {
  width: 0.4rem;
  height: 0.4rem;
  border-radius: 999px;
  background: var(--game-btn-border);
}

.theme-preview-lines {
  display: grid;
  gap: 0.25rem;
}

.theme-preview-line {
  height: 0.23rem;
  border-radius: 999px;
  background: color-mix(in oklab, var(--game-btn-text) 70%, transparent);
  opacity: 0.45;
}

.theme-preview-line-short {
  width: 68%;
}

.theme-preview-cta {
  margin-top: 0.45rem;
  border-radius: 999px;
  background: var(--game-btn-primary-bg);
  color: var(--game-btn-primary-text);
  font-size: 0.56rem;
  width: fit-content;
  padding: 0.13rem 0.42rem;
  letter-spacing: 0.02em;
}

.theme-preview-meta {
  margin-top: 0.48rem;
}

.theme-preview-name {
  font-size: 0.84rem;
  font-weight: 600;
  color: var(--game-btn-text);
}

.theme-preview-desc {
  margin-top: 0.08rem;
  font-size: 0.72rem;
  color: color-mix(in oklab, var(--game-btn-text) 72%, transparent);
}

@media (max-width: 900px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }

  .theme-preview-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
