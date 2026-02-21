<template>
  <div class="menu-shell">
    <div class="menu-card">
      <p class="menu-kicker">ROLECLAW</p>
      <h2 class="menu-title">{{ title }}</h2>
      <p class="menu-subtitle">{{ subtitle }}</p>

      <div class="menu-actions">
        <button class="menu-btn" @click="$emit('select', 'start')">开始游戏</button>
        <button class="menu-btn" @click="$emit('select', 'saves')">存档管理</button>
        <button class="menu-btn" @click="$emit('select', 'ai')">AI设置</button>
        <button class="menu-btn" @click="$emit('select', 'cards')">世界卡管理</button>
        <button class="menu-btn" @click="$emit('select', 'settings')">游戏设置</button>
        <button class="menu-btn menu-btn-danger" @click="$emit('select', 'exit')">退出</button>
      </div>

      <button v-if="showClose" class="menu-close" @click="$emit('close')">关闭菜单 (Esc)</button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  subtitle: string;
  showClose?: boolean;
}>();

defineEmits<{
  select: [action: "start" | "saves" | "ai" | "cards" | "settings" | "exit"];
  close: [];
}>();
</script>

<style scoped>
.menu-shell {
  display: flex;
  justify-content: center;
}

.menu-card {
  width: min(460px, 90vw);
  border: 1px solid var(--game-menu-card-border);
  border-radius: 16px;
  padding: 1.4rem;
  background: var(--game-menu-card-bg);
  color: var(--game-btn-text);
  box-shadow: 0 20px 50px var(--game-menu-shadow);
}

.menu-kicker {
  letter-spacing: 0.2em;
  font-size: 0.7rem;
  color: var(--game-menu-kicker);
  margin-bottom: 0.4rem;
}

.menu-title {
  font-size: 1.4rem;
  font-weight: 700;
  color: var(--game-btn-text);
  margin-bottom: 0.25rem;
}

.menu-subtitle {
  font-size: 0.875rem;
  color: var(--game-text-muted);
  margin-bottom: 1rem;
}

.menu-actions {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  align-items: stretch;
}

.menu-btn {
  width: 100%;
  text-align: center;
  border: 1px solid var(--game-btn-border);
  border-radius: var(--radius-md);
  padding: 0.62rem 0.8rem;
  font-size: 0.95rem;
  background: var(--game-btn-bg);
  color: var(--game-btn-text);
  transition: transform 120ms ease, background 120ms ease;
}

.menu-btn:hover {
  transform: translateY(-1px);
  background: var(--game-btn-hover-bg);
}

.menu-btn-danger {
  color: var(--game-danger-text);
}

.menu-close {
  margin-top: 0.9rem;
  width: 100%;
  border: 1px dashed var(--game-btn-border);
  border-radius: var(--radius-md);
  padding: 0.45rem;
  font-size: 0.8rem;
  color: var(--game-text-muted);
}
</style>
