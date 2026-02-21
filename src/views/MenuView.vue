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
import { computed } from "vue";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import { useGameAppContext } from "@/composables/useGameAppContext";
import { exitApp } from "@/lib/api";

const { defaultModelId, errorMsg, saves, openSave, setView } = useGameAppContext();

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
      errorMsg.value = "无法开始游戏：当前没有默认 AI 模型。请先到 AI 设置新增模型并设为默认。";
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
</script>
