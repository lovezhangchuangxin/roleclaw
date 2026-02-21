<template>
  <section v-if="activeSave" class="game-play-shell">
    <header class="panel game-play-head">
      <div class="game-play-head-main">
        <p class="game-play-kicker">当前冒险</p>
        <h2 class="game-play-title">{{ activeSave.meta.name }}</h2>
        <p class="game-play-meta">
          回合 {{ activeSave.snapshot.turn }} · 角色 {{ activeSave.snapshot.playerRole }} · 模型
          {{ activeSave.snapshot.modelLabel || activeSave.meta.model }}
        </p>
      </div>
      <div class="game-play-head-actions">
        <button class="btn" @click="openReplayView">打开回放页</button>
        <button class="btn btn-primary" @click="forkActiveSave">从当前回合分叉</button>
      </div>
    </header>

    <div class="game-mobile-tabs" role="tablist" aria-label="游戏面板切换">
      <button class="game-mobile-tab" :class="{ 'game-mobile-tab-active': gameMobileTab === 'map' }"
        @click="gameMobileTab = 'map'">
        地图
      </button>
      <button class="game-mobile-tab" :class="{ 'game-mobile-tab-active': gameMobileTab === 'story' }"
        @click="gameMobileTab = 'story'">
        叙事
      </button>
      <button class="game-mobile-tab" :class="{ 'game-mobile-tab-active': gameMobileTab === 'state' }"
        @click="gameMobileTab = 'state'">
        状态
      </button>
    </div>

    <div class="panel game-panel game-panel-map" :class="{ 'game-panel-mobile-hidden': gameMobileTab !== 'map' }">
      <h2 class="panel-title">地图</h2>
      <div class="game-panel-content">
        <GameMapCanvas :snapshot="activeSave.snapshot" :reachable-locations="reachableLocations" @move="move" />
      </div>
    </div>

    <div class="panel game-panel game-panel-story" :class="{ 'game-panel-mobile-hidden': gameMobileTab !== 'story' }">
      <h2 class="panel-title">叙事与对话</h2>
      <div class="game-panel-content game-story-content">
        <p class="text-sm leading-6 game-story-narration">{{ narrationText }}</p>

        <div class="space-y-2 game-story-options">
          <button v-for="opt in options" :key="opt.id" class="btn w-full text-left" @click="submitOption(opt.id)">
            [{{ opt.kind }}] {{ opt.text }}
          </button>
        </div>

        <div class="game-custom-row flex gap-2">
          <input v-model="customInput" class="input flex-1" placeholder="输入你的自定义第四选项..." />
          <button class="btn btn-primary" @click="submitCustom">提交</button>
        </div>
      </div>
    </div>

    <div class="panel game-panel game-panel-state" :class="{ 'game-panel-mobile-hidden': gameMobileTab !== 'state' }">
      <h2 class="panel-title">状态</h2>
      <div class="game-panel-content game-state-content">
        <section class="game-state-block">
          <h3 class="font-medium">最近变化</h3>
          <ul class="mt-2 list-disc pl-5 text-sm">
            <li v-for="line in stateChanges" :key="line">{{ line }}</li>
            <li v-if="stateChanges.length === 0" class="game-text-muted">暂无变化</li>
          </ul>
        </section>

        <section class="game-state-block">
          <h3 class="font-medium">任务进度</h3>
          <ul class="mt-2 space-y-1 text-xs">
            <li v-for="quest in activeSave.snapshot.quests" :key="quest.id" class="rounded border px-2 py-1">
              {{ quest.title }} · 阶段 {{ quest.stage }} · {{ quest.completed ? "已完成" : "进行中" }}
            </li>
            <li v-if="activeSave.snapshot.quests.length === 0" class="game-text-muted">暂无任务</li>
          </ul>
        </section>

        <section class="game-state-block">
          <h3 class="font-medium">关系矩阵</h3>
          <ul class="mt-2 space-y-1 text-xs">
            <li v-for="entry in relationshipEntries" :key="entry.id" class="rounded border px-2 py-1">
              {{ entry.id }}: {{ entry.value }}
            </li>
            <li v-if="relationshipEntries.length === 0" class="game-text-muted">暂无关系记录</li>
          </ul>
        </section>

        <section v-if="replayPreview.length > 0" class="game-state-block">
          <h3 class="font-medium">回放预览</h3>
          <ul class="mt-2 space-y-1 text-xs">
            <li v-for="item in replayPreview" :key="item.turn" class="rounded border px-2 py-1">
              T{{ item.turn }} · {{ item.output?.stateChangesPreview?.join(" / ") || "无摘要" }}
            </li>
          </ul>
        </section>

        <section v-if="gameSettings.logLevel === 'debug'" class="game-state-block">
          <p class="font-medium mb-1">调试状态</p>
          <pre class="overflow-auto">{{ JSON.stringify(activeSave.snapshot.worldVariables, null, 2) }}</pre>
        </section>
      </div>
    </div>

    <div v-if="showInGameMenu" class="overlay">
      <GameSettingsMenu title="游戏菜单" subtitle="按 Esc 关闭菜单并继续游戏" :show-close="true" @select="handleMenuSelect"
        @close="showInGameMenu = false" />
    </div>
  </section>
  <section v-else class="panel max-w-2xl">
    <h2 class="panel-title">游戏中</h2>
    <p class="text-sm game-text-muted">当前没有激活存档，请先进入存档或新建游戏。</p>
  </section>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import GameMapCanvas from "@/components/game/GameMapCanvas.vue";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import { useGameAppContext } from "@/composables/useGameAppContext";

const {
  activeSave,
  narrationText,
  stateChanges,
  options,
  customInput,
  replayPreview,
  gameSettings,
  defaultModelId,
  errorMsg,
  reachableLocations,
  setView,
  openReplayView,
  submitOption,
  submitCustom,
  move,
  forkActiveSave,
} = useGameAppContext();

const showInGameMenu = ref(false);
const gameMobileTab = ref<"map" | "story" | "state">("story");

const relationshipEntries = computed(() => {
  if (!activeSave.value) {
    return [];
  }
  return Object.entries(activeSave.value.snapshot.relationships).map(([id, value]) => ({
    id,
    value: typeof value === "number" ? value.toFixed(1) : String(value),
  }));
});

watch(
  () => activeSave.value?.meta.id,
  () => {
    gameMobileTab.value = "story";
    showInGameMenu.value = false;
  },
);

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") {
    return;
  }
  showInGameMenu.value = !showInGameMenu.value;
}

function handleMenuSelect(action: "start" | "saves" | "ai" | "cards" | "settings" | "exit") {
  if (action === "exit") {
    void exitGame();
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

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>
