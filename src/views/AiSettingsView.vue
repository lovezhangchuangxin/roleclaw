<template>
  <section class="panel w-full ai-settings-panel">
    <h2 class="panel-title">AI设置</h2>
    <p class="text-sm game-text-muted">当前仅支持 OpenAI Compatible 协议配置。</p>

    <div class="ai-layout mt-4">
      <div class="ai-block ai-block-scroll">
        <div class="ai-block-head">
          <h3 class="ai-block-title">{{ editingModelId ? "编辑模型" : "新建模型" }}</h3>
          <p class="text-xs game-text-muted">左侧维护模型连接信息，保存后立即生效。</p>
        </div>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="field">
            <span>协议类型</span>
            <Select v-model="aiDraft.providerType">
              <SelectTrigger class="w-full settings-select-trigger">
                <SelectValue placeholder="选择协议" />
              </SelectTrigger>
              <SelectContent class="settings-select-content">
                <SelectItem value="openai_compatible" class="settings-select-item">OpenAI Compatible</SelectItem>
              </SelectContent>
            </Select>
          </label>
          <label class="field">
            <span>Provider 名称</span>
            <input v-model="aiDraft.provider" class="input" placeholder="例如 OpenAI / DeepSeek" />
          </label>
          <label class="field md:col-span-2">
            <span>Base URL</span>
            <input v-model="aiDraft.baseUrl" class="input" placeholder="例如 https://api.openai.com/v1" />
          </label>
          <label class="field md:col-span-2">
            <span>API Key</span>
            <input v-model="aiDraft.apiKey" class="input" type="password" placeholder="sk-..." />
          </label>
          <label class="field md:col-span-2">
            <span>模型名</span>
            <input v-model="aiDraft.model" class="input" />
          </label>
          <label class="field">
            <span>Temperature</span>
            <input v-model.number="aiDraft.temperature" class="input" type="number" step="0.1" />
          </label>
          <label class="field">
            <span>Timeout(ms)</span>
            <input v-model.number="aiDraft.timeoutMs" class="input" type="number" step="100" />
          </label>
        </div>

        <div class="mt-4 flex flex-wrap gap-2">
          <button class="btn" @click="testAiDraft">连通性测试</button>
          <button class="btn btn-primary" @click="saveAiModel">保存模型</button>
          <button class="btn" @click="resetAiDraft">重置为新建</button>
        </div>

        <div v-if="modelCheckMsg" class="mt-3 model-check-msg"
          :class="modelCheckOk ? 'model-check-success' : 'model-check-fail'">
          <p class="text-sm">{{ modelCheckMsg }}</p>
          <button v-if="modelCheckOk === false" class="btn model-check-copy-btn" @click="copyModelCheckError">
            {{ copiedModelCheck ? "已复制" : "复制失败原因" }}
          </button>
        </div>
      </div>

      <div class="ai-block">
        <div class="ai-block-head">
          <h3 class="ai-block-title">已配置模型</h3>
          <p class="text-xs game-text-muted">共 {{ aiModels.length }} 个 · 默认 {{ defaultModelId || "未设置" }}</p>
        </div>
        <div class="mb-3 flex flex-wrap gap-2">
          <button class="btn" @click="selectAllForExport">全选导出</button>
          <button class="btn" @click="clearExportSelection">清空勾选</button>
          <button class="btn btn-primary" @click="generateSelectedExportJson">
            导出勾选({{ selectedExportIds.length }})
          </button>
          <button class="btn" @click="generateAllExportJson">导出全部</button>
        </div>
        <div class="ai-list">
          <div v-for="model in aiModels" :key="model.id" class="ai-list-item"
            :class="{ 'ai-list-item-active': editingModelId === model.id }" @click="selectAiModel(model.id)">
            <div class="ai-list-main">
              <p class="font-medium">{{ model.provider }}/{{ model.model }}</p>
              <p class="text-xs game-text-muted">{{ model.providerType }}</p>
              <p class="text-xs game-text-muted truncate">{{ model.baseUrl }}</p>
            </div>
            <div class="ai-list-actions">
              <span v-if="defaultModelId === model.id" class="ai-default-badge">默认</span>
              <label class="ai-export-check">
                <input type="checkbox" :checked="selectedExportIds.includes(model.id)"
                  @click.stop="toggleExportModel(model.id)" />
                <span>导出</span>
              </label>
              <button class="ai-action-btn" @click.stop="markDefaultAiModel(model.id)">设为默认</button>
              <button class="ai-action-btn ai-action-btn-danger"
                @click.stop="confirmRemoveAiModel(model.id)">删除</button>
            </div>
          </div>
          <div v-if="aiModels.length === 0" class="ai-empty">
            <p class="text-sm game-text-muted">当前还没有配置任何 AI 模型。</p>
            <p class="text-xs game-text-muted">请在左侧填写参数并点击“保存模型”。</p>
          </div>
        </div>

        <div class="mt-4 ai-transfer-grid">
          <label class="field">
            <span>导入模型配置 JSON（支持对象/数组）</span>
            <textarea v-model="modelImportText" class="input ai-transfer-textarea"
              placeholder='示例：{"models":[{"provider":"OpenAI","baseUrl":"https://api.openai.com/v1","model":"gpt-4.1","apiKey":"..."}]}' />
          </label>
          <div class="flex flex-wrap gap-2">
            <button class="btn btn-primary" @click="runImportModels">导入模型配置</button>
            <button class="btn" @click="modelImportText = ''">清空导入</button>
          </div>
          <p v-if="importExportMsg" class="text-xs game-text-muted">{{ importExportMsg }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import { toast } from "vue-sonner";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameAppContext } from "@/composables/useGameAppContext";

const {
  modelCheckMsg,
  modelCheckOk,
  aiModels,
  defaultModelId,
  editingModelId,
  aiDraft,
  selectAiModel,
  resetAiDraft,
  testAiDraft,
  saveAiModel,
  removeAiModel,
  markDefaultAiModel,
  modelImportText,
  exportAiModels,
  importAiModelsFromText,
} = useGameAppContext();

const copiedModelCheck = ref(false);
const importExportMsg = ref("");
const selectedExportIds = ref<string[]>([]);

watch(modelCheckMsg, () => {
  copiedModelCheck.value = false;
});

watch(aiModels, (models) => {
  const idSet = new Set(models.map((item) => item.id));
  selectedExportIds.value = selectedExportIds.value.filter((id) => idSet.has(id));
});

async function confirmRemoveAiModel(modelId: string) {
  const ok = window.confirm("确认删除该 AI 模型吗？");
  if (!ok) {
    return;
  }
  await removeAiModel(modelId);
}

function toggleExportModel(modelId: string) {
  if (selectedExportIds.value.includes(modelId)) {
    selectedExportIds.value = selectedExportIds.value.filter((id) => id !== modelId);
    return;
  }
  selectedExportIds.value = [...selectedExportIds.value, modelId];
}

function selectAllForExport() {
  selectedExportIds.value = aiModels.value.map((item) => item.id);
}

function clearExportSelection() {
  selectedExportIds.value = [];
}

function generateSelectedExportJson() {
  const ids = selectedExportIds.value;
  if (ids.length === 0) {
    importExportMsg.value = "请先勾选至少一个模型。";
    return;
  }
  void copyExportJson(ids, `已复制 ${ids.length} 个模型配置到剪贴板`);
}

function generateAllExportJson() {
  void copyExportJson([], `已复制 ${aiModels.value.length} 个模型配置到剪贴板`);
}

async function runImportModels() {
  try {
    const count = await importAiModelsFromText();
    importExportMsg.value = `导入成功，共处理 ${count} 个模型配置。`;
    toast.success(`导入成功，共处理 ${count} 个模型配置`);
  } catch (err) {
    importExportMsg.value = err instanceof Error ? err.message : "导入失败，请检查 JSON 格式";
    toast.error(importExportMsg.value);
  }
}

async function copyModelCheckError() {
  if (!modelCheckMsg.value || modelCheckOk.value !== false) {
    return;
  }
  try {
    await navigator.clipboard.writeText(modelCheckMsg.value);
    copiedModelCheck.value = true;
    setTimeout(() => {
      copiedModelCheck.value = false;
    }, 1200);
  } catch {
    copiedModelCheck.value = false;
  }
}

async function copyExportJson(modelIds: string[], successMsg: string) {
  try {
    const content = exportAiModels(modelIds);
    await navigator.clipboard.writeText(content);
    toast.success(successMsg);
  } catch (err) {
    toast.error(err instanceof Error ? err.message : "复制失败");
  }
}
</script>
