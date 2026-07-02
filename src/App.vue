<script setup lang="ts">
import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter, RouterView } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { useSettingsStore } from '@/stores/settings'
import { useSvnStore } from '@/stores/svn'

const { locale } = useI18n()
const settingsStore = useSettingsStore()
const svnStore = useSvnStore()
const router = useRouter()

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
