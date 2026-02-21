<template>
  <div class="panel max-w-5xl">
    <h2 class="panel-title">世界卡管理</h2>
    <p class="mb-4 text-sm text-muted-foreground">支持列表查看、JSON 导入、指定路径导出。</p>

    <div class="mb-4 grid gap-4 lg:grid-cols-[1.2fr_1fr]">
      <div>
        <h3 class="mb-2 font-medium">导入（粘贴 JSON）</h3>
        <textarea
          :value="cardImportText"
          class="input h-44 w-full"
          placeholder='{"id":"my_card","name":"新世界", ...}'
          @input="$emit('update:cardImportText', ($event.target as HTMLTextAreaElement).value)"
        />
        <button class="btn btn-primary mt-2" @click="$emit('importCard')">导入世界卡</button>
      </div>

      <div>
        <h3 class="mb-2 font-medium">导出目标路径</h3>
        <input
          :value="cardExportPath"
          class="input w-full"
          placeholder="/tmp/world-card.json"
          @input="$emit('update:cardExportPath', ($event.target as HTMLInputElement).value)"
        />
        <p class="mt-2 text-xs text-muted-foreground">点击列表中的“导出”会输出到该路径。</p>
      </div>
    </div>

    <h3 class="mb-2 font-medium">已有世界卡</h3>
    <div class="space-y-2">
      <div v-for="card in worldCards" :key="card.id" class="rounded border p-3">
        <div class="flex items-center justify-between gap-3">
          <div>
            <p class="font-medium">{{ card.name }}</p>
            <p class="text-xs text-muted-foreground">
              {{ card.id }} · {{ card.genre }} / {{ card.tone }} · v{{ card.schemaVersion }}
            </p>
          </div>
          <button class="btn" @click="$emit('exportCard', card.id)">导出</button>
        </div>
      </div>
      <p v-if="worldCards.length === 0" class="text-sm text-muted-foreground">暂无世界卡。</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { WorldCard } from "@/types";

defineProps<{
  worldCards: WorldCard[];
  cardImportText: string;
  cardExportPath: string;
}>();

defineEmits<{
  importCard: [];
  exportCard: [cardId: string];
  "update:cardImportText": [value: string];
  "update:cardExportPath": [value: string];
}>();
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
</style>
