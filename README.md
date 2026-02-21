# RoleClaw

<div align="center">

[![GitHub release (latest by date)](https://img.shields.io/github/v/release/lovezhangchuangxin/roleclaw)](https://github.com/lovezhangchuangxin/roleclaw/releases)
[![GitHub stars](https://img.shields.io/github/stars/lovezhangchuangxin/roleclaw)](https://github.com/lovezhangchuangxin/roleclaw/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/lovezhangchuangxin/roleclaw)](https://github.com/lovezhangchuangxin/roleclaw/network)
[![GitHub issues](https://img.shields.io/github/issues/lovezhangchuangxin/roleclaw)](https://github.com/lovezhangchuangxin/roleclaw/issues)
[![GitHub license](https://img.shields.io/github/license/lovezhangchuangxin/roleclaw)](https://github.com/lovezhangchuangxin/roleclaw/blob/master/LICENSE)
[![CI](https://github.com/lovezhangchuangxin/roleclaw/actions/workflows/ci.yml/badge.svg)](https://github.com/lovezhangchuangxin/roleclaw/actions/workflows/ci.yml)

</div>

RoleClaw 是一个基于 Tauri 2 + Vue 3 + Rust 的 AI 角色扮演游戏（AI RPG）桌面应用。  
项目强调高自由度叙事、世界卡驱动内容生成、地图探索与存档分叉回放。

## 核心特性

- 🤖 非固定剧本叙事推进（AI 驱动）
- 🃏 世界卡（World Card）管理：导入、导出、编辑、AI 生成草稿
- 💬 3+1 对话机制：三个系统选项 + 玩家自定义输入
- 🗺️ 地图探索与事件触发
- 💾 存档系统：创建、载入、删除、分叉、回放时间线
- ⚙️ AI 模型配置：多模型管理、默认模型、连通性测试
- 🎨 多主题游戏 UI（`default` / `fantasy` / `terminal` / `archive`）

## 快速开始

### 前置要求

- Node.js 18+
- [pnpm](https://pnpm.io/) 9+
- Rust (stable)
- Tauri 2 构建所需系统依赖

#### macOS

```bash
brew install rustup
rustup-init
# 按照提示完成 Rust 安装
```

#### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev
```

#### Windows

安装 [Rust](https://rustup.rs/) 和 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)。

### 安装与运行

```bash
# 克隆项目
git clone https://github.com/lovezhangchuangxin/roleclaw.git
cd roleclaw

# 安装依赖
pnpm install

# 启动前端开发服务
pnpm dev

# 启动桌面应用（新终端）
pnpm tauri dev
```

### 构建发布版

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 技术栈

- **桌面端**: [Tauri 2](https://tauri.app/)
- **前端**: Vue 3 + TypeScript + Vite + Vue Router
- **UI**: Tailwind CSS + [shadcn/vue](https://github.com/radix-vue/shadcn-vue)（基于 reka-ui）
- **后端**: Rust
- **数据**: 本地 JSON / NDJSON

## 项目结构

```
roleclaw/
├─ .github/workflows/          # CI/CD 配置
├─ docs/                      # PRD / 数据结构 / Prompt 规范 / 任务拆分
├─ data/world-cards/           # 预设世界卡
├─ src/                       # 前端
│  ├─ views/                  # Menu / NewGame / GamePlay / Saves / Replay / Settings
│  ├─ composables/useGameApp.ts # 前端状态与业务编排
│  ├─ lib/api.ts              # Tauri invoke API 封装
│  └─ styles/                 # 主题与游戏样式
└─ src-tauri/                 # Rust + Tauri
   └─ src/
      ├─ commands.rs          # Tauri command 边界层
      ├─ game.rs              # 核心游戏逻辑
      ├─ storage.rs           # 本地存储读写
      ├─ llm.rs               # LLM provider 抽象
      └─ error.rs             # 统一错误模型
```

## 常用命令

```bash
# 前端类型检查 + 构建
pnpm build

# 前端预览构建结果
pnpm preview

# Rust 侧检查
cd src-tauri && cargo check
cd src-tauri && cargo fmt    # 格式化
cd src-tauri && cargo clippy # 代码检查
```

## 预设世界卡

项目内置以下世界卡模板：

| 模板 | 风格 | 描述 |
|------|------|------|
| `fantasy-kingdom` | 奇幻 | 中世纪魔法与冒险 |
| `cyberpunk-city` | 赛博朋克 | 高科技低生活都市 |
| `wuxia-world` | 武侠 | 刀光剑影的武林世界 |

## 应用数据目录

应用运行后会在 app data 目录下维护 `game-data`：

- `game-data/saves/<save_id>/meta.json` - 存档元信息
- `game-data/saves/<save_id>/snapshot.json` - 状态快照
- `game-data/saves/<save_id>/events.ndjson` - 事件日志
- `game-data/world-cards/<card_id>.json` - 世界卡数据

## 页面与路由

| 路由 | 页面 | 说明 |
|------|------|------|
| `/` | 菜单 | 主菜单 |
| `/new` | 新游戏 | 创建存档 |
| `/game` | 游戏中 | 游戏主界面 |
| `/saves` | 存档管理 | 存档列表 |
| `/replay` | 回放/分叉 | 历史回放 |
| `/cards` | 世界卡管理 | 世界卡编辑 |
| `/ai-settings` | AI 设置 | 模型配置 |
| `/settings` | 游戏设置 | 主题等 |

## 开发约定

- Command 层只做参数接收与调用，不堆业务逻辑
- 游戏规则优先放 `src-tauri/src/game.rs`
- 文件读写统一走 `src-tauri/src/storage.rs`
- 错误统一使用 `AppError`
- 前端状态逻辑优先放 `composables`
- 样式优先使用主题变量，避免硬编码主色

详细规范请查看：[AGENTS.md](./AGENTS.md)

## 相关文档

- [AI RPG PRD](./docs/ai-rpg-prd.md)
- [任务拆解](./docs/mvp-task-breakdown.md)
- [数据结构](./docs/data-schemas.md)
- [Prompt 规范](./docs/prompt-spec.md)

## 贡献

欢迎贡献！请阅读 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解如何参与开发。

## 许可证

[MIT](./LICENSE) © lovezhangchuangxin
