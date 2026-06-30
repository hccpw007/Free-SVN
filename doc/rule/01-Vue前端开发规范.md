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
- 单个 `.vue` 文件不超过 400 行，超出则拆分子组件
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

不只看行数，出现以下情况也应立即拆分：

1. **职责混杂** — 一个文件同时处理 SVN 状态获取、提交记录解析、文件过滤 → 拆
2. **多处 import 同一文件的特定函数** — 说明这些函数属于独立模块
3. **多人同时编辑同一文件的不同功能** — 说明文件太大，应该按功能拆分
4. **修改一个功能需要滚动 3 屏以上** — 说明文件可读性已下降，需提取方法或模块

## 拆分方式

```
# 拆分前
services/svn-service.ts        ← 300 行，包含 status/commit/log/diff

# 拆分后
services/svn/
├── index.ts                   # 统一导出
├── status.ts                  # svn status 相关
├── commit.ts                  # svn commit 相关
└── log.ts                     # svn log 相关
```

原则：**拆出目录而不是拆出平铺文件**。功能单元超过 3 个文件时建目录，用 `index.ts` 统一导出。

---

# 08-死代码预防（AI 自动化）

> 本项目为 AI 编程模式，死代码清理由 AI 自动执行，无需人工检查。

## 检测工具链

| 检测项 | 工具 | 命令 |
|--------|------|------|
| 未使用的 export/文件/组件 | `knip` | `cnpm run knip` |
| 未使用的 import / 类型错误 | vue-tsc | `cnpm run lint` |
| 未使用的 Tauri Command | 自定义脚本 knip.json | 内置在 knip 配置中 |

## AI 自动清理流程

删除一个功能时，AI 必须按以下顺序执行，不可跳过任何一步：

### 步骤一：上游入口删除（由开发者指令触发）

```bash
# AI 收到"删除 XXX 功能"指令后，先删除入口点
rm -rf src/pages/xxx/          # 页面目录
src/router/index.ts            # 删除对应路由配置
```

### 步骤二：自动递归清理

删除入口后立即运行检测工具，让工具报告哪些代码不再被引用：

```bash
cnpm run lint                    # 类型检查 → 识别未使用的 import
cnpm run knip                    # 识别未使用的 export / 文件 / 组件
```

根据 knip/vue-tsc 输出**自动删除**未被引用的文件：

```bash
# knip 输出例如: "src/services/xxx.ts" → 未使用
# AI 自动执行:
rm src/services/xxx.ts
# 继续运行 knip，直到零报告
```

### 步骤三：迭代清除 — 循环直到清白

```bash
while knip 报告还有未使用代码; do
  1. 读取 knip 报告，定位未使用的文件/导出
  2. 删除对应文件或 export
  3. 重新运行 `cnpm run knip`
done
```

### 步骤四：全量验证

```bash
cnpm run lint       # 零错误 → 通过
cnpm run build      # 构建通过
```

### 步骤五：同步后端

前端清理完成后，自动检查 Rust 端残余：

1. 检索 `src-tauri/src/lib.rs` 中 `generate_handler!` 列表
2. 检查 `commands/` 目录中每个命令是否被前端（`services/`）通过 `invoke` 调用
3. 发现孤立命令 → 通知开发者后端侧也需清理，或由 Rust 规范中的死代码流程自动处理

## 死代码预防（编码阶段）

AI 在编码过程中遵循以下规则避免产生死代码：

1. **不注释代码块** — 不需要的代码直接删除，git 历史可找回
2. **重命名/重构时即时清理旧文件** — 重命名组件后立即删除旧的 `.vue` 文件
3. **测试文件同步** — 删除源文件后检查同名 `*.spec.ts` / `*.test.ts` 是否也需要删除
4. **功能分支合并前** — 在 PR 分支上运行 `cnpm run knip`，确保没有引入死代码

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

## 单元测试

- 使用 Vitest + Vue Test Utils 编写测试
- **测试文件与被测文件保持同级**，以 `.test.ts` 后缀命名
- 组件测试：验证 props 渲染、emit 是否触发、插槽内容
  ```ts
  // components/RepoTree.test.ts
  import { mount } from '@vue/test-utils'
  import { describe, it, expect } from 'vitest'
  import RepoTree from './RepoTree.vue'

  it('renders repo path', () => {
    const wrapper = mount(RepoTree, { props: { path: '/test' } })
    expect(wrapper.text()).toContain('/test')
  })
  ```
- Store 测试：直接调用 store action，验证 state 变化
- Service 测试：测试纯业务逻辑函数（不涉及 Tauri invoke 的部分）

## 测试覆盖率目标

| 类型 | 目标 |
|------|------|
| 工具函数（`utils/`） | ≥ 90% |
| Service 逻辑（`services/`） | ≥ 70% |
| 组件渲染 | 核心组件覆盖 |
| 路由/页面级 | E2E
