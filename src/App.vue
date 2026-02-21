<template>
  <div class="app-root min-h-screen p-6">
    <div v-if="view === 'menu'" class="start-screen">
      <div class="start-stack">
        <div class="start-title-wrap">
          <p class="start-kicker">AI ROLEPLAYING GAME</p>
          <h1 class="start-title">RoleClaw</h1>
        </div>
        <GameSettingsMenu
          title="主菜单"
          subtitle="选择你的下一步行动"
          @select="handleMenuSelect"
        />
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
        </div>
        <div class="mt-4 flex gap-2">
          <button class="btn" @click="setView('menu')">返回</button>
          <button class="btn btn-primary" @click="createNewSave">生成世界并开始</button>
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

      <section v-if="view === 'ai-settings'" class="panel max-w-3xl">
        <h2 class="panel-title">AI设置</h2>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="field">
            <span>Provider</span>
            <select v-model="newSave.modelConfig.provider" class="input">
              <option value="openai">openai</option>
              <option value="claude">claude</option>
            </select>
          </label>
          <label class="field">
            <span>模型名</span>
            <input v-model="newSave.modelConfig.model" class="input" />
          </label>
          <label class="field">
            <span>Temperature</span>
            <input v-model.number="newSave.modelConfig.temperature" class="input" type="number" step="0.1" />
          </label>
          <label class="field">
            <span>Max Tokens</span>
            <input v-model.number="newSave.modelConfig.maxTokens" class="input" type="number" />
          </label>
        </div>
        <div class="mt-4 flex gap-2">
          <button class="btn" @click="checkModel">测试模型配置</button>
          <p v-if="modelCheckMsg" class="text-sm text-muted-foreground">{{ modelCheckMsg }}</p>
        </div>
      </section>

      <section v-if="view === 'settings'" class="panel max-w-3xl">
        <h2 class="panel-title">游戏设置</h2>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="field">
            <span>主题</span>
            <select v-model="uiSettings.theme" class="input">
              <option value="default">默认</option>
              <option value="fantasy">沉浸幻想</option>
              <option value="terminal">科幻终端</option>
              <option value="archive">古典档案</option>
            </select>
          </label>
          <label class="field">
            <span>消息速度</span>
            <select v-model="uiSettings.messageSpeed" class="input">
              <option value="slow">慢</option>
              <option value="normal">中</option>
              <option value="fast">快</option>
            </select>
          </label>
        </div>
      </section>

      <section v-if="view === 'cards'">
        <WorldCardManager
          :world-cards="worldCards"
          :card-import-text="cardImportText"
          :card-export-path="cardExportPath"
          @update:card-import-text="cardImportText = $event"
          @update:card-export-path="cardExportPath = $event"
          @import-card="importCardFromText"
          @export-card="exportCard"
        />
      </section>

      <section v-if="view === 'game' && activeSave" class="grid gap-4 lg:grid-cols-[1.2fr_1.8fr_1fr]">
        <div class="panel">
          <h2 class="panel-title">地图</h2>
          <GameMapCanvas
            :snapshot="activeSave.snapshot"
            :reachable-locations="reachableLocations"
            @move="move"
          />
        </div>

        <div class="panel">
          <h2 class="panel-title">叙事与对话</h2>
          <p class="mb-3 text-sm leading-6">{{ narrationText }}</p>

          <div class="space-y-2">
            <button
              v-for="opt in options"
              :key="opt.id"
              class="btn w-full text-left"
              @click="submitOption(opt.id)"
            >
              [{{ opt.kind }}] {{ opt.text }}
            </button>
          </div>

          <div class="mt-3 flex gap-2">
            <input
              v-model="customInput"
              class="input flex-1"
              placeholder="输入你的自定义第四选项..."
            />
            <button class="btn btn-primary" @click="submitCustom">提交</button>
          </div>
        </div>

        <div class="panel">
          <h2 class="panel-title">状态</h2>
          <p class="text-sm">存档：{{ activeSave.meta.name }}</p>
          <p class="text-sm">回合：{{ activeSave.snapshot.turn }}</p>
          <p class="text-sm">角色：{{ activeSave.snapshot.playerRole }}</p>
          <p class="text-sm">模型：{{ activeSave.snapshot.modelConfig.provider }} / {{ activeSave.snapshot.modelConfig.model }}</p>

          <h3 class="mt-4 font-medium">最近变化</h3>
          <ul class="mt-2 list-disc pl-5 text-sm">
            <li v-for="line in stateChanges" :key="line">{{ line }}</li>
          </ul>
        </div>
      </section>

      <div v-if="showInGameMenu && view === 'game'" class="overlay">
        <GameSettingsMenu
          title="游戏菜单"
          subtitle="按 Esc 关闭菜单并继续游戏"
          :show-close="true"
          @select="handleMenuSelect"
          @close="showInGameMenu = false"
        />
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
import { useGameApp } from "@/composables/useGameApp";

const {
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
} = useGameApp();

const showInGameMenu = ref(false);
const uiSettings = ref({
  theme: "default" as "default" | "fantasy" | "terminal" | "archive",
  messageSpeed: "normal",
});

const THEME_KEY = "roleclaw:theme";

function handleMenuSelect(action: "start" | "saves" | "ai" | "cards" | "settings" | "exit") {
  if (action === "exit") {
    exitGame();
    return;
  }

  showInGameMenu.value = false;
  if (action === "start") {
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
  () => uiSettings.value.theme,
  (next) => {
    applyTheme(next);
    localStorage.setItem(THEME_KEY, next);
  }
);

onMounted(async () => {
  const stored = localStorage.getItem(THEME_KEY);
  if (stored === "default" || stored === "fantasy" || stored === "terminal" || stored === "archive") {
    uiSettings.value.theme = stored;
  }
  applyTheme(uiSettings.value.theme);
  await refreshHome();
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
</style>
