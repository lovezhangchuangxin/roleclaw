<template>
  <div class="start-screen">
    <div class="start-stack">
      <div class="start-title-wrap">
        <p class="start-kicker">AI ROLEPLAYING GAME</p>
        <h1 class="start-title">RoleClaw</h1>
      </div>
      <GameSettingsMenu title="主菜单" subtitle="选择你的下一步行动" @select="handleMenuSelect" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import { useGameAppContext } from "@/composables/useGameAppContext";

const { defaultModelId, errorMsg, setView } = useGameAppContext();

function handleMenuSelect(action: "start" | "saves" | "ai" | "cards" | "settings" | "exit") {
  if (action === "exit") {
    void exitGame();
    return;
  }

  if (action === "start") {
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
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch {
    window.close();
  }
}
</script>
