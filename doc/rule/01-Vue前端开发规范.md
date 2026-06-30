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

```
pages/          → 路由入口，组合组件，零业务逻辑
components/     → UI 单元，接收 props、抛出 emit
stores/         → 跨组件共享状态 + 状态变更逻辑
services/       → 无状态的纯业务逻辑，与 Tauri 后端通信
utils/          → 纯函数，无副作用
```

禁止跨层调用：`services/` 不能 import `components/`，`pages/` 不能直接调用 Tauri invoke 等。

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

- **一个文件只导出一个组件**（默认导出）
- 单个 `.vue` 文件不超过 200 行，超出则拆分子组件
- 纯展示组件放在 `components/common/`，与业务无关
- SVN 业务组件放在 `components/svn/`，可跨页面复用

## Props / Emit 规范

- Props 必须定义类型和默认值，使用 `withDefaults`
- Emit 必须使用基于类型的声明（如上例）
- 禁止使用 `defineExpose` —— 用 props + emit 替代
- 禁止跨多级透传 props —— 使用 store 或 provide/inject

## 模板规范

```vue
<!-- v-for 必须绑定 key -->
<div v-for="item in list" :key="item.id">

<!-- v-if / v-else-if / v-else 必须连续，中间不插空行 -->
<div v-if="loading">加载中</div>
<div v-else-if="error">加载失败</div>
<div v-else>内容</div>

<!-- 事件处理使用函数名，不写内联表达式 -->
<button @click="handleSubmit">提交</button>
```

---

# 03-TypeScript 规范

- 启用 `strict: true`
- **禁止使用 `any`**。不确定的类型用 `unknown` 并做类型收窄
- 接口命名以 `I` 开头：`interface IRepoInfo { ... }`
- 枚举命名使用 PascalCase：`enum RepoStatus { ... }`
- 类型定义集中在 `types/` 目录，按模块拆分文件

```ts
// types/repo.ts
export interface IRepoInfo {
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

## 常用样式对照

| 场景 | Tailwind 类 |
|------|------------|
| 弹性布局 | `flex items-center justify-between` |
| 网格 | `grid grid-cols-3 gap-4` |
| 内边距 | `p-4` `px-6` `py-2` |
| 外边距 | `m-4` `mt-2` `mb-4` |
| 字体 | `text-sm` `text-lg` `font-bold` `font-mono` |
| 颜色 | `text-gray-600` `bg-white` `border-gray-200` |
| 圆角 | `rounded` `rounded-lg` `rounded-full` |
| 阴影 | `shadow` `shadow-md` `shadow-lg` |
| 悬停 | `hover:bg-gray-100` `hover:text-blue-600` |
| 过渡 | `transition-colors` `duration-200` |

## Element Plus 与 Tailwind 协作

- Element Plus 组件通过全局 class 覆盖调整样式（不修改组件内部）
- 高优先级覆盖：`class="!p-0"`（Tailwind 的 `!` 前缀生成 `!important`）
- 布局容器使用 Tailwind，具体表单/表格/对话框使用 Element Plus 组件

---

# 05-状态管理与路由

## Pinia 规范

```ts
// stores/repo.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useRepoStore = defineStore('repo', () => {
  const currentPath = ref('')
  const fileList = ref<IFileItem[]>([])

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
| 接口 | PascalCase + I 前缀 | `IRepoInfo` `ICommitLog` |
| 枚举 | PascalCase | `RepoStatus` `FileState` |
| 常量 | UPPER_SNAKE_CASE | `MAX_HISTORY_COUNT` |
| Pinia Store | useXxxStore | `useRepoStore` |

---

# 07-文件级编码约定

- 文件编码：UTF-8（无 BOM）
- 缩进：2 空格
- 行尾：LF
- 每行最长 100 字符
- 文件末尾保留一个空行
- import 顺序：Vue → 第三方库 → 内部模块，每组空行分隔
