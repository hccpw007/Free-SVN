<script setup lang="ts">
/** 顶栏组件——工作副本路径显示、分支名徽章、暗色模式切换、设置入口。 */
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { GitBranch, Moon, Sun, Settings } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import { open } from '@tauri-apps/plugin-dialog'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'
import { getInfo } from '@/services/svn'

const { t } = useI18n()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const settingsStore = useSettingsStore()

// 当前工作副本路径（CSS 截断）
const displayPath = computed(() => workspaceStore.currentPath)

// 完整路径（用于 tooltip）
const fullPath = computed(() => workspaceStore.currentPath)
const branchName = computed(() => workspaceStore.branchName)

// 暗色模式状态
const isDark = computed(() => settingsStore.darkMode)

function toggleDarkMode() {
  const html = document.documentElement
  if (settingsStore.darkMode) {
    html.classList.remove('dark')
    settingsStore.darkMode = false
  } else {
    html.classList.add('dark')
    settingsStore.darkMode = true
  }
  settingsStore.save()
}

// 点击路径复制
async function copyPath() {
  if (!workspaceStore.currentPath) return
  try {
    await navigator.clipboard.writeText(workspaceStore.currentPath)
    ElMessage.success(t('common.copied'))
  } catch (e: unknown) {
    console.warn('[TopBar] 复制路径失败:', e)
    ElMessage.warning(`${t('common.copyFailed')}：${workspaceStore.currentPath}`)
  }
}

// 打开文件夹选择窗口切换工作空间
async function handleSwitchWorkspace() {
  try {
    const selected = await open({ directory: true })
    if (selected && typeof selected === 'string') {
      // 先检测是否为 SVN 工作副本
      try {
        await getInfo(selected)
        // 是工作副本 → 正常切换
        await workspaceStore.switchWorkspace(selected)
      } catch {
        // 如果是已知的不完整检出目录，仍然切换进去（首页会显示继续更新提示）
        if (selected === workspaceStore.incompleteCheckoutPath) {
          workspaceStore.clearIncompleteCheckout()
          await workspaceStore.switchWorkspace(selected)
          return
        }
        // 非工作副本 → 打开检出对话框（预填路径，不切换）
        workspaceStore.checkoutInitialPath = selected
        workspaceStore.showCheckoutDialog = true
      }
    }
  } catch (e: unknown) {
    console.warn('[TopBar] 切换工作空间失败:', e)
  }
}
</script>

<template>
  <div class="h-12 px-4 flex items-center gap-3 bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700 text-sm">
    <!-- 左边区域 -->
    <div class="flex items-center gap-2 flex-1 min-w-0">
      <GitBranch class="w-5 h-5 text-green-500 shrink-0" />

      <!-- 有工作副本时：切换按钮 + 路径 + 分支名 -->
      <template v-if="workspaceStore.currentPath">
        <!-- 切换工作空间按钮 -->
        <button
          class="px-3 py-1 rounded-md bg-green-500 hover:bg-green-600 text-white text-xs font-medium transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none shrink-0"
          @click="handleSwitchWorkspace"
        >
          {{ t('toolbar.switchWorkspace') }}
        </button>

        <span
          class="truncate overflow-hidden whitespace-nowrap max-w-[400px] text-slate-600 dark:text-slate-300 cursor-pointer hover:text-green-500 transition-colors duration-150"
          :title="fullPath"
          @click="copyPath"
        >
          {{ displayPath }}
        </span>

        <!-- 分支名 -->
        <span
          v-if="branchName"
          class="px-2 py-0.5 text-xs font-mono rounded-full bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400 border border-green-200 dark:border-green-800"
        >
          {{ branchName }}
        </span>
      </template>

      <!-- 无工作副本时：显示应用名 -->
      <template v-else>
        <span class="text-sm font-semibold text-slate-800 dark:text-slate-200">
          Free-SVN
        </span>
      </template>
    </div>

    <!-- 右边区域：暗色模式切换 + 设置 -->
    <button
      class="p-1.5 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
      :aria-label="isDark ? t('common.switchToLightMode') : t('common.switchToDarkMode')"
      @click="toggleDarkMode"
    >
      <Moon v-if="!isDark" class="w-4 h-4 text-slate-500" />
      <Sun v-else class="w-4 h-4 text-sky-400" />
    </button>
    <button
      class="p-1.5 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
      :aria-label="t('toolbar.settings')"
      @click="router.push('/settings')"
    >
      <Settings class="w-4 h-4 text-slate-500" />
    </button>
  </div>
</template>
