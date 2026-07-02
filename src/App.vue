<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, useRoute, RouterView } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from '@/stores/settings'
import { useSvnStore } from '@/stores/svn'
import { useKeyboardShortcuts, setOperationRunning } from '@/composables/useKeyboardShortcuts'
import { useFileListStore } from '@/stores/fileList'

const { locale } = useI18n()
const settingsStore = useSettingsStore()
const svnStore = useSvnStore()
const router = useRouter()
const route = useRoute()
const fileListStore = useFileListStore()

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
        router.push('/workspace/commit')
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
        svnStore.showUpdateRevisionDialog = true
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

  // 关闭按钮隐藏窗口到托盘而非退出
  try {
    const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow')
    const win = getCurrentWebviewWindow()
    win.onCloseRequested(async () => {
      await win.hide()
    })
  } catch {
    // Tauri 环境不可用（如浏览器开发环境）
  }

  // 全局焦点跟踪：同步 fileListStore.isOperationRunning 到 useKeyboardShortcuts
  watch(() => fileListStore.isOperationRunning, (v) => {
    setOperationRunning(v)
  })

  // 注册 8 组全局键盘快捷键
  const { register } = useKeyboardShortcuts()
  register([
    // Ctrl+Enter → 导航到提交页面
    { id: 'global-commit', keys: 'ctrl+Enter', handler: () => router.push('/workspace/commit'), scope: 'global' },
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
    // Esc → 如果在提交或日志页面，返回工作区首页
    { id: 'global-escape', keys: 'Escape', handler: () => {
      if (route.path.startsWith('/workspace/commit') || route.path.startsWith('/workspace/log')) {
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
  } catch {
    // Tauri 环境不可用
  }
})
</script>

<template>
  <RouterView />
</template>
