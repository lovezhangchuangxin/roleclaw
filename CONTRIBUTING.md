# 贡献指南

感谢你对 RoleClaw 项目的兴趣！我们欢迎各种形式的贡献，包括但不限于：

- 🐛 报告 Bug
- 💡 提出新功能或改进建议
- 📝 完善文档
- 💻 提交代码修复或新功能
- 🌍 翻译文档

## 开发环境准备

### 前置要求

- Node.js 18+
- pnpm 9+
- Rust stable
- Tauri 2 构建所需系统依赖

### 本地开发

```bash
# 克隆项目
git clone https://github.com/lovezhangchuangxin/roleclaw.git
cd roleclaw

# 安装依赖
pnpm install

# 启动前端开发服务器
pnpm dev

# 启动 Tauri 开发模式（新终端）
pnpm tauri dev
```

### 代码规范

#### Rust (后端)

- 遵循 Rust 官方格式化风格 (`cargo fmt`)
- 使用 Clippy 进行代码检查
- 所有公开 API 必须有文档注释
- 错误处理使用 `AppError`

```rust
/// 创建新存档
#[tauri::command]
pub async fn create_save(config: CreateSaveConfig) -> Result<SaveMeta, AppError> {
    // ...
}
```

#### Vue + TypeScript (前端)

- 使用 Vue 3 Composition API
- 组件文件使用 `<script setup>` 语法
- 类型定义优先使用 TypeScript
- 使用 shadcn/vue 组件库

```vue
<script setup lang="ts">
const props = defineProps<{
  title: string
}>()

const emit = defineEmits<{
  (e: 'update', value: string): void
}>()
</script>
```

## 项目结构

```
roleclaw/
├── .github/workflows/     # CI/CD 配置
├── docs/                 # 项目文档
├── src/                  # Vue 前端
│   ├── components/       # Vue 组件
│   ├── composables/     # 组合式函数
│   ├── views/           # 页面视图
│   ├── lib/             # 工具函数
│   └── types/           # TypeScript 类型
└── src-tauri/           # Rust 后端
    └── src/
        ├── commands.rs  # Tauri 命令
        ├── game.rs      # 游戏逻辑
        ├── storage.rs  # 存储层
        └── llm.rs      # LLM 集成
```

## 提交 Pull Request

### 分支命名规范

- `fix/` - Bug 修复 (e.g., `fix/issue-123`)
- `feat/` - 新功能 (e.g., `feat/add-world-card-import`)
- `docs/` - 文档更新
- `refactor/` - 代码重构
- `chore/` - 维护性更新

### 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型 (type):
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档
- `style`: 格式调整
- `refactor`: 重构
- `test`: 测试
- `chore`: 维护

示例:

```
feat(world-card): 添加世界卡导入功能

- 支持 JSON 格式世界卡导入
- 增加 schema 版本校验
- 添加导入错误提示

Closes #123
```

### PR 检查清单

- [ ] 代码符合格式化规范
- [ ] 通过 Clippy 检查（Rust）
- [ ] 通过类型检查（TypeScript）
- [ ] 相关功能已添加测试
- [ ] 更新了相关文档
- [ ] 提交信息清晰明确

## 问题和讨论

- 使用 GitHub Issues 报告 Bug
- 使用 GitHub Discussions 进行讨论
- 请确保在报告 Bug 时包含复现步骤

## 许可证

贡献本项目即表示你同意其 MIT 许可证。
