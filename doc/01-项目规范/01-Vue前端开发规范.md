# Vue 前端开发规范

## 01-技术栈

| 层面 | 选型 |
|------|------|
| 框架 | Vue 3 (Composition API + `<script setup>`) + TypeScript strict |
| 构建 | Vite 6 |
| 样式 | Tailwind CSS 4（**禁止**写 `<style>` 标签和自定义 CSS） |
| UI | Element Plus 2.x |
| 路由 | Vue Router 4 (lazy load) |
| 状态 | Pinia (Setup Store 语法) |
| i18n | vue-i18n |
| 持久化 | tauri-plugin-store |

## 02-项目结构

```
src/
├── pages/         # 页面组件，组合子组件，不写业务逻辑
├── components/    # 通用组件 / SVN业务组件 / 对话框
├── composables/   # 带状态的复用逻辑（useXxx）
├── stores/        # 跨组件共享状态
├── services/      # 无状态业务逻辑（封装 Tauri invoke）
├── utils/         # 纯函数
├── types/         # 类型定义
├── locales/       # i18n 语言包
└── router/        # 路由配置
```

分层禁止：`services/` import `components/`；`pages/` 直接调用 Tauri invoke。

## 03-Vue 组件规范

- 统一 `<script setup lang="ts">`，禁止 Options API
- Props 必须定义类型和默认值（`withDefaults`）
- Emit 必须使用基于类型的声明
- 禁止 `defineExpose` — 用 props + emit 替代
- 禁止跨多级透传 props — 用 store 或 provide/inject
- 一个 .vue 文件只导出一个组件，超过 400 行则拆子组件

```vue
<script setup lang="ts">
interface Props { repoPath: string; readonly?: boolean }
const props = withDefaults(defineProps<Props>(), { readonly: false })
const emit = defineEmits<{ select: [path: string] }>()
</script>
```

## 04-状态管理（Pinia）

- 使用 Setup Store 函数语法，禁止 Options Store
- 一个文件只包含一个 `defineStore`
- Store 只放跨组件共享状态，组件私有状态就地管理
- 每个 Store 实现 `reset()` 方法用于切换工作副本时重置

## 05-错误处理（永不吞错误）

- 每个 `try` 必须有 `catch`，`catch` 必须输出日志
- 日志格式：`console.error('[ComponentName] 操作描述:', e)`
- 禁止 `.catch(() => {})`
- 允许静默的场景：剪贴板 API 失败、用户取消确认对话框（需注释说明）

## 06-样式规范

- 仅使用 Tailwind 原子类，**绝对不写 `<style>` 标签和 .css 文件**
- 仅 `src/style.css` 作为 Tailwind 入口（`@import "tailwindcss"` + `@theme` 自定义 Token）
- 暗色模式使用 `dark:` 前缀适配，禁止自定义 CSS 变量
- 使用 Element Plus 内置组件（`<el-table>`、`<el-dialog>`、`<el-button>` 等）

## 07-命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件文件 | PascalCase | `FileListTable.vue` |
| 页面文件 | PascalCase + Page 后缀 | `HomePage.vue` |
| 目录名 | kebab-case | `svn-tree/` |
| TS 文件 | kebab-case | `repo-store.ts` |
| 组合式函数 | useXxx | `useAutoRefresh` |
| 变量/函数 | camelCase | `currentPath` |
| 接口 | PascalCase | `RepoInfo` |
| 常量 | UPPER_SNAKE_CASE | `MAX_HISTORY_COUNT` |

## 08-文件拆分标准

| 类型 | 警戒线 | 拆分方式 |
|------|--------|----------|
| `.vue` | 400 行 | 提取子组件 + 逻辑到 composable/store |
| `stores/*.ts` | 150 行 | 按领域拆多个 store |
| `services/*.ts` | 200 行 | 按功能拆文件 |
| `composables/*.ts` | 100 行 | 按场景拆分 |
| `router/*.ts` | 80 行 | 提取 `router/modules/` |

拆分信号：一文件多职责 / 多处 import 同一文件的部分函数 / 修改需滚动 3 屏以上。

## 09-编码约定

- UTF-8 无 BOM，2 空格缩进，LF 行尾，最长 100 字符，末尾空行
- import 顺序：Vue → 第三方库 → 内部模块，每组空行分隔

## 10-死代码预防

> CLAUDE.md 已有最高优先级规则，此处仅说明工具：
> - 检测：`cnpm run knip`（未使用导出/文件）+ `cnpm run lint`（类型错误/import）
> - 删除功能时：删入口文件 → `lint + knip` → 删未引用 → 重复到零报告 → `build` 验证
> - 不注释代码（直接删），重命名时删旧文件
