<template>
  <div>
    <canvas ref="canvasRef" width="560" height="360" class="game-canvas w-full rounded border"></canvas>
    <p class="mt-2 text-xs text-muted-foreground">当前位置：{{ snapshot.currentLocationId }}</p>
    <div class="mt-2 flex flex-wrap gap-2">
      <button v-for="loc in reachableLocations" :key="loc.id" class="btn" @click="$emit('move', loc.id)">
        前往 {{ loc.name }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from "vue";
import type { LocationNode, SaveSnapshot } from "@/types";

const props = defineProps<{
  snapshot: SaveSnapshot;
  reachableLocations: LocationNode[];
}>();

defineEmits<{
  move: [locationId: string];
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);

function themeColor(name: string): string {
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value;
}

function drawMap() {
  if (!canvasRef.value) {
    return;
  }
  const snapshot = props.snapshot;
  const canvas = canvasRef.value;
  const ctx = canvas.getContext("2d");
  if (!ctx) {
    return;
  }

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = themeColor("--game-canvas-bg");
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  const locMap = new Map(snapshot.locations.map((l) => [l.id, l]));

  for (const edge of snapshot.paths) {
    const from = locMap.get(edge.from);
    const to = locMap.get(edge.to);
    if (!from || !to) {
      continue;
    }
    ctx.strokeStyle = edge.locked ? themeColor("--game-canvas-edge-locked") : themeColor("--game-canvas-edge");
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(from.x, from.y);
    ctx.lineTo(to.x, to.y);
    ctx.stroke();
  }

  for (const loc of snapshot.locations) {
    const isCurrent = loc.id === snapshot.currentLocationId;
    ctx.beginPath();
    ctx.arc(loc.x, loc.y, isCurrent ? 11 : 8, 0, Math.PI * 2);
    ctx.fillStyle = isCurrent ? themeColor("--game-canvas-node-active") : themeColor("--game-canvas-node");
    ctx.fill();

    ctx.fillStyle = themeColor("--game-canvas-label");
    ctx.font = "12px sans-serif";
    ctx.fillText(loc.name, loc.x + 12, loc.y + 4);
  }
}

watch(
  () => props.snapshot,
  async () => {
    await nextTick();
    drawMap();
  },
  { deep: true }
);

onMounted(drawMap);
</script>

<style scoped>
.game-canvas {
  background: var(--game-canvas-bg);
  border-color: var(--game-panel-border);
}

.btn {
  border: 1px solid var(--game-btn-border);
  border-radius: var(--radius-md);
  padding: 0.4rem 0.7rem;
  font-size: 0.875rem;
  background: var(--game-btn-bg);
  color: var(--game-btn-text);
}

.btn:hover {
  background: var(--game-btn-hover-bg);
}
</style>
