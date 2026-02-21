<template>
  <div class="start-screen">
    <div class="start-stack">
      <div class="start-title-wrap">
        <p class="start-kicker">AI ROLEPLAYING GAME</p>
        <h1 class="start-title">RoleClaw</h1>
      </div>
      <GameSettingsMenu
        title="主菜单"
        subtitle="选择你的下一步行动"
        :start-label="startLabel"
        :start-disabled="!latestSave"
        :show-new-game="true"
        new-game-label="新游戏"
        @select="handleMenuSelect"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount } from "vue";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import { useGameAppContext } from "@/composables/useGameAppContext";
import { exitApp } from "@/lib/api";

const { defaultModelId, errorMsg, saves, openSave, setView } = useGameAppContext();
const START_BLOCKED_MSG = "未设置默认 AI 模型，请前往 AI 设置。";
const START_BLOCKED_HINT_MS = 3600;
let startBlockedTimer: ReturnType<typeof setTimeout> | null = null;

const latestSave = computed(() => saves.value[0] ?? null);
const startLabel = computed(() =>
  latestSave.value ? `开始游戏 · ${latestSave.value.name}` : "开始游戏 · 暂无存档",
);

async function handleMenuSelect(
  action: "start" | "new" | "saves" | "ai" | "cards" | "settings" | "exit",
) {
  if (action === "exit") {
    void exitGame();
    return;
  }

  if (action === "start") {
    if (!latestSave.value) {
      errorMsg.value = "暂无可继续的存档，请先创建新游戏。";
      return;
    }
    await openSave(latestSave.value.id, true);
    return;
  }

  if (action === "new") {
    if (!defaultModelId.value) {
      showStartBlockedHint();
      setView("ai-settings");
      return;
    }
    setView("new");
    return;
  }

  if (action === "saves") {
    setView("saves");
  } else if (action === "ai") {
    setView("ai-settings");
  } else if (action === "cards") {
    setView("cards");
  } else if (action === "settings") {
    setView("settings");
  }
}

function showStartBlockedHint() {
  errorMsg.value = START_BLOCKED_MSG;
  if (startBlockedTimer) {
    clearTimeout(startBlockedTimer);
  }
  startBlockedTimer = setTimeout(() => {
    if (errorMsg.value === START_BLOCKED_MSG) {
      errorMsg.value = "";
    }
    startBlockedTimer = null;
  }, START_BLOCKED_HINT_MS);
}

async function exitGame() {
  try {
    await exitApp();
    return;
  } catch {
    // Fall back to closing current window if backend command fails.
  }

  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch {
    window.close();
  }
}

onBeforeUnmount(() => {
  if (startBlockedTimer) {
    clearTimeout(startBlockedTimer);
    startBlockedTimer = null;
  }
});
</script>
