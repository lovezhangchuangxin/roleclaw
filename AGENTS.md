# AGENTS.md

本文件是 `RoleClaw` 项目的统一协作规范。适用于参与本仓库的开发者与智能体。

## 1. 项目介绍

`RoleClaw` 是一款 AI 角色扮演游戏（AI RPG）桌面应用，核心卖点是：

1. 非固定剧本，支持高自由度叙事推进。
2. 基于世界卡（World Card）生成世界观、角色、地点与剧情钩子。
3. 对话采用 `3+1` 机制：三个系统选项 + 玩家自定义输入。
4. 地图探索（节点与路径）+ 事件系统驱动剧情演化。
5. 支持存档回放与分叉扩展。
6. 多端适配，游戏 UI 同时支持桌面端和移动端。

## 2. 技术栈

1. 桌面框架：`Tauri 2`
2. 后端语言：`Rust`
3. 前端框架：`Vue 3 + TypeScript + Vite`
4. UI：`Tailwind CSS + shadcn/vue`
5. 数据存储：本地 JSON / NDJSON 文件（应用数据目录）

## 3. 核心功能范围

1. 开始菜单与游戏内 `Esc` 菜单（统一封装）。
2. 新建存档、载入存档、删除存档。
3. 世界卡列表、导入、导出。
4. AI 设置（provider/model/参数）。
5. 游戏设置（主题切换、消息速度等）。
6. 地图移动、回合推进、对话交互。

## 4. 目录架构（当前）

```text
roleclaw/
|-- AGENTS.md
|-- docs/
|   |-- ai-rpg-prd.md
|   |-- mvp-task-breakdown.md
|   |-- data-schemas.md
|   `-- prompt-spec.md
|-- src/
|   |-- App.vue                      # 主页面编排与路由视图切换
|   |-- types.ts                     # 前后端共享类型（TS 侧）
|   |-- composables/
|   |   `-- useGameApp.ts            # 页面状态与动作逻辑
|   |-- components/
|   |   `-- game/
|   |       |-- GameSettingsMenu.vue # 开始菜单/ESC 菜单
|   |       |-- GameMapCanvas.vue    # 地图绘制与移动入口
|   |       `-- WorldCardManager.vue # 世界卡管理页面
|   |-- lib/
|   |   |-- api.ts                   # Tauri invoke API 封装
|   |   `-- errors.ts                # 前端错误规范化
|   `-- styles/
|       |-- main.css                 # 样式入口
|       `-- theme.css                # 多主题变量（default/fantasy/terminal/archive）
`-- src-tauri/
    |-- Cargo.toml
    `-- src/
        |-- lib.rs                   # Tauri 启动与 command 注册
        |-- commands.rs              # 命令层（I/O 边界）
        |-- domain.rs                # 领域数据结构
        |-- storage.rs               # 本地持久化读写
        |-- game.rs                  # 核心游戏逻辑
        |-- llm.rs                   # 模型 Provider 抽象层
        `-- error.rs                 # 统一错误模型（结构化错误）
```

## 5. 常用命令

在仓库根目录执行：

1. 前端包管理器优先使用 `pnpm`，包括安装、运行、构建命令。
2. 安装依赖：`pnpm install`
3. 前端开发：`pnpm dev`
4. 前端构建：`pnpm build`
5. Tauri 开发：`pnpm tauri dev`
6. Rust 检查：`cd src-tauri && cargo check`

## 6. 数据与存储规则

应用数据目录下使用以下布局：

1. `/game-data/saves/<save_id>/meta.json`
2. `/game-data/saves/<save_id>/snapshot.json`
3. `/game-data/saves/<save_id>/events.ndjson`
4. `/game-data/world-cards/<card_id>.json`

规则：

1. `snapshot.json` 作为当前状态真值。
2. `events.ndjson` 只追加，不覆盖历史。
3. 结构变更要同步更新 `docs/data-schemas.md`。

## 7. 代码架构规则

1. Command 层只做参数接收、调用服务、返回结果，不堆业务逻辑。
2. 游戏规则放在 `game.rs`（后续事件引擎也放此层或子模块）。
3. 文件读写统一走 `storage.rs`，避免散落 `fs` 调用。
4. 错误统一使用 `AppError`，禁止裸字符串错误扩散。
5. 新增 Provider 时只扩展 `llm.rs`，不破坏 command 接口。
6. 前端状态逻辑优先放 `composable`，页面组件以展示为主。

## 8. UI 与主题规则

1. 主题变量集中在 `src/styles/theme.css`。
2. 至少维护四套主题：`default/fantasy/terminal/archive`。
3. 游戏 UI 颜色优先使用主题变量，避免硬编码字面量颜色。
4. 新增组件样式时，优先复用：
   - `--game-panel-*`
   - `--game-btn-*`
   - `--game-input-*`
   - `--game-canvas-*`
5. UI 组件优先使用 `shadcn/vue` 现成组件，而不是优先自定义基础组件。
6. 新增 `shadcn/vue` 组件使用命令：
   `pnpm dlx shadcn-vue@latest add <component>`

## 9. 提交前检查清单

1. `pnpm build` 必须通过。
2. `cargo check` 必须通过。
3. 若涉及数据结构变更，需同步更新 `docs/` 文档。
4. 新增主题相关样式时，不得直接写死主要颜色值。

## 10. 约束说明

1. 当前为本地优先架构，不引入远程数据库。
2. 任何破坏存档格式的改动必须提供迁移策略。
3. 未经明确需求，不引入独立数值战斗系统。
