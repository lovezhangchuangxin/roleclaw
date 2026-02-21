<template>
  <div class="app-root min-h-screen p-6" :style="appStyleVars">
    <div v-if="view === 'menu'" class="start-screen">
      <div class="start-stack">
        <div class="start-title-wrap">
          <p class="start-kicker">AI ROLEPLAYING GAME</p>
          <h1 class="start-title">RoleClaw</h1>
        </div>
        <GameSettingsMenu title="主菜单" subtitle="选择你的下一步行动" @select="handleMenuSelect" />
      </div>
    </div>

    <template v-else>
      <header class="mb-6 flex items-center justify-between">
        <div>
          <h1 class="text-2xl font-semibold">RoleClaw AI RPG</h1>
          <p class="text-sm game-text-muted">叙事驱动 · 世界卡生成 · 3+1 对话</p>
        </div>
        <div class="flex gap-2">
          <button v-if="activeSave" class="btn" @click="openReplayView">回放/分叉</button>
          <button class="btn" @click="setView('menu')">主菜单</button>
          <button class="btn" @click="refreshHome">刷新</button>
        </div>
      </header>

      <p v-if="errorMsg" class="error-banner mb-4 rounded p-2 text-sm">
        {{ errorMsg }}
      </p>

      <section v-if="view === 'new'" class="new-game-shell">
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

      <section v-if="view === 'saves'" class="saves-shell">
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

      <section v-if="view === 'ai-settings'" class="panel w-full ai-settings-panel">
        <h2 class="panel-title">AI设置</h2>
        <p class="text-sm game-text-muted">当前仅支持 OpenAI Compatible 协议配置。</p>

        <div class="ai-layout mt-4">
          <div class="ai-block">
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
          </div>
        </div>
      </section>

      <section v-if="view === 'settings'" class="settings-shell">
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

      <section v-if="view === 'cards'" class="w-full">
        <WorldCardManager :world-cards="worldCards" :card-import-text="cardImportText"
          :card-export-path="cardExportPath" :ai-world-card-prompt="aiWorldCardPrompt"
          :ai-world-card-generating="aiWorldCardGenerating" :ai-generated-card="aiGeneratedWorldCard"
          :ai-world-card-stream-text="aiWorldCardStreamText" :ai-world-card-stream-parsed-ok="aiWorldCardStreamParsedOk"
          @update:card-import-text="cardImportText = $event" @update:card-export-path="cardExportPath = $event"
          @update:ai-world-card-prompt="aiWorldCardPrompt = $event" @import-card="importCardFromText"
          @export-card="exportCard" @duplicate-card="duplicateCard" @save-edited-card="saveEditedCard"
          @generate-card-by-ai="generateCardDraftWithAi" />
      </section>

      <section v-if="view === 'replay' && activeSave" class="panel max-w-6xl">
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

      <section v-if="view === 'game' && activeSave" class="game-shell">
        <div class="panel game-panel">
          <h2 class="panel-title">地图</h2>
          <div class="game-panel-content">
            <GameMapCanvas :snapshot="activeSave.snapshot" :reachable-locations="reachableLocations" @move="move" />
          </div>
        </div>

        <div class="panel game-panel">
          <h2 class="panel-title">叙事与对话</h2>
          <div class="game-panel-content game-story-content">
            <p class="text-sm leading-6">{{ narrationText }}</p>

            <div class="space-y-2">
              <button v-for="opt in options" :key="opt.id" class="btn w-full text-left" @click="submitOption(opt.id)">
                [{{ opt.kind }}] {{ opt.text }}
              </button>
            </div>

            <div class="game-custom-row mt-auto flex gap-2">
              <input v-model="customInput" class="input flex-1" placeholder="输入你的自定义第四选项..." />
              <button class="btn btn-primary" @click="submitCustom">提交</button>
            </div>
          </div>
        </div>

        <div class="panel game-panel">
          <h2 class="panel-title">状态</h2>
          <div class="game-panel-content game-state-content">
            <p class="text-sm">存档：{{ activeSave.meta.name }}</p>
            <p class="text-sm">回合：{{ activeSave.snapshot.turn }}</p>
            <p class="text-sm">角色：{{ activeSave.snapshot.playerRole }}</p>
            <p class="text-sm">模型：{{ activeSave.snapshot.modelLabel || activeSave.meta.model }}</p>

            <h3 class="mt-4 font-medium">最近变化</h3>
            <ul class="mt-2 list-disc pl-5 text-sm">
              <li v-for="line in stateChanges" :key="line">{{ line }}</li>
            </ul>

            <h3 class="mt-4 font-medium">任务进度</h3>
            <ul class="mt-2 space-y-1 text-xs">
              <li v-for="quest in activeSave.snapshot.quests" :key="quest.id" class="rounded border px-2 py-1">
                {{ quest.title }} · 阶段 {{ quest.stage }} · {{ quest.completed ? "已完成" : "进行中" }}
              </li>
              <li v-if="activeSave.snapshot.quests.length === 0" class="game-text-muted">暂无任务</li>
            </ul>

            <h3 class="mt-4 font-medium">关系矩阵</h3>
            <ul class="mt-2 space-y-1 text-xs">
              <li v-for="entry in relationshipEntries" :key="entry.id" class="rounded border px-2 py-1">
                {{ entry.id }}: {{ entry.value }}
              </li>
              <li v-if="relationshipEntries.length === 0" class="game-text-muted">暂无关系记录</li>
            </ul>

            <div class="mt-4 flex flex-wrap gap-2">
              <button class="btn" @click="openReplayView">打开回放页</button>
              <button class="btn btn-primary" @click="forkActiveSave">从当前回合分叉</button>
            </div>
            <ul v-if="replayPreview.length > 0" class="mt-3 space-y-1 text-xs">
              <li v-for="item in replayPreview" :key="item.turn" class="rounded border px-2 py-1">
                T{{ item.turn }} · {{ item.output?.stateChangesPreview?.join(" / ") || "无摘要" }}
              </li>
            </ul>

            <div v-if="gameSettings.logLevel === 'debug'" class="mt-3 rounded border p-2 text-xs">
              <p class="font-medium mb-1">调试状态</p>
              <pre class="overflow-auto">{{ JSON.stringify(activeSave.snapshot.worldVariables, null, 2) }}</pre>
            </div>
          </div>
        </div>
      </section>

      <div v-if="showInGameMenu && view === 'game'" class="overlay">
        <GameSettingsMenu title="游戏菜单" subtitle="按 Esc 关闭菜单并继续游戏" :show-close="true" @select="handleMenuSelect"
          @close="showInGameMenu = false" />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import GameMapCanvas from "@/components/game/GameMapCanvas.vue";
import GameSettingsMenu from "@/components/game/GameSettingsMenu.vue";
import WorldCardManager from "@/components/game/WorldCardManager.vue";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useGameApp } from "@/composables/useGameApp";

const {
  view,
  errorMsg,
  modelCheckMsg,
  modelCheckOk,
  narrationText,
  stateChanges,
  options,
  customInput,
  cardImportText,
  cardExportPath,
  aiWorldCardPrompt,
  aiWorldCardGenerating,
  aiGeneratedWorldCard,
  aiWorldCardStreamText,
  aiWorldCardStreamParsedOk,
  replayPreview,
  replayResult,
  replaySelectedTurn,
  replayHasMore,
  replayLoading,
  saves,
  worldCards,
  activeSave,
  newSave,
  gameSettings,
  aiModels,
  defaultModelId,
  editingModelId,
  aiDraft,
  reachableLocations,
  setView,
  refreshHome,
  openSave,
  removeSave,
  selectAiModel,
  resetAiDraft,
  testAiDraft,
  saveAiModel,
  removeAiModel,
  markDefaultAiModel,
  saveGlobalGameData,
  createNewSave,
  submitOption,
  submitCustom,
  move,
  importCardFromText,
  exportCard,
  duplicateCard,
  saveEditedCard,
  generateCardDraftWithAi,
  loadReplayTimeline,
  refreshReplayData,
  openReplayView,
  forkAtTurn,
  forkActiveSave,
} = useGameApp();

const showInGameMenu = ref(false);
const copiedModelCheck = ref(false);
const saveSearch = ref("");
const saveSort = ref<"updated_desc" | "created_desc" | "turn_desc" | "name_asc">(
  "updated_desc",
);
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
const relationshipEntries = computed(() => {
  if (!activeSave.value) {
    return [];
  }
  return Object.entries(activeSave.value.snapshot.relationships).map(([id, value]) => ({
    id,
    value: typeof value === "number" ? value.toFixed(1) : String(value),
  }));
});
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
const selectedReplayItem = computed(() => {
  if (replaySelectedTurn.value == null) {
    return replayPreview.value[replayPreview.value.length - 1] ?? null;
  }
  return replayPreview.value.find((item) => item.turn === replaySelectedTurn.value) ?? null;
});
const appStyleVars = computed<Record<string, string>>(() => ({
  "--game-font-scale": String(gameSettings.value.fontScale ?? 1),
  "--game-ui-zoom": String(gameSettings.value.uiZoom ?? 1),
}));

function formatDateTime(value: string): string {
  if (!value) return "--";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function handleMenuSelect(action: "start" | "saves" | "ai" | "cards" | "settings" | "exit") {
  if (action === "exit") {
    exitGame();
    return;
  }

  showInGameMenu.value = false;
  if (action === "start") {
    if (!defaultModelId.value) {
      errorMsg.value = "无法开始游戏：当前没有默认 AI 模型。请先到 AI 设置新增模型并设为默认。";
      setView("ai-settings");
      return;
    }
    setView("new");
  } else if (action === "saves") {
    setView("saves");
  } else if (action === "ai") {
    setView("ai-settings");
  } else if (action === "cards") {
    setView("cards");
  } else if (action === "settings") {
    setView("settings");
  }
}

async function exitGame() {
  try {
    const appWindow = getCurrentWindow();
    await appWindow.close();
  } catch {
    window.close();
  }
}

async function confirmRemoveAiModel(modelId: string) {
  const ok = window.confirm("确认删除该 AI 模型吗？");
  if (!ok) {
    return;
  }
  await removeAiModel(modelId);
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

function onKeydown(event: KeyboardEvent) {
  if (event.key !== "Escape") {
    return;
  }
  if (view.value !== "game") {
    return;
  }
  showInGameMenu.value = !showInGameMenu.value;
}

function applyTheme(theme: string) {
  document.documentElement.setAttribute("data-game-theme", theme);
}

watch(
  () => gameSettings.value.theme,
  async (next, prev) => {
    applyTheme(next);
    if (prev && prev !== next) {
      await saveGlobalGameData();
    }
  }
);

watch(modelCheckMsg, () => {
  copiedModelCheck.value = false;
});

onMounted(async () => {
  await refreshHome();
  applyTheme(gameSettings.value.theme);
  window.addEventListener("keydown", onKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeydown);
});
</script>

<style scoped>
.app-root {
  background-color: var(--game-page-bg);
  color: var(--game-btn-text);
  font-size: calc(16px * var(--game-font-scale, 1));
  zoom: var(--game-ui-zoom, 1);
  background:
    radial-gradient(circle at 10% 0%, var(--game-bg-layer-1), transparent 28%),
    radial-gradient(circle at 90% 90%, var(--game-bg-layer-2), transparent 30%);
}

.start-screen {
  min-height: calc(100vh - 3rem);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
}

.start-screen::before {
  content: "";
  position: absolute;
  inset: 0;
  pointer-events: none;
  opacity: 0.45;
  background-image:
    linear-gradient(to right, var(--game-panel-border) 1px, transparent 1px),
    linear-gradient(to bottom, var(--game-panel-border) 1px, transparent 1px);
  background-size: 28px 28px;
  mask-image: radial-gradient(circle at 50% 40%, var(--game-mask-dark) 30%, transparent 90%);
}

.start-stack {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  animation: menu-reveal 520ms ease-out 120ms both;
}

.start-title-wrap {
  text-align: center;
  animation: title-reveal 700ms cubic-bezier(0.16, 1, 0.3, 1) both;
}

.start-title-wrap::after {
  content: "";
  display: block;
  width: clamp(140px, 28vw, 260px);
  height: 16px;
  margin: 0.5rem auto 0;
  border-radius: 999px;
  background: radial-gradient(circle, var(--game-bg-layer-1), transparent 70%);
  filter: blur(6px);
  opacity: 0.8;
  animation: title-glow 2.6s ease-in-out 1s infinite;
}

.start-kicker {
  font-size: 0.72rem;
  letter-spacing: 0.22em;
  color: var(--game-menu-kicker);
}

.start-title {
  font-size: clamp(2.2rem, 5vw, 3.4rem);
  line-height: 1.05;
  font-weight: 800;
  letter-spacing: 0.04em;
}

@keyframes title-reveal {
  from {
    opacity: 0;
    transform: translateY(14px) scale(0.985);
    filter: blur(2px);
  }

  to {
    opacity: 1;
    transform: translateY(0) scale(1);
    filter: blur(0);
  }
}

@keyframes menu-reveal {
  from {
    opacity: 0;
    transform: translateY(10px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes title-glow {

  0%,
  100% {
    opacity: 0.65;
    transform: scaleX(0.96);
  }

  50% {
    opacity: 1;
    transform: scaleX(1.03);
  }
}

.overlay {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--game-overlay-bg);
  backdrop-filter: blur(2px);
  z-index: 40;
}

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

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
  font-size: 0.875rem;
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

.btn:hover {
  background: var(--game-btn-hover-bg);
}

.replay-item-active {
  border-color: var(--game-btn-primary-bg);
  box-shadow: 0 0 0 1px var(--game-btn-primary-bg);
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--game-btn-primary-bg);
  color: var(--game-btn-primary-text);
}

.btn-primary:hover {
  background: var(--game-btn-primary-hover-bg);
}

.game-shell {
  min-height: calc(100dvh - 11rem);
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(0, 1.2fr) minmax(0, 1.8fr) minmax(0, 1fr);
  align-items: stretch;
}

.game-panel {
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.game-panel-content {
  min-height: 0;
}

.game-story-content {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  height: 100%;
}

.game-state-content {
  overflow: auto;
  padding-right: 0.2rem;
}

.error-banner {
  border: 1px solid var(--game-error-border);
  background: var(--game-error-bg);
  color: var(--game-error-text);
}

.ai-layout {
  display: grid;
  gap: 1rem;
  grid-template-columns: minmax(420px, 1.7fr) minmax(360px, 1fr);
  flex: 1;
  min-height: 0;
}

@media (max-width: 960px) {
  .ai-layout {
    grid-template-columns: 1fr;
  }
}

.ai-block {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-lg);
  padding: 1rem;
  background: color-mix(in oklab, var(--game-panel-bg) 92%, var(--game-mix-light) 8%);
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.ai-settings-panel {
  min-height: calc(100dvh - 11rem);
  display: flex;
  flex-direction: column;
}

.ai-block-head {
  margin-bottom: 0.75rem;
}

.ai-block-title {
  font-weight: 600;
}

.model-check-msg {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.7rem 0.75rem;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.model-check-success {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 55%, var(--game-panel-border) 45%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 14%, transparent);
}

.model-check-fail {
  border-color: var(--game-error-border);
  background: color-mix(in oklab, var(--game-error-bg) 75%, transparent);
}

.model-check-copy-btn {
  white-space: nowrap;
  padding: 0.2rem 0.56rem;
  font-size: 0.76rem;
  border-color: var(--game-panel-border);
  background: color-mix(in oklab, var(--game-panel-bg) 92%, transparent);
  color: var(--game-btn-text);
}

.model-check-copy-btn:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 35%, var(--game-panel-border) 65%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
}

.ai-list {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  flex: 1;
  min-height: 220px;
  overflow: auto;
  padding-right: 0.2rem;
}

.ai-list-item {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: color-mix(in oklab, var(--game-panel-bg) 96%, transparent);
  padding: 0.68rem 0.72rem;
  width: 100%;
  text-align: left;
  display: flex;
  justify-content: space-between;
  gap: 0.8rem;
  transition: border-color 140ms ease, background 140ms ease;
}

.ai-list-item:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 26%, var(--game-panel-border) 74%);
  background: color-mix(in oklab, var(--game-panel-bg) 90%, var(--game-mix-light) 10%);
}

.ai-list-item-active {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 48%, var(--game-panel-border) 52%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
}

.ai-list-main {
  min-width: 0;
}

.ai-list-actions {
  display: flex;
  gap: 0.4rem;
  align-items: center;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-content: flex-start;
  min-width: 182px;
}

.ai-action-btn {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background: color-mix(in oklab, var(--game-panel-bg) 94%, transparent);
  color: var(--game-btn-text);
  font-size: 0.76rem;
  line-height: 1;
  padding: 0.34rem 0.52rem;
  transition: border-color 120ms ease, background 120ms ease;
}

.ai-action-btn:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 38%, var(--game-panel-border) 62%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 8%, var(--game-panel-bg) 92%);
}

.ai-action-btn-danger:hover {
  border-color: color-mix(in oklab, var(--game-error-border) 60%, var(--game-panel-border) 40%);
  background: color-mix(in oklab, var(--game-error-bg) 25%, var(--game-panel-bg) 75%);
}

.ai-default-badge {
  border: 1px solid color-mix(in oklab, var(--game-btn-primary-bg) 45%, var(--game-panel-border) 55%);
  color: color-mix(in oklab, var(--game-btn-text) 82%, var(--game-btn-primary-text) 18%);
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
  border-radius: 999px;
  padding: 0.1rem 0.46rem;
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.01em;
}

.ai-empty {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.8rem;
  background: color-mix(in oklab, var(--game-panel-bg) 94%, var(--game-mix-light) 6%);
}

.settings-shell {
  min-height: calc(100dvh - 11rem);
  display: flex;
  align-items: center;
  justify-content: center;
}

.new-game-shell {
  min-height: calc(100dvh - 11rem);
  display: flex;
  align-items: center;
  justify-content: center;
}

.new-game-layout {
  width: min(1100px, 100%);
  display: grid;
  grid-template-columns: minmax(260px, 0.9fr) minmax(0, 1.4fr);
  gap: 0.9rem;
  align-items: stretch;
}

.new-game-overview {
  padding: 1rem;
  display: grid;
  align-content: start;
  gap: 0.65rem;
}

.new-game-overview-content {
  display: grid;
  gap: 0.65rem;
}

.new-game-overview-head {
  padding-bottom: 0.4rem;
  border-bottom: 1px dashed var(--game-panel-border);
}

.new-game-panel {
  padding: 1.15rem;
}

.new-game-head {
  margin-bottom: 0.85rem;
}

.new-game-warning {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.55rem 0.65rem;
  margin-bottom: 0.85rem;
  background: color-mix(in oklab, var(--game-panel-bg) 88%, var(--game-mix-light) 12%);
}

.new-game-form {
  align-items: end;
}

.new-game-actions {
  justify-content: flex-end;
}

.new-game-metrics {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.45rem;
}

.new-game-metric {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.45rem 0.55rem;
  background: color-mix(in oklab, var(--game-panel-bg) 92%, var(--game-mix-light) 8%);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.new-game-metric-label {
  font-size: 0.74rem;
  color: var(--game-text-muted);
}

.new-game-tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.36rem;
}

.new-game-tag {
  border: 1px solid var(--game-panel-border);
  border-radius: 999px;
  padding: 0.12rem 0.46rem;
  font-size: 0.72rem;
  background: color-mix(in oklab, var(--game-btn-primary-bg) 10%, var(--game-panel-bg) 90%);
}

.saves-shell {
  min-height: calc(100dvh - 11rem);
  display: flex;
  justify-content: center;
  align-items: flex-start;
}

.saves-panel {
  width: min(1040px, 100%);
  padding: 1rem;
}

.saves-head {
  display: flex;
  justify-content: space-between;
  align-items: flex-end;
  gap: 0.8rem;
  margin-bottom: 0.9rem;
}

.saves-summary {
  display: flex;
  gap: 0.45rem;
  flex-wrap: wrap;
}

.saves-summary-item {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.38rem 0.6rem;
  min-width: 86px;
  display: grid;
  gap: 0.06rem;
  background: color-mix(in oklab, var(--game-panel-bg) 92%, var(--game-mix-light) 8%);
}

.saves-summary-item span {
  font-size: 0.7rem;
  color: var(--game-text-muted);
}

.saves-summary-item b {
  font-size: 0.9rem;
}

.saves-list {
  display: grid;
  gap: 0.58rem;
}

.saves-toolbar {
  margin-bottom: 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.5rem;
  flex-wrap: wrap;
}

.saves-search {
  width: min(360px, 100%);
}

.save-slot {
  border: 1px solid var(--game-panel-border);
  border-radius: calc(var(--radius-lg) + 2px);
  background:
    linear-gradient(95deg, color-mix(in oklab, var(--game-panel-bg) 96%, transparent), color-mix(in oklab, var(--game-bg-layer-1) 24%, transparent));
  padding: 0.75rem;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.8rem;
  align-items: center;
}

.save-slot:hover {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 42%, var(--game-panel-border) 58%);
}

.save-slot-main {
  min-width: 0;
  display: grid;
  gap: 0.28rem;
}

.save-slot-title-row {
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.save-slot-title {
  font-size: 1rem;
  font-weight: 600;
}

.save-slot-badge {
  border: 1px solid color-mix(in oklab, var(--game-btn-primary-bg) 48%, var(--game-panel-border) 52%);
  border-radius: 999px;
  font-size: 0.68rem;
  padding: 0.05rem 0.42rem;
  background: color-mix(in oklab, var(--game-btn-primary-bg) 12%, var(--game-panel-bg) 88%);
}

.save-slot-subtitle {
  font-size: 0.8rem;
  color: var(--game-text-muted);
}

.save-slot-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.55rem;
  font-size: 0.72rem;
  color: color-mix(in oklab, var(--game-btn-text) 78%, transparent);
}

.save-slot-id {
  font-size: 0.7rem;
  color: color-mix(in oklab, var(--game-btn-text) 62%, transparent);
}

.save-slot-actions {
  display: flex;
  gap: 0.45rem;
  align-items: center;
}

.saves-empty {
  border: 1px dashed var(--game-panel-border);
  border-radius: var(--radius-md);
  padding: 0.85rem;
  background: color-mix(in oklab, var(--game-panel-bg) 95%, transparent);
  display: grid;
  gap: 0.2rem;
}

.settings-panel {
  width: min(980px, 100%);
  padding: 1.1rem;
  color: var(--game-btn-text);
}

.settings-head {
  margin-bottom: 1rem;
}

.settings-grid {
  display: grid;
  gap: 0.9rem;
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.settings-select-trigger {
  border-color: var(--game-input-border);
  background: var(--game-input-bg);
  color: var(--game-btn-text);
}

.settings-select-trigger[data-placeholder] {
  color: var(--game-text-muted);
}

.settings-select-trigger :deep(svg) {
  color: color-mix(in oklab, var(--game-btn-text) 62%, transparent);
}

.settings-select-content {
  border-color: var(--game-panel-border);
  background: var(--game-panel-bg);
  color: var(--game-btn-text);
}

.settings-select-item {
  color: var(--game-btn-text);
}

.settings-select-item[data-highlighted] {
  background: color-mix(in oklab, var(--game-btn-primary-bg) 12%, var(--game-panel-bg) 88%);
  color: var(--game-btn-text);
}

.settings-select-item[data-state="checked"] {
  background: color-mix(in oklab, var(--game-btn-primary-bg) 18%, var(--game-panel-bg) 82%);
  color: var(--game-btn-text);
}

.settings-select-theme {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.7rem;
}

.settings-select-theme-name {
  font-size: 0.84rem;
}

.settings-select-swatch-row {
  display: inline-flex;
  gap: 0.25rem;
}

.settings-select-swatch {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 999px;
  border: 1px solid color-mix(in oklab, var(--game-panel-border) 78%, transparent);
}

.settings-select-swatch-panel {
  background: var(--game-panel-bg);
}

.settings-select-swatch-primary {
  background: var(--game-btn-primary-bg);
}

.settings-select-swatch-accent {
  background: var(--game-bg-layer-1);
}

.theme-preview-grid {
  margin-top: 1rem;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0.75rem;
}

.theme-preview-card {
  border: 1px solid var(--game-panel-border);
  border-radius: calc(var(--radius-lg) + 2px);
  padding: 0.68rem;
  background: color-mix(in oklab, var(--game-panel-bg) 96%, transparent);
  text-align: left;
  transition: border-color 140ms ease, transform 140ms ease, box-shadow 140ms ease;
}

.theme-preview-card:hover {
  transform: translateY(-2px);
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 46%, var(--game-panel-border) 54%);
  box-shadow: 0 10px 22px color-mix(in oklab, var(--game-overlay-bg) 25%, transparent);
}

.theme-preview-card-active {
  border-color: color-mix(in oklab, var(--game-btn-primary-bg) 62%, var(--game-panel-border) 38%);
  box-shadow: 0 0 0 2px color-mix(in oklab, var(--game-btn-primary-bg) 24%, transparent);
}

.theme-preview-surface {
  border: 1px solid var(--game-panel-border);
  border-radius: var(--radius-md);
  background:
    linear-gradient(180deg, color-mix(in oklab, var(--game-panel-bg) 88%, transparent), color-mix(in oklab, var(--game-bg-layer-1) 35%, transparent));
  padding: 0.5rem;
}

.theme-preview-header {
  display: flex;
  gap: 0.2rem;
  margin-bottom: 0.45rem;
}

.theme-preview-dot {
  width: 0.4rem;
  height: 0.4rem;
  border-radius: 999px;
  background: var(--game-btn-border);
}

.theme-preview-lines {
  display: grid;
  gap: 0.25rem;
}

.theme-preview-line {
  height: 0.23rem;
  border-radius: 999px;
  background: color-mix(in oklab, var(--game-btn-text) 70%, transparent);
  opacity: 0.45;
}

.theme-preview-line-short {
  width: 68%;
}

.theme-preview-cta {
  margin-top: 0.45rem;
  border-radius: 999px;
  background: var(--game-btn-primary-bg);
  color: var(--game-btn-primary-text);
  font-size: 0.56rem;
  width: fit-content;
  padding: 0.13rem 0.42rem;
  letter-spacing: 0.02em;
}

.theme-preview-meta {
  margin-top: 0.48rem;
}

.theme-preview-name {
  font-size: 0.84rem;
  font-weight: 600;
  color: var(--game-btn-text);
}

.theme-preview-desc {
  margin-top: 0.08rem;
  font-size: 0.72rem;
  color: color-mix(in oklab, var(--game-btn-text) 72%, transparent);
}

.game-text-muted {
  color: var(--game-text-muted);
}

@media (max-width: 900px) {
  .game-shell {
    min-height: auto;
    grid-template-columns: 1fr;
  }

  .game-state-content {
    overflow: visible;
    padding-right: 0;
  }

  .game-custom-row {
    flex-direction: column;
  }

  .game-custom-row .btn {
    width: 100%;
  }

  .new-game-shell {
    align-items: flex-start;
  }

  .new-game-layout {
    grid-template-columns: 1fr;
  }

  .new-game-panel {
    padding: 0.95rem;
  }

  .new-game-overview {
    padding: 0.9rem;
  }

  .new-game-actions {
    justify-content: stretch;
  }

  .new-game-actions .btn {
    flex: 1;
  }

  .saves-shell {
    align-items: stretch;
  }

  .saves-panel {
    padding: 0.85rem;
  }

  .saves-head {
    align-items: flex-start;
    flex-direction: column;
  }

  .saves-toolbar {
    align-items: stretch;
  }

  .saves-search {
    width: 100%;
  }

  .save-slot {
    grid-template-columns: 1fr;
    gap: 0.55rem;
  }

  .save-slot-actions {
    width: 100%;
  }

  .save-slot-actions .btn {
    flex: 1;
  }

  .settings-grid {
    grid-template-columns: 1fr;
  }

  .theme-preview-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
