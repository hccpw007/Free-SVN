<script setup lang="ts">
/** 应用根组件——全局弹窗管理（认证、提交、检出）及 Shell 命令监听。 */
import { ref, onMounted, onUnmounted, watch, provide } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute, RouterView } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from '@/stores/settings'
import { useSvnStore } from '@/stores/svn'
import { useSvnEventsStore } from '@/stores/svnEvents'
import { useWorkspaceStore } from '@/stores/workspace'
import { useKeyboardShortcuts, setOperationRunning } from '@/composables/useKeyboardShortcuts'
import { useFileListStore } from '@/stores/fileList'
import AuthDialog from '@/components/dialogs/AuthDialog.vue'
import CommitDialog from '@/components/dialogs/CommitDialog.vue'
import CheckoutDialog from '@/components/dialogs/CheckoutDialog.vue'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

const { locale, t } = useI18n()
const settingsStore = useSettingsStore()
const svnStore = useSvnStore()
const svnEventsStore = useSvnEventsStore()
const workspaceStore = useWorkspaceStore()
const router = useRouter()
const route = useRoute()
const fileListStore = useFileListStore()

// ── 提交弹窗 ──
const showCommitDialog = ref(false)
provide('openCommitDialog', () => { showCommitDialog.value = true })

// ── 认证失败→AuthDialog 自动弹窗 ──
const showAuthDialog = ref(false)

// ── 进度独立窗口管理 ──
let progressWindow: WebviewWindow | null = null

async function openProgressWindow() {
  // 如果窗口已存在，激活并返回
  const existing = await WebviewWindow.getByLabel('progress').catch(() => null)
  if (existing) {
    progressWindow = existing
    await existing.show()
    await existing.setFocus()
    return
  }
  // 创建新窗口
  const win = new WebviewWindow('progress', {
    url: '/progress-window',
    width: 880,
    height: 587,
    minWidth: 480,
    minHeight: 360,
    resizable: true,
    decorations: true,
    center: true,
  })
  // 监听窗口创建失败事件
  win.once('tauri://error', (e) => {
    console.error('[App] 进度窗口创建失败:', e)
  })
  progressWindow = win
}

function closeProgressWindow() {
  if (progressWindow) {
    try { progressWindow.close() } catch { /* ignore */ }
    progressWindow = null
  }
}

watch(() => svnEventsStore.authFailed, (val) => {
  showAuthDialog.value = val
})

async function handleAuthRetry(username: string, password: string, saveToCache: boolean): Promise<boolean> {
  // 在 retryAuth 清除 authContext 前捕获上下文
  const ctx = svnEventsStore.authContext
  const ok = await svnEventsStore.retryAuth(username, password, saveToCache)
  if (ok) {
    showAuthDialog.value = false
    // 重试成功且为检出操作 → 切换到工作区（同时会关闭 CheckoutDialog）
    if (ctx?.command === 'checkout_repo' && ctx?.args?.targetPath) {
      const { useWorkspaceStore } = await import('@/stores/workspace')
      const ws = useWorkspaceStore()
      await ws.switchWorkspace(ctx.args.targetPath as string)
    }
  }
  return ok
}

function handleAuthClose() {
  showAuthDialog.value = false
  svnEventsStore.cancelAuth()
}

/** 根据 settings.language 字段设置 vue-i18n locale，支持 'system' 自动检测 */
function applyLanguage(lang: string) {
  if (lang === 'system') {
    const navLang = navigator.language
    // 支持 zh-CN/en/ja/ko，其余回退 en
    if (['zh-CN', 'en', 'ja', 'ko'].includes(navLang)) {
      locale.value = navLang
    } else {
      locale.value = 'en'
    }
  } else {
    locale.value = lang
  }
}

/** 应用暗色模式设置 */
function applyDarkMode(darkMode: boolean | undefined) {
  if (darkMode === true) {
    document.documentElement.classList.add('dark')
  } else if (darkMode === false) {
    document.documentElement.classList.remove('dark')
  } else {
    // system
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    document.documentElement.classList.toggle('dark', mq.matches)
    mq.addEventListener('change', (e) => {
      if (settingsStore.darkMode === undefined) {
        document.documentElement.classList.toggle('dark', e.matches)
      }
    })
  }
}

/** 处理右键菜单传递的 Shell 命令 */
async function handleShellCommand(command: string, files: string[]) {
  // 需要交互的操作 → 导航到对应页面
  const interactiveCommands = ['commit', 'log', 'diff', 'switch', 'branch-tag', 'merge', 'checkout', 'export', 'update-rev']

  if (interactiveCommands.includes(command)) {
    switch (command) {
      case 'commit':
        showCommitDialog.value = true
        break
      case 'log':
        router.push('/workspace/log')
        break
      case 'diff':
        if (files.length > 0) {
          router.push(`/workspace/diff?file=${encodeURIComponent(files[0])}`)
        }
        break
      // 以下为对话框操作，无独立路由，不执行导航
      case 'switch':
      case 'branch-tag':
      case 'merge':
      case 'checkout':
      case 'export':
      case 'update-rev':
        svnEventsStore.showUpdateRevisionDialog = true
        break
    }
    return
  }

  // 后台操作直接执行
  const workspaceStore = (await import('@/stores/workspace')).useWorkspaceStore()
  const ws = workspaceStore()
  const path = ws.currentPath
  if (!path) return

  switch (command) {
    case 'update':
      await svnStore.updateWorkspace({ path })
      break
    case 'cleanup':
      await svnStore.cleanup(path)
      break
    case 'revert':
      if (files.length > 0) {
        await svnStore.revertFiles(files)
      }
      break
  }
}

onMounted(async () => {
  await settingsStore.load()
  applyLanguage(settingsStore.language)
  applyDarkMode(settingsStore.darkMode)

  // 初始化 SVN 事件监听（监听 operation:started/progress/line/cancelled/completed/error）
  svnEventsStore.initEventListeners()

  // 全局焦点跟踪：同步 fileListStore.isOperationRunning 到 useKeyboardShortcuts
  watch(() => fileListStore.isOperationRunning, (v) => {
    setOperationRunning(v)
  })

  // 操作进行中时创建/关闭进度独立窗口
  watch(() => svnEventsStore.isOperationRunning, (running) => {
    if (running) {
      openProgressWindow()
    }
  })

  // 进度窗口就绪时，转发当前操作状态（防止窗口创建时序导致的事件丢失）
  listen('progress-window:ready', async () => {
    const pw = await WebviewWindow.getByLabel('progress').catch(() => null)
    if (!pw) return
    pw.emit('progress-window:catchup', {
      isOperationRunning: svnEventsStore.isOperationRunning,
      fileLines: svnEventsStore.fileLines,
      progress: svnEventsStore.progress,
    }).catch(() => {})
  })

  // 注册 8 组全局键盘快捷键
  const { register } = useKeyboardShortcuts()
  register([
    // Ctrl+Enter → 打开提交弹窗
    { id: 'global-commit', keys: 'ctrl+Enter', handler: () => { showCommitDialog.value = true }, scope: 'global' },
    // Ctrl+D → 选中文件时导航到差异对比页面
    { id: 'global-diff', keys: 'ctrl+d', handler: () => {
      const paths = fileListStore.selectedPaths
      if (paths.size > 0) {
        const first = paths.values().next().value
        if (first) router.push(`/workspace/diff?file=${encodeURIComponent(first)}`)
      }
    }, scope: 'global' },
    // Ctrl+R → 刷新文件列表
    { id: 'global-refresh', keys: 'ctrl+r', handler: () => { fileListStore.refresh() }, scope: 'global' },
    // Ctrl+, → 导航到设置页面
    { id: 'global-settings', keys: 'ctrl+,', handler: () => router.push('/settings'), scope: 'global' },
    // Ctrl+W → 如果在 /workspace 子路由下，返回工作区首页
    { id: 'global-back-workspace', keys: 'ctrl+w', handler: () => {
      if (route.path.startsWith('/workspace')) router.push('/workspace')
    }, scope: 'global' },
    // Esc → 如果在日志页面，返回工作区首页
    { id: 'global-escape', keys: 'Escape', handler: () => {
      if (route.path.startsWith('/workspace/log')) {
        router.push('/workspace')
      }
    }, scope: 'global' },
    // Ctrl+F → 如果在工作区首页，聚焦搜索框
    { id: 'global-search', keys: 'ctrl+f', handler: () => {
      if (route.path === '/workspace') {
        document.querySelector<HTMLInputElement>('.el-input__inner')?.focus()
      }
    }, scope: 'global' },
    // Ctrl+↓ → 导航下一个差异块（DiffViewer 可见时由组件内 panel 级快捷键优先处理）
    { id: 'global-diff-next', keys: 'ctrl+ArrowDown', handler: () => {}, scope: 'global' },
    // Ctrl+↑ → 导航上一个差异块（DiffViewer 可见时由组件内 panel 级快捷键优先处理）
    { id: 'global-diff-prev', keys: 'ctrl+ArrowUp', handler: () => {}, scope: 'global' },
  ])

  // 监听 svn:shell-cmd 事件（右键菜单触发）
  try {
    await listen<{ command: string; files: string[] }>('svn:shell-cmd', (event) => {
      const { command, files } = event.payload
      handleShellCommand(command, files)
    })
  } catch (e: unknown) {
    console.warn('[App] Tauri event listener 不可用（浏览器环境）:', e)
  }
})

onUnmounted(() => {
  closeProgressWindow()
})
</script>

<template>
  <!-- 路由视图出口 -->
  <RouterView />
  <!-- 认证失败时弹出 -->
  <AuthDialog
    v-if="showAuthDialog"
    :repo-url="svnEventsStore.authContext?.args?.url as string || ''"
    :on-retry="handleAuthRetry"
    @close="handleAuthClose"
    @success="showAuthDialog = false"
  />
  <!-- 提交弹窗 -->
  <CommitDialog v-if="showCommitDialog" @close="showCommitDialog = false" />
  <!-- 检出弹窗 -->
  <CheckoutDialog
    v-if="workspaceStore.showCheckoutDialog"
    :initialPath="workspaceStore.checkoutInitialPath"
    @close="workspaceStore.showCheckoutDialog = false; workspaceStore.checkoutInitialPath = ''"
  />
</template>
