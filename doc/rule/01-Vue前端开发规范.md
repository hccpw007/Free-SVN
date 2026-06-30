# 01-总则

## 技术栈

| 层面 | 技术选型 |
|------|----------|
| 框架 | Vue 3 (Composition API + `<script setup>`) |
| 语言 | TypeScript (strict mode) |
| 构建 | Vite 6 |
| UI 组件 | Element Plus 2.x |
| 样式 | Tailwind CSS 4（仅使用 Tailwind，禁止 `<style>` 标签和自定义 CSS 文件） |
| 路由 | Vue Router 4 |
| 状态管理 | Pinia 3 |
| 持久化 | Tauri Store Plugin（替代 localStorage） |
| 包管理 | cnpm |

## 项目结构

```
src/
├── assets/           # 静态资源（图片、字体等）
├── components/       # 通用可复用组件
│   ├── common/       #   跨业务通用组件（Modal、Table、Tree等）
│   └── svn/          #   SVN 业务专用组件
├── layouts/          # 布局组件（侧边栏、顶栏等整体布局）
├── pages/            # 页面级组件（一个目录一个页面，支持嵌套子页面）
│   └── HomePage.vue
├── stores/           # Pinia 状态仓库
├── services/         # 业务逻辑层（封装 Tauri invoke 调用、数据处理）
├── types/            # TypeScript 类型定义（接口、枚举、类型别名）
├── utils/            # 纯工具函数
├── router/           # 路由配置
│   └── index.ts
├── App.vue           # 根组件（仅放置 <RouterView />）
├── main.ts           # 应用入口
└── style.css         # Tailwind 入口（仅一行 @import "tailwindcss"）
```

## 分层职责

- `pages/` → 组合组件，禁止写业务逻辑
- `components/` → UI 单元，接收 props、抛出 emit
- `stores/` → 跨组件共享状态
- `services/` → 无状态业务逻辑，封装 Tauri invoke
- `utils/` → 纯函数

禁止：`services/` import `components/`；`pages/` 直接调用 Tauri invoke。

---

# 02-Vue 组件规范

## 组件编写

统一使用 `<script setup lang="ts">` 组合式 API，禁止 Options API。

```vue
<script setup lang="ts">
interface Props {
  repoPath: string
  readonly?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  readonly: false,
})

const emit = defineEmits<{
  select: [path: string]
  delete: [id: number]
}>()
</script>

<template>
  <div class="p-4">
    <!-- 仅使用 Tailwind -->
  </div>
</template>
```

## 组件粒度原则

- **一个文件只导出一个组件**
- `.vue` 文件超过 400 行则拆子组件
- 纯展示组件放 `components/common/`，SVN 业务组件放 `components/svn/`

## Props / Emit 规范

- Props 必须定义类型和默认值，使用 `withDefaults`
- Emit 必须使用基于类型的声明
- 禁止 `defineExpose` — 用 props + emit 替代
- 禁止跨多级透传 props — 用 store 或 provide/inject

## 模板规范

```vue
<div v-for="item in list" :key="item.id">

<div v-if="loading">加载中</div>
<div v-else-if="error">加载失败</div>
<div v-else>内容</div>

<button @click="handleSubmit">提交</button>
```

---

# 03-TypeScript 规范

- 启用 `strict: true`
- **禁止使用 `any`**。不确定的类型用 `unknown` 并做类型收窄
- 接口命名使用 PascalCase：`interface RepoInfo { ... }`
- 枚举命名使用 PascalCase：`enum RepoStatus { ... }`
- 类型定义集中在 `types/` 目录，按模块拆分文件

```ts
// types/repo.ts
export interface RepoInfo {
  path: string
  url: string
  status: RepoStatus
  lastCommit: string
}

export enum RepoStatus {
  Clean = 'clean',
  Modified = 'modified',
  Conflict = 'conflict',
}
```

---

# 04-Tailwind 样式规范

## 核心规则

1. **绝对不写 `<style>` 标签**
2. **不创建任何 `.css` 文件**（除了 `src/style.css` 的 Tailwind 入口）
3. 所有样式只用 Tailwind 原子类完成

## Element Plus 与 Tailwind 协作

- 布局容器用 Tailwind，表单/表格/对话框用 Element Plus 组件
- Element Plus 组件通过全局 class 覆盖调整样式
- 高优先级覆盖：`class="!p-0"`（Tailwind `!` 前缀生成 `!important`）

---

# 05-状态管理与路由

## Pinia 规范

```ts
// stores/repo.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useRepoStore = defineStore('repo', () => {
  const currentPath = ref('')
  const fileList = ref<FileItem[]>([])

  async function loadFiles(path: string) {
    // 调用 services/ 中的业务函数
  }

  return { currentPath, fileList, loadFiles }
})
```

- 使用 Setup Store 语法（函数形式），禁止 Options Store
- 一个 store 文件只包含一个 `defineStore`
- Store 只放**跨组件共享**的状态，组件私有状态就地管理

## 路由规范

```ts
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: () => import('@/layouts/MainLayout.vue'),
    children: [
      {
        path: '',
        name: 'home',
        component: () => import('@/pages/HomePage.vue'),
      },
      {
        path: 'log',
        name: 'log',
        component: () => import('@/pages/log/LogPage.vue'),
      },
    ],
  },
]
```

- **懒加载**所有页面组件
- 有布局嵌套时使用 children 路由
- 路由 path 使用 kebab-case

---

# 06-命名规范

| 类型 | 规范 | 示例 |
|------|------|------|
| 组件文件 | PascalCase | `RepoTree.vue` `CommitDialog.vue` |
| 页面文件 | PascalCase + 后缀 Page | `HomePage.vue` `LogPage.vue` |
| 目录名 | kebab-case | `svn-tree/` `commit-dialog/` |
| TS 文件 | kebab-case | `repo-store.ts` `svn-service.ts` |
| 变量/函数 | camelCase | `currentPath` `loadFiles()` |
| 接口 | PascalCase | `RepoInfo` `CommitLog` |
| 枚举 | PascalCase | `RepoStatus` `FileState` |
| 常量 | UPPER_SNAKE_CASE | `MAX_HISTORY_COUNT` |
| Pinia Store | useXxxStore | `useRepoStore` |

---

# 07-文件拆分时机

## 按文件类型约定

| 类型 | 行数警戒线 | 达到后如何拆分 |
|------|-----------|---------------|
| `.vue` 组件 | 400 行 | 提取子组件到 `components/`，提取逻辑到 `stores/` 或 `services/` |
| `stores/*.ts` | 150 行 | 按业务领域拆分为多个 store（如 `repoStore`、`commitStore`） |
| `services/*.ts` | 200 行 | 按功能拆分为多个 service 文件（如 `svn-status.ts`、`svn-commit.ts`） |
| `types/*.ts` | 100 行 | 按模块拆分（如 `types/repo.ts`、`types/commit.ts`） |
| `utils/*.ts` | 100 行 | 按功能拆分（如 `utils/format.ts`、`utils/path.ts`） |
| `router/index.ts` | 80 行 | 提取路由模块到 `router/modules/` 目录 |

## 拆分信号

出现以下情况立即拆分：

1. **一文件处理多个职责**（如同时做 status/commit/log）
2. **多处 import 同一文件的特定函数** — 属于独立模块
3. **修改一个功能需滚动 3 屏以上**

## 拆分方式

```
# 拆分前
services/svn-service.ts

# 拆分后
services/svn/
├── index.ts
├── status.ts
├── commit.ts
└── log.ts
```

原则：功能单元超过 3 个文件时建目录 + `index.ts` 统一导出。

---

# 08-死代码预防（AI 自动化）

> AI 自动执行，无需人工检查。

## 检测命令

| 检测项 | 命令 |
|--------|------|
| 未使用的 export/文件/组件 | `cnpm run knip` |
| 未使用的 import / 类型错误 | `cnpm run lint` |

## AI 清理流程（删除功能时）

1. 收到"删除 XXX"指令后，先删除入口文件（页面目录 / 路由配置）
2. 运行 `cnpm run lint && cnpm run knip`
3. 根据报告删除未引用的文件/导出
4. 重复步骤 2-3 直到零报告
5. 运行 `cnpm run build` 验证
6. 检查 `src-tauri/src/lib.rs` 中 `generate_handler!` 的残余命令

## 编码阶段规则

1. **不注释代码** — 不需要的直接删除，git 可找回
2. **重命名时即时删除旧文件**
3. **删除源文件后删除同名 `*.spec.ts` / `*.test.ts`**
4. **功能合并前运行 `cnpm run knip`**

---

# 09-文件级编码约定

- 文件编码：UTF-8（无 BOM）
- 缩进：2 空格
- 行尾：LF
- 每行最长 100 字符
- 文件末尾保留一个空行
- import 顺序：Vue → 第三方库 → 内部模块，每组空行分隔

---

# 10-测试策略

- 使用 Vitest + Vue Test Utils
- 测试文件与被测文件同级，命名 `*.test.ts`
- Store 测试：直接调用 action，验证 state
- Service 测试：测试纯逻辑函数（不涉及 Tauri invoke 的部分）

覆盖率目标：

| 类型 | 目标 |
|------|------|
| `utils/` | ≥ 90% |
| `services/` | ≥ 70% |
| 核心组件 | 覆盖 |
| 路由/页面 | E2E |
