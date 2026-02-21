<template>
  <div class="space-y-3">
    <div class="flex flex-wrap gap-2">
      <button class="btn" @click="addNode">新增地点</button>
      <button class="btn" :disabled="!selectedNode" @click="removeSelectedNode">删除选中地点</button>
      <button class="btn" :disabled="!selectedEdge" @click="removeSelectedEdge">删除选中连线</button>
      <button class="btn" :disabled="!selectedNode" @click="setStartNode">设为起点</button>
    </div>

    <p class="text-xs game-text-muted">
      点击节点选中，拖拽移动；依次点击两个节点即可创建/删除无向连线。当前起点：{{ localMap.startNodeId || "未设置" }}
    </p>

    <canvas
      ref="canvasRef"
      :width="localMap.canvas.width"
      :height="localMap.canvas.height"
      class="map-editor-canvas"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @click="onCanvasClick"
    />

    <div v-if="selectedNode" class="detail-panel">
      <h5 class="text-sm font-semibold">选中地点详情</h5>
      <label class="field">
        <span>地点名称</span>
        <input v-model="selectedNode.name" class="input" @input="emitMap" />
      </label>
      <label class="field">
        <span>地点描述</span>
        <textarea v-model="selectedNode.description" class="input h-20" @input="emitMap" />
      </label>
      <label class="field">
        <span>标签（逗号分隔）</span>
        <input :value="selectedNode.tags.join(',')" class="input" @input="setSelectedNodeTags(($event.target as HTMLInputElement).value)" />
      </label>
      <p class="text-xs game-text-muted">ID: {{ selectedNode.id }} · 坐标: ({{ Math.round(selectedNode.x) }}, {{ Math.round(selectedNode.y) }})</p>
    </div>

    <div v-else class="detail-panel text-sm game-text-muted">
      请选择一个地点节点后编辑详情。
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { WorldMap } from "@/types";

const props = defineProps<{
  modelValue: WorldMap;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: WorldMap];
}>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
const selectedNodeId = ref<string | null>(null);
const selectedEdgeId = ref<string | null>(null);
const dragNodeId = ref<string | null>(null);
const dragOffset = ref({ x: 0, y: 0 });
const linkAnchorNodeId = ref<string | null>(null);
const pointerDown = ref<{ x: number; y: number } | null>(null);
const draggedSinceDown = ref(false);

const localMap = ref<WorldMap>(cloneMap(props.modelValue));

const nodeById = computed(() => new Map(localMap.value.nodes.map((node) => [node.id, node])));
const selectedNode = computed(() => {
  if (!selectedNodeId.value) return null;
  return localMap.value.nodes.find((node) => node.id === selectedNodeId.value) ?? null;
});
const selectedEdge = computed(() => {
  if (!selectedEdgeId.value) return null;
  return localMap.value.edges.find((edge) => edge.id === selectedEdgeId.value) ?? null;
});

function cloneMap(map: WorldMap): WorldMap {
  return JSON.parse(JSON.stringify(map)) as WorldMap;
}

function splitCsv(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}

function emitMap() {
  emit("update:modelValue", cloneMap(localMap.value));
  draw();
}

function setSelectedNodeTags(value: string) {
  if (!selectedNode.value) return;
  selectedNode.value.tags = splitCsv(value);
  emitMap();
}

function pickNode(x: number, y: number): string | null {
  const hitRadius = 22;
  for (const node of [...localMap.value.nodes].reverse()) {
    const dx = x - node.x;
    const dy = y - node.y;
    if (Math.sqrt(dx * dx + dy * dy) <= hitRadius) {
      return node.id;
    }
  }
  return null;
}

function pickEdge(x: number, y: number): string | null {
  const threshold = 8;
  for (const edge of localMap.value.edges) {
    const a = nodeById.value.get(edge.a);
    const b = nodeById.value.get(edge.b);
    if (!a || !b) continue;
    const lenSq = (b.x - a.x) ** 2 + (b.y - a.y) ** 2;
    if (lenSq <= 1) continue;
    let t = ((x - a.x) * (b.x - a.x) + (y - a.y) * (b.y - a.y)) / lenSq;
    t = Math.max(0, Math.min(1, t));
    const px = a.x + t * (b.x - a.x);
    const py = a.y + t * (b.y - a.y);
    const d = Math.sqrt((x - px) ** 2 + (y - py) ** 2);
    if (d <= threshold) {
      return edge.id;
    }
  }
  return null;
}

function upsertEdge(aId: string, bId: string) {
  if (aId === bId) return;
  const exists = localMap.value.edges.find(
    (edge) => (edge.a === aId && edge.b === bId) || (edge.a === bId && edge.b === aId),
  );
  if (exists) {
    localMap.value.edges = localMap.value.edges.filter((edge) => edge.id !== exists.id);
    selectedEdgeId.value = null;
  } else {
    localMap.value.edges.push({
      id: `edge_${Date.now()}_${Math.random().toString(36).slice(2, 6)}`,
      a: aId,
      b: bId,
      locked: false,
      unlockConditions: [],
    });
  }
  emitMap();
}

function addNode() {
  const id = `loc_${Date.now()}`;
  localMap.value.nodes.push({
    id,
    name: `地点${localMap.value.nodes.length + 1}`,
    description: "",
    tags: [],
    x: 120 + localMap.value.nodes.length * 32,
    y: 120 + localMap.value.nodes.length * 26,
  });
  if (!localMap.value.startNodeId) {
    localMap.value.startNodeId = id;
  }
  selectedNodeId.value = id;
  emitMap();
}

function removeSelectedNode() {
  if (!selectedNodeId.value) return;
  const targetId = selectedNodeId.value;
  localMap.value.nodes = localMap.value.nodes.filter((node) => node.id !== targetId);
  localMap.value.edges = localMap.value.edges.filter((edge) => edge.a !== targetId && edge.b !== targetId);
  if (localMap.value.startNodeId === targetId) {
    localMap.value.startNodeId = localMap.value.nodes[0]?.id ?? "";
  }
  selectedNodeId.value = null;
  emitMap();
}

function removeSelectedEdge() {
  if (!selectedEdgeId.value) return;
  localMap.value.edges = localMap.value.edges.filter((edge) => edge.id !== selectedEdgeId.value);
  selectedEdgeId.value = null;
  emitMap();
}

function setStartNode() {
  if (!selectedNodeId.value) return;
  localMap.value.startNodeId = selectedNodeId.value;
  emitMap();
}

function getPos(evt: MouseEvent) {
  const canvas = canvasRef.value!;
  const rect = canvas.getBoundingClientRect();
  const scaleX = canvas.width / rect.width;
  const scaleY = canvas.height / rect.height;
  return {
    x: (evt.clientX - rect.left) * scaleX,
    y: (evt.clientY - rect.top) * scaleY,
  };
}

function onMouseDown(evt: MouseEvent) {
  const pos = getPos(evt);
  pointerDown.value = pos;
  draggedSinceDown.value = false;
  const nodeId = pickNode(pos.x, pos.y);
  if (!nodeId) return;
  const node = nodeById.value.get(nodeId);
  if (!node) return;
  selectedNodeId.value = nodeId;
  selectedEdgeId.value = null;
  dragNodeId.value = nodeId;
  dragOffset.value = { x: pos.x - node.x, y: pos.y - node.y };
  draw();
}

function onMouseMove(evt: MouseEvent) {
  if (!dragNodeId.value) return;
  const pos = getPos(evt);
  if (pointerDown.value) {
    const moved = Math.hypot(pos.x - pointerDown.value.x, pos.y - pointerDown.value.y);
    if (moved > 3) {
      draggedSinceDown.value = true;
    }
  }
  const node = nodeById.value.get(dragNodeId.value);
  if (!node) return;
  node.x = Math.max(20, Math.min(localMap.value.canvas.width - 20, pos.x - dragOffset.value.x));
  node.y = Math.max(20, Math.min(localMap.value.canvas.height - 20, pos.y - dragOffset.value.y));
  draw();
}

function onMouseUp() {
  pointerDown.value = null;
  if (!dragNodeId.value) return;
  dragNodeId.value = null;
  emitMap();
}

function onCanvasClick(evt: MouseEvent) {
  // Drag end may still fire click; ignore this click to avoid accidental re-selection/edge toggles.
  if (draggedSinceDown.value) {
    draggedSinceDown.value = false;
    return;
  }
  const pos = getPos(evt);
  const nodeId = pickNode(pos.x, pos.y);
  if (nodeId) {
    selectedNodeId.value = nodeId;
    selectedEdgeId.value = null;
    if (!linkAnchorNodeId.value) {
      linkAnchorNodeId.value = nodeId;
    } else {
      upsertEdge(linkAnchorNodeId.value, nodeId);
      linkAnchorNodeId.value = null;
    }
    draw();
    return;
  }

  const edgeId = pickEdge(pos.x, pos.y);
  if (edgeId) {
    selectedEdgeId.value = edgeId;
    selectedNodeId.value = null;
    linkAnchorNodeId.value = null;
    draw();
    return;
  }

  selectedNodeId.value = null;
  selectedEdgeId.value = null;
  linkAnchorNodeId.value = null;
  draw();
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "rgba(15, 23, 42, 0.05)";
  ctx.fillRect(0, 0, canvas.width, canvas.height);

  for (const edge of localMap.value.edges) {
    const a = nodeById.value.get(edge.a);
    const b = nodeById.value.get(edge.b);
    if (!a || !b) continue;
    ctx.strokeStyle = selectedEdgeId.value === edge.id ? "#dc2626" : "#334155";
    ctx.lineWidth = selectedEdgeId.value === edge.id ? 3 : 2;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.lineTo(b.x, b.y);
    ctx.stroke();
  }

  for (const node of localMap.value.nodes) {
    const isStart = localMap.value.startNodeId === node.id;
    const isSelected = selectedNodeId.value === node.id;
    ctx.beginPath();
    ctx.arc(node.x, node.y, isSelected ? 13 : 10, 0, Math.PI * 2);
    ctx.fillStyle = isStart ? "#16a34a" : isSelected ? "#dc2626" : "#2563eb";
    ctx.fill();

    ctx.fillStyle = "#0f172a";
    ctx.font = "12px sans-serif";
    ctx.fillText(node.name, node.x + 12, node.y + 4);
  }
}

watch(
  () => props.modelValue,
  (next) => {
    localMap.value = cloneMap(next);
    if (selectedNodeId.value && !nodeById.value.has(selectedNodeId.value)) {
      selectedNodeId.value = null;
    }
    if (
      selectedEdgeId.value &&
      !localMap.value.edges.some((edge) => edge.id === selectedEdgeId.value)
    ) {
      selectedEdgeId.value = null;
    }
    draw();
  },
  { deep: true, immediate: true },
);
</script>

<style scoped>
.map-editor-canvas {
  width: 100%;
  max-width: 100%;
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: var(--game-canvas-bg);
  cursor: crosshair;
}

.btn {
  border: 1px solid var(--game-btn-border);
  border-radius: var(--radius-md);
  padding: 0.4rem 0.7rem;
  font-size: 0.8rem;
  background: var(--game-btn-bg);
  color: var(--game-btn-text);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.detail-panel {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.65rem;
  display: grid;
  gap: 0.5rem;
}

.field {
  display: grid;
  gap: 0.35rem;
}

.input {
  border: 1px solid var(--game-input-border);
  border-radius: var(--radius-md);
  background: var(--game-input-bg);
  padding: 0.5rem 0.625rem;
}

.game-text-muted {
  color: var(--game-text-muted);
}
</style>
