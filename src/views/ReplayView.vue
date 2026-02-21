<template>
  <section v-if="activeSave" class="panel w-full">
    <div class="mb-3 flex items-center justify-between">
      <div>
        <h2 class="panel-title mb-1">回放与分叉</h2>
        <p class="text-xs game-text-muted">存档 {{ activeSave.meta.name }} · 当前回合 {{ activeSave.snapshot.turn }}</p>
      </div>
      <div class="flex gap-2">
        <button class="btn" @click="setView('game')">返回游戏</button>
        <button class="btn" @click="refreshReplayData">刷新回放</button>
        <button class="btn" :disabled="!replayHasMore || replayLoading" @click="loadReplayTimeline(false)">
          {{ replayLoading ? "加载中..." : replayHasMore ? "加载更多历史" : "已到底" }}
        </button>
      </div>
    </div>

    <div v-if="replayResult" class="mb-3 rounded border p-2 text-xs">
      <p>日志末回合：{{ replayResult.consistency.logLastTurn }} · 快照回合：{{ replayResult.consistency.snapshotTurn }}</p>
      <p>
        一致性：
        <b :class="replayResult.consistency.matchesSnapshot ? 'text-emerald-600' : 'text-red-600'">
          {{ replayResult.consistency.matchesSnapshot ? "通过" : "不一致" }}
        </b>
        · 单调递增：{{ replayResult.consistency.isMonotonic ? "是" : "否" }}
      </p>
      <ul v-if="replayResult.consistency.warnings.length" class="mt-1 list-disc pl-5">
        <li v-for="warning in replayResult.consistency.warnings" :key="warning">{{ warning }}</li>
      </ul>
    </div>

    <div class="grid gap-3 lg:grid-cols-[1fr_1.5fr]">
      <div class="max-h-[440px] overflow-auto space-y-2 pr-1">
        <button v-for="item in replayPreview" :key="item.turn" class="w-full rounded border p-2 text-left"
          :class="replaySelectedTurn === item.turn ? 'replay-item-active' : ''" @click="replaySelectedTurn = item.turn">
          <p class="text-xs font-medium">T{{ item.turn }}</p>
          <p class="text-xs game-text-muted truncate">{{ item.output?.stateChangesPreview?.join(" / ") || "无摘要" }}</p>
        </button>
      </div>

      <div class="rounded border p-3" v-if="selectedReplayItem">
        <h3 class="font-medium">回合 T{{ selectedReplayItem.turn }}</h3>
        <p class="mt-2 text-xs"><b>输入：</b>{{ selectedReplayItem.input.customText || selectedReplayItem.input.optionId || "无" }}</p>
        <p class="mt-2 text-xs"><b>叙事：</b>{{ selectedReplayItem.output.narration }}</p>
        <p class="mt-2 text-xs"><b>状态变化：</b>{{ selectedReplayItem.output.stateChangesPreview.join(" / ") || "无" }}</p>
        <p class="mt-2 text-xs"><b>事件：</b>{{ selectedReplayItem.triggeredEventIds.join(", ") || "无" }}</p>
        <div class="mt-3 flex gap-2">
          <button class="btn btn-primary" @click="forkAtTurn(selectedReplayItem.turn)">从该回合分叉</button>
        </div>
      </div>
    </div>
  </section>
  <section v-else class="panel w-full">
    <h2 class="panel-title">回放与分叉</h2>
    <p class="text-sm game-text-muted">请先加载存档，再进入回放页。</p>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useGameAppContext } from "@/composables/useGameAppContext";

const {
  activeSave,
  replayPreview,
  replayResult,
  replaySelectedTurn,
  replayHasMore,
  replayLoading,
  setView,
  loadReplayTimeline,
  refreshReplayData,
  forkAtTurn,
} = useGameAppContext();

const selectedReplayItem = computed(() => {
  if (replaySelectedTurn.value == null) {
    return replayPreview.value[replayPreview.value.length - 1] ?? null;
  }
  return replayPreview.value.find((item) => item.turn === replaySelectedTurn.value) ?? null;
});
</script>
