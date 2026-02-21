# RoleClaw

RoleClaw 是一个基于 Tauri 2 + Vue 3 + Rust 的 AI 角色扮演游戏（AI RPG）桌面应用。  
项目强调高自由度叙事、世界卡驱动内容生成、地图探索与存档分叉回放。

## 核心特性

- 非固定剧本叙事推进（AI 驱动）
- 世界卡（World Card）管理：导入、导出、编辑、AI 生成草稿
- 3+1 对话机制：三个系统选项 + 玩家自定义输入
- 地图探索与事件触发
- 存档系统：创建、载入、删除、分叉、回放时间线
- AI 模型配置：多模型管理、默认模型、连通性测试
- 多主题游戏 UI（`default` / `fantasy` / `terminal` / `archive`）

## 技术栈

- Desktop: `Tauri 2`
- Frontend: `Vue 3` + `TypeScript` + `Vite` + `Vue Router`
- UI: `Tailwind CSS` + `shadcn/vue`（基于 `reka-ui`）
- Backend: `Rust`
- Data: 本地 JSON / NDJSON（应用数据目录）

## 目录结构

```text
roleclaw/
├─ docs/                        # PRD / 数据结构 / Prompt 规范 / 任务拆分
├─ src/                         # 前端
│  ├─ views/                    # Menu / NewGame / GamePlay / Saves / Replay / Settings
│  ├─ composables/useGameApp.ts # 前端状态与业务编排
│  ├─ lib/api.ts                # Tauri invoke API 封装
│  └─ styles/                   # 主题与游戏样式
└─ src-tauri/                   # Rust + Tauri
   └─ src/
      ├─ commands.rs            # Tauri command 边界层
      ├─ game.rs                # 核心游戏逻辑
      ├─ storage.rs             # 本地存储读写
      ├─ llm.rs                 # LLM provider 抽象
      └─ error.rs               # 统一错误模型
```

## 环境要求

- Node.js 18+
- `pnpm`
- Rust（stable）
- Tauri 2 构建所需系统依赖（按你的 OS 安装）

## 快速开始

1. 安装依赖

```bash
pnpm install
```

2. 启动前端开发服务

```bash
pnpm dev
```

3. 启动桌面应用（Tauri）

```bash
pnpm tauri dev
```

## 常用命令

```bash
# 前端类型检查 + 构建
pnpm build

# 前端预览构建结果
pnpm preview

# Rust 侧检查
cd src-tauri && cargo check
```

## 应用数据目录与存档格式

应用运行后会在 app data 目录下维护 `game-data`，核心文件布局如下：

- `game-data/saves/<save_id>/meta.json`
- `game-data/saves/<save_id>/snapshot.json`
- `game-data/saves/<save_id>/events.ndjson`
- `game-data/world-cards/<card_id>.json`

规则约定：

- `snapshot.json` 是当前状态真值
- `events.ndjson` 只追加，不覆盖历史
- 若变更数据结构，请同步更新 `docs/data-schemas.md`

## 页面与路由

当前主视图包含：

- 菜单：`/`
- 新游戏：`/new`
- 游戏中：`/game`
- 存档管理：`/saves`
- 回放/分叉：`/replay`
- 世界卡管理：`/cards`
- AI 设置：`/ai-settings`
- 游戏设置：`/settings`

## 开发约定（简版）

- Command 层只做参数接收与调用，不堆业务逻辑
- 游戏规则优先放 `src-tauri/src/game.rs`
- 文件读写统一走 `src-tauri/src/storage.rs`
- 错误统一使用 `AppError`
- 前端状态逻辑优先放 `composables`
- 样式优先使用主题变量，避免硬编码主色

详细规范请查看：`AGENTS.md`

## 相关文档

- `docs/ai-rpg-prd.md`
- `docs/mvp-task-breakdown.md`
- `docs/data-schemas.md`
- `docs/prompt-spec.md`
