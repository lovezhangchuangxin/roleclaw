<template>
  <div class="app-root min-h-screen p-6" :style="appStyleVars">
    <header v-if="view !== 'menu'" class="mb-6 flex items-center justify-between">
      <div>
        <h1 class="text-2xl font-semibold">RoleClaw AI RPG</h1>
        <p class="text-sm game-text-muted">叙事驱动 · 世界卡生成 · 3+1 对话</p>
      </div>
      <div class="flex gap-2">
        <button v-if="activeSave" class="btn" @click="openReplayView">回放/分叉</button>
        <button class="btn" @click="setView('menu')">主菜单</button>
        <button class="btn" @click="refreshHome">刷新</button>
      </div>
    </header>

    <p v-if="errorMsg" class="error-banner mb-4 rounded p-2 text-sm">
      {{ errorMsg }}
    </p>

    <RouterView />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, provide, watch } from "vue";
import { RouterView, useRoute, useRouter } from "vue-router";
import { gameAppContextKey } from "@/composables/useGameAppContext";
import { useGameApp, type ViewMode } from "@/composables/useGameApp";
import { routeNameByView } from "@/router";

const gameApp = useGameApp();
provide(gameAppContextKey, gameApp);

const {
  view,
  errorMsg,
  gameSettings,
  activeSave,
  refreshHome,
  openReplayView,
  setView,
} = gameApp;

const router = useRouter();
const route = useRoute();

const appStyleVars = computed<Record<string, string>>(() => ({
  "--game-font-scale": String(gameSettings.value.fontScale ?? 1),
  "--game-ui-zoom": String(gameSettings.value.uiZoom ?? 1),
}));

const viewByRouteName: Record<string, ViewMode> = {
  [routeNameByView.menu]: "menu",
  [routeNameByView.new]: "new",
  [routeNameByView.game]: "game",
  [routeNameByView.cards]: "cards",
  [routeNameByView.saves]: "saves",
  [routeNameByView["ai-settings"]]: "ai-settings",
  [routeNameByView.settings]: "settings",
  [routeNameByView.replay]: "replay",
};

function applyTheme(theme: string) {
  document.documentElement.setAttribute("data-game-theme", theme);
}

watch(
  () => gameSettings.value.theme,
  async (next, prev) => {
    applyTheme(next);
    if (prev && prev !== next) {
      await gameApp.saveGlobalGameData();
    }
  },
);

watch(
  () => route.name,
  (next) => {
    if (!next) {
      return;
    }
    const mapped = viewByRouteName[String(next)];
    if (mapped && mapped !== view.value) {
      setView(mapped);
    }
  },
  { immediate: true },
);

watch(
  () => view.value,
  (next) => {
    const nextRouteName = routeNameByView[next];
    if (route.name !== nextRouteName) {
      void router.push({ name: nextRouteName });
    }
  },
);

onMounted(async () => {
  await refreshHome();
  applyTheme(gameSettings.value.theme);
});
</script>
