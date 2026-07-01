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
├── assets/               # 静态资源（图片、字体、图标等）
├── components/           # 通用可复用组件 + 对话框组件
│   ├── common/           #   跨业务通用组件
│   ├── svn/              #   SVN 业务专用组件
│   └── dialogs/          #   模态对话框组件
├── composables/          # 组合式函数（带状态的复用逻辑）
├── layouts/              # 布局组件
├── pages/                # 页面级组件
├── stores/               # Pinia 状态仓库
├── services/             # 业务逻辑层（封装 Tauri invoke）
├── types/                # TypeScript 类型定义
├── utils/                # 纯工具函数
├── locales/              # i18n 语言包
├── router/               # 路由配置
├── fonts/                # JetBrains Mono 字体文件（打包到 Tauri resources）
├── App.vue               # 根组件（仅 <RouterView />）
├── main.ts               # 应用入口
└── style.css             # Tailwind 入口 + @theme Token + @font-face
```

**组件目录树**（与交互设计文档一致）：

```
layouts/
└── MainLayout.vue               # TopBar + ToolBar + RouterView + StatusBar

pages/
├── HomePage.vue                  # 欢迎页/变更文件列表（空状态切换）
├── CommitPage.vue                # 提交面板
├── LogPage.vue                   # 日志视图
├── SettingsPage.vue              # 设置页

components/common/
├── TopBar.vue                    # 顶部状态栏（路径/切换/分支 + 暗色切换）
├── ToolBar.vue                   # SVN 操作工具栏
├── StatusBar.vue                 # 底部状态栏

components/svn/
├── FileListTable.vue             # 变更文件表格（含全选+批量+搜索/筛选+排序）
├── FileStatusIcon.vue            # 文件状态图标（M/✚/!/🗑/?/-）
├── FileActionButtons.vue         # 操作按钮列
├── DiffViewer.vue                # diff2html 差异查看器（底部面板，可路由）
├── CommitForm.vue                # 提交信息输入区
├── LogTable.vue                  # 日志列表（含展开详情）
├── LogDetail.vue                 # 单条日志详情
├── ProgressOverlay.vue           # 操作进度遮罩（含 [取消] 按钮）

components/dialogs/
├── CheckoutDialog.vue            # 检出对话框
├── SwitchDialog.vue              # 切换分支对话框
├── BranchTagDialog.vue           # 分支/标签创建对话框
├── MergeWizard.vue               # 合并向导（4 步）
├── ResolveConflict.vue           # 冲突解决对话框
├── IgnoreDialog.vue              # 忽略确认对话框
├── ConfirmDialog.vue             # 通用确认对话框
├── ExportDialog.vue              # 导出对话框
└── UpdateRevisionDialog.vue      # 更新到版本对话框
```

## 路由设计

```
/                           → 重定向到 /workspace
/workspace                  → 变更文件列表/欢迎页
/workspace/commit           → 提交面板
/workspace/log              → 日志页面
/workspace/diff?file=xxx    → 差异对比（可路由）
/workspace/resolve?file=xxx → 冲突解决
/settings                   → 设置页面
```

- 所有 `workspace` 前缀的路由共享 MainLayout（顶栏+工具栏+底栏）
- `settings` 页面独立布局
- 对话框操作不占用路由：检出/分支标签/合并向导/切换/更新到版本 使用模态对话框

## 分层职责

- `pages/` → 组合组件，禁止写业务逻辑
- `components/` → UI 单元，接收 props、抛出 emit
- `components/dialogs/` → 模态对话框，使用 `<el-dialog>` 实现
- `composables/` → 组合式函数，封装带状态的复用逻辑
- `stores/` → 跨组件共享状态
- `services/` → 无状态业务逻辑，封装 Tauri invoke
- `utils/` → 纯函数

禁止：`services/` import `components/`；`pages/` 直接调用 Tauri invoke。

---

# 01.1-设计系统（Design Token）

## 色彩体系

使用 Tailwind 内置色板，**禁止**在 `<style>` 或自定义 CSS 中定义 CSS 变量。Token 通过 `src/style.css` 的 `@theme` 指令声明。

```css
/* style.css — 仅允许的全局样式入口 */
@import "tailwindcss";
@theme {
  --color-primary: #22C55E;   /* green-500 */
  --color-danger: #EF4444;    /* red-500 */
  --color-warning: #F59E0B;   /* amber-500 */
}
```

**亮色模式：**

| 角色 | Tailwind | 用途 |
|------|----------|------|
| 主要色 | `slate-800` | 顶部栏、活跃状态 |
| 品牌色 | `green-500` | 提交按钮、SVN 状态指示 |
| 背景色 | `slate-50` | 主背景 |
| 卡片色 | `white` | 文件列表、对话框 |
| 前景色 | `slate-900` | 主要文字 |
| 次要文字 | `slate-500` | 辅助信息 |
| 边框色 | `slate-200` | 分割线、表格边框 |
| 危险色 | `red-500` | 删除、冲突警告 |
| 警告色 | `amber-500` | 注意提示 |

**暗色模式：** 使用 `dark:` 前缀覆盖，暗色使用 `slate-900` 背景 / `slate-800` 卡片 / `slate-50` 前景 / `slate-400` 次要文字。

## 文件状态色

| 状态 | 亮色 | 暗色 |
|------|------|------|
| 已修改 (M) | `blue-500` | `blue-400` |
| 新增 (✚) | `green-500` | `green-400` |
| 冲突 (!) | `red-500`（加粗+左侧3px红色指示条） | `red-400` |
| 删除 (🗑) | `amber-500` | `amber-400` |
| 未加入 (?) | `slate-400` | `slate-500` |
| 锁定 (-) | `violet-500` | `violet-400` |

## 字体排版

| 层次 | Tailwind 类 | 字体栈 | 用法 |
|------|-------------|--------|------|
| Display | `text-3xl font-bold` | `'Inter', system-ui, sans-serif` | 欢迎页标题 |
| H1 | `text-xl font-semibold` | 同上 | 页面标题 |
| H2 | `text-base font-semibold` | 同上 | 面板标题 |
| Body | `text-sm` | 同上 | 正文、文件列表 |
| Body Small | `text-sm` | 同上 | 次要文字、状态栏 |
| Mono | `text-sm font-mono` | `'JetBrains Mono', monospace` | 文件路径、diff、版本号 |
| Mono Small | `text-xs font-mono` | 同上 | diff 行内文字 |
| Label | `text-sm font-medium` | `'Inter', system-ui, sans-serif` | 按钮、标签 |
| Tooltip | `text-xs` | 同上 | 工具提示 |

**字体加载：**
- Inter：使用系统字体栈（`system-ui, -apple-system`），不额外加载
- JetBrains Mono：打包字体文件到 `resources/fonts/`，`style.css` 中 `@font-face` 加载

## 间距系统

使用 Tailwind 内置间距类（4px 基准）：`p-1`(4px) / `p-2`(8px) / `p-3`(12px) / `p-4`(16px) / `p-5`(20px) / `p-6`(24px) / `p-8`(32px)

## 圆角与阴影

| 层级 | Tailwind 类 |
|------|-------------|
| 小型控件 | `rounded-sm` |
| 按钮、对话框 | `rounded-md` |
| 卡片、面板 | `rounded-lg` |
| 卡片阴影（亮色） | `shadow-sm` |
| 对话框阴影 | `shadow-md` |
| 模态遮罩层 | `shadow-lg`（暗色 `dark:shadow-none` + 边框替代）|

## 图标

- 使用 **Lucide** 图标库，统一 1.5px 描边
- 尺寸：16px（内联）/ 20px（按钮）/ 24px（状态指示）
- **禁止使用 emoji 作为功能图标**

## 动画基础

| 场景 | 持续时间 | 缓动 |
|------|---------|------|
| 悬停状态 | 150ms | ease-out |
| 面板展开/收起 | 200ms | ease-in-out |
| 对话框出现 | 200ms | ease-out + scale(0.95→1) |
| toast | 300ms | ease-out |
| 列表刷新 | 150ms | ease-out |
| 进度条 | 持续 | linear |

- 使用 Tailwind `transition-{property}` 和 `duration-{n}` 实现
- 尊重系统 `prefers-reduced-motion`，动画用 `motion-safe:` 前缀

---

# 01.2-Element Plus 组件使用约定

| 组件 | 场景 | 定制 |
|------|------|------|
| `<el-table>` | 文件列表、日志表 | `size="small"` 紧凑模式，行高 40px；10000+ 行时使用 `<el-table-v2>` 虚拟滚动表格 |
| `<el-dialog>` | 所有对话框 | 默认支持遮罩点击关闭；**操作进行中的对话框禁止遮罩关闭** |
| `<el-input>` | 输入框 | `size="default"` |
| `<el-select>` | 下拉选择 | 全局使用 |
| `<el-button>` | 按钮 | 主操作 `type="primary"`（映射为 green）|
| `<el-message>` | toast | 成功/错误/警告三类 |
| `<el-progress>` | 进度条 | 线性样式 |
| `<el-switch>` | 开关（设置页） | — |
| `<el-tooltip>` | 禁用解释 | `effect="dark"` |

Element Plus 主题色在 `style.css` 的 `@theme` 中覆盖，不修改 Element Plus 源码。

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

# 04.1-暗色模式

- **默认跟随系统**：通过 CSS `prefers-color-scheme` media query（Tailwind 的 `dark:` 变体）
- **手动切换**：TopBar 右侧提供切换按钮，选择持久化到 `tauri-plugin-store`
- **仅使用 `dark:` 前缀**：所有暗色适配只通过 Tailwind `dark:` 变体实现，禁止自定义 CSS 变量
- 禁止在 `<style>` 标签中写暗色样式

```html
<!-- 正确 -->
<div class="bg-white dark:bg-slate-800 text-slate-900 dark:text-slate-50">
```

**暗色特殊处理：**

| 元素 | 暗色适配 |
|------|---------|
| Diff 删除行 | `dark:bg-red-900/30` |
| Diff 新增行 | `dark:bg-green-900/30` |
| 对话框遮罩 | `dark:bg-black/60` |
| 阴影 | `dark:shadow-none` + 边框替代 |
| 输入框 | `dark:bg-slate-800 dark:border-slate-600` |

---

# 04.2-交互模式

## 行状态

| 模式 | 亮色 | 暗色 |
|------|------|------|
| hover | `bg-slate-50` | `dark:bg-slate-700/50` |
| 选中 | `bg-blue-50` | `dark:bg-blue-900/20` |
| 冲突行指示 | 左侧 3px `border-l-3 border-red-500` | 同左 |

## 操作按钮颜色

| 按钮 | 颜色 |
|------|------|
| [差异] | `text-blue-600`，hover 下划线 |
| [还原] | `text-amber-600`，hover 下划线 |
| [忽略] | `text-slate-500`，hover 下划线 |
| [合并] | `text-red-600`，hover 下划线 |
| [删除] | `text-red-500`，hover 下划线 |
| [解锁] | `text-violet-600`，hover 下划线 |

所有操作按钮 hover 均显示下划线 + 对应颜色加深。

## 文件路径溢出

- 文件路径列使用 `truncate` 类（`text-overflow: ellipsis`），过长时末尾省略
- 鼠标悬停显示完整路径 tooltip
- TopBar 中的工作副本路径采用中间省略（保留首尾路径段）

## 多选交互

- `Shift+Click`：范围选择（点击起始行，Shift+点击结束行）
- `Ctrl+Click` / `Cmd+Click`：切换单行选中状态（macOS 用 Cmd）
- 复选框操作等价于点击该行切换选中

## 排序

- 点击列头（状态/文件）按该列排序，再次点击切换升序/降序
- 排序指示器使用 `text-blue-500` 的 ▲/▼ 图标
- 默认按文件状态分组排序：冲突 → 已修改 → 新增 → 其他
- 日志表的版本列默认降序

## 搜索与筛选

- 搜索框输入使用 **300ms 防抖** 后过滤
- 筛选切换即时生效
- 搜索/筛选状态下显示"清除过滤"按钮
- 搜索框 placeholder 明确告知搜索能力边界

## 工具栏溢出

窗口宽度不足时，超出按钮自动折叠到末尾的 `···` 更多菜单中。溢出阈值动态计算，至少保留"提交"和"设置"两个按钮在可见区域。禁用态使用 `opacity-50 cursor-not-allowed`。

## 全选底栏粘性

底栏（"全选/已选 N 个文件/批量还原"）固定在表格底部，不随列表内容滚动。表格高度不足以显示底栏时仍固定在表格底部边界。

## 状态栏可交互性

- 点击版本号 → 复制 revision 号到剪贴板
- 鼠标悬停状态栏项 → 显示 tooltip 完整信息

---

# 04.3-状态与反馈

## 加载状态

| 场景 | 反馈 | 实现 |
|------|------|------|
| 首次加载文件列表 | 骨架屏/加载遮罩 | `<el-table v-loading>` |
| 提交中 | 按钮 loading + 进度条 | `<el-button loading>` + `<el-progress>` |
| 更新/检出中 | ProgressOverlay（可取消） | 全屏半透明 + 进度 + 阶段文字 + [取消] 按钮 |
| 差异加载中 | diff 区 skeleton | 灰色矩形脉冲动画 |
| 日志加载中 | 行 skeleton | 3-5 行灰色条脉冲动画 |

## 空状态

| 场景 | 说明 | 区别 |
|------|------|------|
| 欢迎页 | 首次启动/无最近工作副本 | 显示 [从仓库检出] 和 [打开已有工作副本] 按钮 |
| 工作副本无变更 | 已关联工作副本，无未提交变更 | 显示 ✅ "工作副本无变更" + [刷新] [查看日志] [切换分支] |
| 搜索/筛选无结果 | 搜索结果为空 | 显示 "未找到匹配的文件" + [清除过滤] |

## 空状态实现

- 欢迎页（无工作副本时）：不显示 ToolBar 和 StatusBar（MainLayout 自动隐藏子组件）
- 导航至变更列表视图时自动判断：有最近工作副本 → 直接进入主视图；否则 → 欢迎页
- 搜索/筛选空结果与"工作副本无变更"是两种不同的空状态，必须分别处理

## Toast 通知

| 类型 | 颜色 | 行为 |
|------|------|------|
| 成功 | `green-500` | 3s 自动关闭 |
| 错误 | `red-500` | 持续显示，手动关闭 |
| 警告 | `amber-500` | 持续显示，手动关闭 |

使用 Element Plus `<el-message>` 组件实现。所有 toast 默认 3s 自动关闭，错误和警告类型持续显示直到用户手动关闭。

## 系统通知

后台操作（更新/清理）完成时通过系统通知反馈（macOS Notification Center / Windows Toast / Linux notify-send）。用户点击通知 → 激活主窗口。

## 操作反馈对照表

参见交互设计文档 §4.2 操作反馈表，编码时按对应关系实现 toast/系统通知/确认对话框。

## 取消按钮交互

- ProgressOverlay 的 [取消] 按钮点击后 → 按钮变为"正在取消..."（禁用态）
- 取消完成后 → ProgressOverlay 关闭 → toast 展示结果（如"检出已取消"）

## 操作进行中全局禁用

ProgressOverlay 展示期间：
- 所有工具栏按钮禁用（tooltip "操作进行中，请等待"）
- 文件行操作按钮禁用
- 键盘快捷键不生效（Esc 和取消按钮除外）

---

# 04.4-键盘快捷键

> **平台适配：** 所有 `Ctrl+` 快捷键在 macOS 上自动映射为 `Cmd+`。实现时使用 Tauri 的 `CmdOrCtrl` 修饰键。下表中统一写作 `Ctrl+`。

| 快捷键 | 操作 | 作用域 |
|--------|------|--------|
| `Ctrl+Enter` | 提交 | 提交面板 |
| `Ctrl+D` | 打开差异 | 全局 |
| `Ctrl+R` | 刷新文件列表 | 全局 |
| `Ctrl+,` | 打开设置 | 全局 |
| `Ctrl+W` | 关闭当前面板 | 全局 |
| `Esc` | 关闭对话框/面板 | 全局 |
| `Ctrl+F` | 在 diff 内容中搜索 | Diff 面板 |
| `Ctrl+Down` | 下一个差异块 | Diff 面板 |
| `Ctrl+Up` | 上一个差异块 | Diff 面板 |

**Diff 面板键盘导航：** `Ctrl+Down` / `Ctrl+Up` 导航差异块；`Ctrl+W`/`Esc` 关闭面板。

**快捷键优先级规则（从高到低）：**
1. 对话框内部（`Enter` 确认、`Esc` 关闭）
2. 面板内部（`Ctrl+F` diff 搜索、`Ctrl+Down/Up` 导航）
3. 全局（`Ctrl+D` 打开差异、`Ctrl+R` 刷新）
4. ProgressOverlay 展示期间：除 `Esc` 外所有快捷键不生效

---

# 04.5-对话框规范

所有对话框使用 `<el-dialog>` 组件。

**入场动画：** 200ms scale(0.95→1) + fade，背景半透明遮罩（`dark:bg-black/60`）。

**遮罩点击行为：**
- 默认：支持遮罩点击关闭
- **操作进行中的对话框（检出中、合并执行中）禁止遮罩点击关闭**，必须通过 [取消] 按钮或操作完成后自动关闭

**必填标记：** 必填输入项右侧显示 `*`。

**确认对话框：** 所有破坏性操作（还原/删除/切换分支等）必须使用确认对话框。确认按钮使用 `type="danger"` 样式。

**对话框互斥规则：**
- ProgressOverlay 展示期间，禁止打开任何对话框
- 同一时间只允许一个模态对话框打开（`el-dialog` 默认行为）
- 检出/切换/合并向导/更新到版本 互斥（均涉及写操作）
- 通用确认对话框（ConfirmDialog）不与操作对话框互斥，可在操作对话框之上叠加

---

# 04.6-无障碍设计

| 要求 | 实现 |
|------|------|
| 对比度 | 文字 4.5:1 最小对比度（亮色/暗色均满足）|
| 键盘导航 | 所有操作均支持键盘完成（见 §04.4 快捷键）|
| 焦点指示 | 2px `blue-400` outline 在所有可交互元素上 |
| 颜色非唯一指示 | 文件状态使用颜色 + 图标双通道传递 |
| 屏幕阅读器 | `<el-table>` 默认支持 ARIA；图标按钮加 `aria-label` |
| 动画尊重 | `prefers-reduced-motion` 时禁用非必要动画 |
| 文本缩放 | 使用 Tailwind 类而非 px 硬编码 |

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

### Store 状态接口定义

每个 Store 必须定义明确的状态接口。四个 Store 的最小状态结构：

```ts
// stores/workspace.ts — 工作副本状态
export const useWorkspaceStore = defineStore('workspace', () => {
  const currentPath = ref<string>('')           // 当前工作副本路径
  const recentWorkspaces = ref<string[]>([])     // 最近打开列表（最多 20 条）
  const branchName = ref<string>('')             // 当前分支名
  const currentRevision = ref<number>(0)          // 当前版本号
  const lastCommitTime = ref<string>('')          // 最新提交时间
  const svnVersion = ref<string>('')              // 内置 svn 版本
  const isOffline = ref<boolean>(false)           // 网络状态（影响服务端操作按钮）
  const isLoading = ref<boolean>(false)           // 工作副本加载中

  // 切换工作副本时重置（保留 recentWorkspaces）
  function reset() { currentPath = ''; branchName = ''; currentRevision = 0; /* ... */ }

  async function refresh() { /* 保持选中状态，不重置表单 */ }
  async function switchWorkspace(path: string) { reset(); /* 切换到新路径 */ }
})
```

```ts
// stores/fileList.ts — 文件列表状态
export const useFileListStore = defineStore('fileList', () => {
  const files = ref<FileItem[]>([])               // 文件列表
  const selectedPaths = ref<Set<string>>(new Set()) // 选中文件路径集合
  const searchQuery = ref<string>('')               // 搜索关键字
  const filterStatus = ref<string>('all')            // 筛选：全部/已修改/新增/冲突
  const sortField = ref<string>('status')            // 排序字段
  const sortOrder = ref<'asc'|'desc'>('asc')         // 排序方向
  const isOperationRunning = ref<boolean>(false)     // 全局操作进行中（ProgressOverlay 期间禁用工具栏和快捷键）

  // 自动刷新策略：refresh() 时保留 selectedPaths、searchQuery、filterStatus
  // 不清除用户已勾选的文件和已输入的搜索条件
  async function refresh() { /* 刷新列表，保留 UI 状态 */ }
  function reset() { files = []; selectedPaths = new Set(); /* 切换工作副本时调用 */ }
})
```

```ts
// stores/settings.ts — 设置状态
export const useSettingsStore = defineStore('settings', () => {
  const defaultCheckoutDir = ref<string>('')
  const globalIgnorePattern = ref<string>('')
  const diffTool = ref<string>('builtin')
  const mergeTool = ref<string>('builtin')
  const showUnversioned = ref<boolean>(true)
  const language = ref<string>('system')
  const autoStart = ref<boolean>(false)

  // settings 在切换工作副本时不重置（全局设置）
  // 仅在应用退出时持久化到 tauri-plugin-store
})
```

```ts
// stores/svn.ts — SVN 操作封装
export const useSvnStore = defineStore('svn', () => {
  // 封装所有 Tauri invoke 调用，统一 try-catch 和错误码翻译
  async function checkStatus(path: string): Promise<FileItem[]> { /* ... */ }
  async function commit(paths: string[], message: string): Promise<number> { /* ... */ }
  // 每个方法返回 Promise，错误统一转换为友好消息
})
```

### 选择状态传递规则

提交面板的选择状态遵循以下规则（交互设计 §3.4）：

1. 进入提交面板时**继承**主视图的 `fileListStore.selectedPaths`
2. 提交面板内修改选择**不影响**主视图的选中状态
3. 提交完成后**清除**两个视图的选中状态

### 自动刷新策略

自动刷新由以下时机触发：

| 时机 | 刷新内容 | 行为 |
|------|---------|------|
| 窗口获得焦点（focus 事件） | 文件列表 | 保留 selectedPaths、searchQuery、filterStatus |
| 提交完成后 | 文件列表 | 清除选中状态 |
| 更新/还原/切换分支后 | 文件列表 | 保留搜索/筛选但不保留选中 |
| 切换工作副本后 | 全部状态 | workspace + fileList 调用 reset() |

刷新期间不重置用户正在进行的操作（如编辑提交日志）。

### Store 重置约定

切换工作副本时各 Store 行为：

| Store | 切换工作副本时 | 应用退出时 |
|-------|--------------|-----------|
| `workspace` | 重置（保留 recentWorkspaces） | 持久化：path、recentWorkspaces |
| `fileList` | 重置 | 不持久化 |
| `settings` | 保留（全局设置） | 持久化 |
| `svn` | 保留（无状态封装） | 不持久化 |

每个 Store 实现 `reset()` 方法，切换时统一调用。

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
| 目录名 | kebab-case | `svn-tree/` `commit-dialog/` `dialogs/` |
| TS 文件 | kebab-case | `repo-store.ts` `svn-service.ts` `use-auto-refresh.ts` |
| 组合式函数 | useXxx | `useKeyboardShortcuts` `useFileSelection` |
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
| `.vue` 组件 | 400 行 | 提取子组件到 `components/`，提取逻辑到 `stores/`、`services/` 或 `composables/` |
| `stores/*.ts` | 150 行 | 按业务领域拆分为多个 store（如 `workspace.ts`、`fileList.ts`） |
| `services/*.ts` | 200 行 | 按功能拆分为多个 service 文件（如 `svn-status.ts`、`svn-commit.ts`） |
| `composables/*.ts` | 100 行 | 按场景拆分（如 `useKeyboardShortcuts.ts`、`useFileSelection.ts`） |
| `types/*.ts` | 100 行 | 按模块拆分（如 `types/svn.ts`、`types/settings.ts`） |
| `utils/*.ts` | 100 行 | 按功能拆分（如 `utils/format.ts`、`utils/path.ts`） |
| `router/index.ts` | 80 行 | 提取路由模块到 `router/modules/` 目录 |

## Composables 目录

`composables/` 目录用于封装带状态的复用逻辑（Vue 3 Composition API 的核心模式），区别于 `utils/`（纯函数）和 `stores/`（全局共享状态）。本项目中需要以下 composables：

| 文件 | 场景 | 说明 |
|------|------|------|
| `useKeyboardShortcuts.ts` | 全局快捷键注册 | 支持作用域管理（对话框 > 面板 > 全局），`Esc` 关闭顶层元素 |
| `useFileSelection.ts` | Shift+Click / Ctrl+Click 多选 | 封装选择逻辑，供 FileListTable 和提交面板共用 |
| `useAutoRefresh.ts` | window focus 自动刷新 | 封装 Tauri window focus 事件监听 + 防抖 |
| `useOperationGuard.ts` | 操作进行中全局禁用 | 读取 `fileListStore.isOperationRunning`，控制工具栏/快捷键/操作按钮 |
| `useNetworkStatus.ts` | 网络可达性检测 | 每个 Command 调用前后端网络检测，更新 `workspaceStore.isOffline` |

**判断是否放入 composables 的标准：**
- 有状态但非全局 → `composables/`
- 跨多个组件复用的纯逻辑 → `composables/`
- 无状态的纯函数 → `utils/`
- 全局共享状态 → `stores/`

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
