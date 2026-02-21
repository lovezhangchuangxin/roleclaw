<template>
  <section class="saves-shell">
    <div class="panel saves-panel">
      <div class="saves-head">
        <div>
          <h2 class="panel-title mb-1">存档管理</h2>
          <p class="text-sm game-text-muted">选择一个存档继续冒险，或清理旧分支。</p>
        </div>
        <div class="saves-summary">
          <div class="saves-summary-item">
            <span>存档数</span>
            <b>{{ saveStats.count }}</b>
          </div>
          <div class="saves-summary-item">
            <span>最高回合</span>
            <b>{{ saveStats.maxTurn }}</b>
          </div>
          <div class="saves-summary-item">
            <span>分叉存档</span>
            <b>{{ saveStats.forks }}</b>
          </div>
        </div>
      </div>

      <div class="saves-toolbar">
        <input
          v-model.trim="saveSearch"
          class="input saves-search"
          placeholder="搜索存档名 / 角色 / ID"
        />
        <Select v-model="saveSort">
          <SelectTrigger class="w-[220px] settings-select-trigger">
            <SelectValue placeholder="排序方式" />
          </SelectTrigger>
          <SelectContent class="settings-select-content">
            <SelectItem value="updated_desc" class="settings-select-item">最近更新优先</SelectItem>
            <SelectItem value="created_desc" class="settings-select-item">最近创建优先</SelectItem>
            <SelectItem value="turn_desc" class="settings-select-item">最高回合优先</SelectItem>
            <SelectItem value="name_asc" class="settings-select-item">名称 A-Z</SelectItem>
          </SelectContent>
        </Select>
        <button class="btn" :disabled="saveStats.forks === 0" @click="clearForkSaves">
          批量清理分叉
        </button>
      </div>

      <div v-if="displayedSaves.length > 0" class="saves-list">
        <article v-for="save in displayedSaves" :key="save.id" class="save-slot">
          <div class="save-slot-main">
            <div class="save-slot-title-row">
              <p class="save-slot-title">{{ save.name }}</p>
              <span v-if="save.parentSaveId" class="save-slot-badge">分叉</span>
            </div>
            <p class="save-slot-subtitle">
              回合 {{ save.currentTurn }} · {{ save.provider }} / {{ save.model }}
            </p>
            <div class="save-slot-meta">
              <span>角色：{{ save.playerRole }}</span>
              <span>更新：{{ formatDateTime(save.updatedAt) }}</span>
              <span>创建：{{ formatDateTime(save.createdAt) }}</span>
            </div>
            <p class="save-slot-id">ID: {{ save.id }}</p>
          </div>
          <div class="save-slot-actions">
            <button class="btn btn-primary" @click="openSave(save.id)">继续</button>
            <button class="btn" @click="removeSave(save.id)">删除</button>
          </div>
        </article>
      </div>

      <div v-else class="saves-empty">
        <p class="text-sm game-text-muted">{{ saves.length === 0 ? "暂无存档。" : "没有匹配的存档。" }}</p>
        <p class="text-xs game-text-muted">
          {{ saves.length === 0 ? "返回主菜单点击“开始游戏”创建第一个存档。" : "尝试更换关键词或排序方式。" }}
        </p>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameAppContext } from "@/composables/useGameAppContext";

const { saves, openSave, removeSave } = useGameAppContext();

const saveSearch = ref("");
const saveSort = ref<"updated_desc" | "created_desc" | "turn_desc" | "name_asc">("updated_desc");

const saveStats = computed(() => ({
  count: saves.value.length,
  maxTurn: saves.value.reduce((max, item) => Math.max(max, item.currentTurn), 0),
  forks: saves.value.filter((item) => Boolean(item.parentSaveId)).length,
}));

const displayedSaves = computed(() => {
  const keyword = saveSearch.value.toLowerCase();
  let rows = saves.value.filter((save) => {
    if (!keyword) {
      return true;
    }
    return (
      save.name.toLowerCase().includes(keyword) ||
      save.id.toLowerCase().includes(keyword) ||
      save.playerRole.toLowerCase().includes(keyword)
    );
  });

  rows = [...rows].sort((a, b) => {
    if (saveSort.value === "created_desc") {
      return b.createdAt.localeCompare(a.createdAt);
    }
    if (saveSort.value === "turn_desc") {
      if (b.currentTurn !== a.currentTurn) {
        return b.currentTurn - a.currentTurn;
      }
      return b.updatedAt.localeCompare(a.updatedAt);
    }
    if (saveSort.value === "name_asc") {
      return a.name.localeCompare(b.name, "zh-Hans-CN");
    }
    return b.updatedAt.localeCompare(a.updatedAt);
  });

  return rows;
});

function formatDateTime(value: string): string {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

async function clearForkSaves() {
  const forkIds = saves.value.filter((item) => item.parentSaveId).map((item) => item.id);
  if (forkIds.length === 0) {
    return;
  }
  const ok = window.confirm(`确认批量删除 ${forkIds.length} 个分叉存档吗？`);
  if (!ok) {
    return;
  }
  for (const id of forkIds) {
    await removeSave(id);
  }
}
</script>
