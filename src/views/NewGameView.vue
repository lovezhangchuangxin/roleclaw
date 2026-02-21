<template>
  <section class="new-game-shell">
    <div class="new-game-layout">
      <aside class="panel new-game-overview">
        <h2 class="panel-title mb-1">世界卡预览</h2>
        <p class="text-sm game-text-muted">选择世界卡后，这里会显示核心摘要。</p>
        <div v-if="selectedNewGameCard" class="new-game-overview-content">
          <div class="new-game-overview-head">
            <p class="text-base font-semibold">{{ selectedNewGameCard.name }}</p>
            <p class="text-xs game-text-muted">{{ selectedNewGameCard.id }} · schema {{ selectedNewGameCard.schemaVersion }}</p>
          </div>
          <p class="text-sm">{{ selectedNewGameCard.worldbook.overview || "暂无世界简介" }}</p>
          <div class="new-game-metrics">
            <div class="new-game-metric">
              <span class="new-game-metric-label">地点</span>
              <b>{{ selectedNewGameCard.map.nodes.length }}</b>
            </div>
            <div class="new-game-metric">
              <span class="new-game-metric-label">NPC</span>
              <b>{{ selectedNewGameCard.npcs.length }}</b>
            </div>
            <div class="new-game-metric">
              <span class="new-game-metric-label">事件</span>
              <b>{{ selectedNewGameCard.events.length }}</b>
            </div>
            <div class="new-game-metric">
              <span class="new-game-metric-label">章节</span>
              <b>{{ selectedNewGameCard.chapterGoals.length }}</b>
            </div>
          </div>
          <div class="new-game-tag-list">
            <span v-for="conflict in selectedNewGameCard.worldbook.coreConflicts.slice(0, 4)" :key="conflict" class="new-game-tag">
              {{ conflict }}
            </span>
            <span v-if="selectedNewGameCard.worldbook.coreConflicts.length === 0" class="text-xs game-text-muted">未设置核心冲突标签</span>
          </div>
        </div>
        <div v-else class="text-sm game-text-muted mt-3">
          暂无可用世界卡，请先在世界卡管理页创建。
        </div>
      </aside>

      <div class="panel new-game-panel">
        <div class="new-game-head">
          <h2 class="panel-title mb-1">开始游戏</h2>
          <p class="text-sm game-text-muted">创建你的冒险入口：角色身份、世界卡与模型配置。</p>
        </div>
        <p v-if="!defaultModelId" class="new-game-warning text-sm game-text-muted">
          尚未设置默认 AI 模型。请先前往“AI设置”新增模型并设为默认。
        </p>
        <div class="new-game-form grid gap-4 md:grid-cols-2">
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
            <Select v-model="newSave.worldCardId">
              <SelectTrigger class="w-full settings-select-trigger">
                <SelectValue placeholder="选择世界卡" />
              </SelectTrigger>
              <SelectContent class="settings-select-content">
                <SelectItem v-for="card in worldCards" :key="card.id" :value="card.id" class="settings-select-item">
                  {{ card.name }} ({{ card.worldbook.playStyle }})
                </SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label class="field md:col-span-2">
            <span>AI模型</span>
            <Select v-model="newSave.modelProfileId">
              <SelectTrigger class="w-full settings-select-trigger">
                <SelectValue placeholder="选择AI模型" />
              </SelectTrigger>
              <SelectContent class="settings-select-content">
                <SelectItem v-for="model in aiModels" :key="model.id" :value="model.id" class="settings-select-item">
                  {{ model.provider }}/{{ model.model }}
                </SelectItem>
              </SelectContent>
            </Select>
          </label>
        </div>
        <div class="new-game-actions mt-4 flex gap-2">
          <button class="btn" @click="setView('menu')">返回</button>
          <button class="btn btn-primary" :disabled="!defaultModelId" @click="createNewSave">生成世界并开始</button>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameAppContext } from "@/composables/useGameAppContext";

const { newSave, worldCards, aiModels, defaultModelId, setView, createNewSave } = useGameAppContext();

const selectedNewGameCard = computed(() => {
  if (!newSave.value.worldCardId) {
    return worldCards.value[0] ?? null;
  }
  return (
    worldCards.value.find((card) => card.id === newSave.value.worldCardId) ??
    worldCards.value[0] ??
    null
  );
});
</script>
