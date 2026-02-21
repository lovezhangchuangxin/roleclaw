import { createRouter, createWebHashHistory } from "vue-router";

import AiSettingsView from "@/views/AiSettingsView.vue";
import GamePlayView from "@/views/GamePlayView.vue";
import GameSettingsView from "@/views/GameSettingsView.vue";
import MenuView from "@/views/MenuView.vue";
import NewGameView from "@/views/NewGameView.vue";
import ReplayView from "@/views/ReplayView.vue";
import SavesView from "@/views/SavesView.vue";
import WorldCardsView from "@/views/WorldCardsView.vue";

export const routeNameByView = {
  menu: "menu",
  new: "new",
  game: "game",
  cards: "cards",
  saves: "saves",
  "ai-settings": "ai-settings",
  settings: "settings",
  replay: "replay",
} as const;

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: "/", name: routeNameByView.menu, component: MenuView },
    { path: "/new", name: routeNameByView.new, component: NewGameView },
    { path: "/game", name: routeNameByView.game, component: GamePlayView },
    { path: "/cards", name: routeNameByView.cards, component: WorldCardsView },
    { path: "/saves", name: routeNameByView.saves, component: SavesView },
    { path: "/ai-settings", name: routeNameByView["ai-settings"], component: AiSettingsView },
    { path: "/settings", name: routeNameByView.settings, component: GameSettingsView },
    { path: "/replay", name: routeNameByView.replay, component: ReplayView },
  ],
});

export default router;
