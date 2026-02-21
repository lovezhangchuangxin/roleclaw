<template>
  <section class="settings-shell">
    <div class="panel settings-panel">
      <div class="settings-head">
        <h2 class="panel-title mb-0">游戏设置</h2>
        <p class="text-sm game-text-muted">主题切换会即时生效，当前存档与全局设置会自动同步。</p>
      </div>

      <div class="settings-grid">
        <label class="field">
          <span>主题</span>
          <Select v-model="gameSettings.theme">
            <SelectTrigger class="w-full settings-select-trigger">
              <SelectValue placeholder="选择主题" />
            </SelectTrigger>
            <SelectContent class="settings-select-content">
              <SelectItem v-for="theme in themeOptions" :key="theme.value" :value="theme.value"
                class="settings-select-item">
                <div class="settings-select-theme" :data-game-theme="theme.value">
                  <span class="settings-select-theme-name">{{ theme.label }}</span>
                  <span class="settings-select-swatch-row">
                    <span class="settings-select-swatch settings-select-swatch-panel" />
                    <span class="settings-select-swatch settings-select-swatch-primary" />
                    <span class="settings-select-swatch settings-select-swatch-accent" />
                  </span>
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
        </label>

        <label class="field">
          <span>消息速度</span>
          <Select v-model="gameSettings.messageSpeed">
            <SelectTrigger class="w-full settings-select-trigger">
              <SelectValue placeholder="选择速度" />
            </SelectTrigger>
            <SelectContent class="settings-select-content">
              <SelectItem v-for="speed in speedOptions" :key="speed.value" :value="speed.value"
                class="settings-select-item">
                {{ speed.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </label>

        <label class="field">
          <span>字体缩放</span>
          <input v-model.number="gameSettings.fontScale" class="input" type="range" min="0.9" max="1.35"
            step="0.05" />
          <small class="text-xs game-text-muted">{{ gameSettings.fontScale.toFixed(2) }}x</small>
        </label>

        <label class="field">
          <span>界面缩放</span>
          <input v-model.number="gameSettings.uiZoom" class="input" type="range" min="0.9" max="1.2"
            step="0.05" />
          <small class="text-xs game-text-muted">{{ gameSettings.uiZoom.toFixed(2) }}x</small>
        </label>

        <label class="field">
          <span>日志级别</span>
          <Select v-model="gameSettings.logLevel">
            <SelectTrigger class="w-full settings-select-trigger">
              <SelectValue placeholder="选择日志级别" />
            </SelectTrigger>
            <SelectContent class="settings-select-content">
              <SelectItem value="error" class="settings-select-item">错误</SelectItem>
              <SelectItem value="warn" class="settings-select-item">警告</SelectItem>
              <SelectItem value="info" class="settings-select-item">信息</SelectItem>
              <SelectItem value="debug" class="settings-select-item">调试</SelectItem>
            </SelectContent>
          </Select>
        </label>
      </div>

      <div class="theme-preview-grid">
        <button v-for="theme in themeOptions" :key="theme.value" type="button" class="theme-preview-card"
          :data-game-theme="theme.value"
          :class="{ 'theme-preview-card-active': gameSettings.theme === theme.value }"
          @click="gameSettings.theme = theme.value">
          <div class="theme-preview-surface">
            <div class="theme-preview-header">
              <span class="theme-preview-dot" />
              <span class="theme-preview-dot" />
              <span class="theme-preview-dot" />
            </div>
            <div class="theme-preview-lines">
              <span class="theme-preview-line" />
              <span class="theme-preview-line theme-preview-line-short" />
            </div>
            <div class="theme-preview-cta">Action</div>
          </div>
          <div class="theme-preview-meta">
            <p class="theme-preview-name">{{ theme.label }}</p>
            <p class="theme-preview-desc">{{ theme.description }}</p>
          </div>
        </button>
      </div>

      <div class="mt-5 flex justify-end">
        <button class="btn btn-primary" @click="saveGlobalGameData">保存游戏设置</button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameAppContext } from "@/composables/useGameAppContext";

const { gameSettings, saveGlobalGameData } = useGameAppContext();

const themeOptions = [
  { value: "default", label: "默认", description: "清晰平衡，通用阅读" },
  { value: "fantasy", label: "沉浸幻想", description: "暖色羊皮卷氛围" },
  { value: "terminal", label: "科幻终端", description: "高对比霓虹控制台" },
  { value: "archive", label: "古典档案", description: "旧纸档案式质感" },
] as const;

const speedOptions = [
  { value: "slow", label: "慢" },
  { value: "normal", label: "中" },
  { value: "fast", label: "快" },
] as const;
</script>
