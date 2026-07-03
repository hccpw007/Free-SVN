<script setup lang="ts">
/** 欢迎页——检出新仓库 / 打开已有工作副本 / 最近工作副本列表。 */
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '@/stores/workspace'
import { useI18n } from 'vue-i18n'
import { watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { X } from 'lucide-vue-next'
import { getInfo } from '@/services/svn'

const router = useRouter()
const { t } = useI18n()
const workspaceStore = useWorkspaceStore()

function handleCheckout() {
  workspaceStore.showCheckoutDialog = true
}

// 检出成功后自动导航到工作区
watch(() => workspaceStore.isWorkingCopy, (isWc) => {
  if (isWc) {
    router.replace('/workspace')
  }
})

async function handleOpenWorkspace() {
  try {
    const selected = await open({ directory: true })
    if (selected && typeof selected === 'string') {
      // 先检测是否为 SVN 工作副本
      try {
        await getInfo(selected)
        // 是工作副本 → 切换并进入工作区
        await workspaceStore.switchWorkspace(selected)
      } catch {
        // 非工作副本 → 弹出检出对话框（路径预填），检出成功后自动进入工作区
        workspaceStore.checkoutInitialPath = selected
        workspaceStore.showCheckoutDialog = true
      }
    }
  } catch (e: unknown) {
    console.warn('[WelcomePage] 打开工作副本失败:', e)
  }
}

async function handleRecentWorkspaceClick(wp: string) {
  try {
    await getInfo(wp)
    // 依然有效的工作副本 → 直接进入
    await workspaceStore.switchWorkspace(wp)
  } catch {
    // 路径已失效（删除、权限变更等）→ 移除出最近列表
    workspaceStore.removeRecent(wp)
  }
}
</script>
<!-- 欢迎页 -->
<template>
  <!-- 欢迎页主容器 -->
  <div class="h-full flex items-center justify-center">
    <div class="text-center max-w-md px-8">
      <h1 class="text-3xl font-bold text-slate-800 dark:text-slate-100">
        {{ t('app.welcomeTitle') }}
      </h1>
      <p class="mt-2 text-sm text-slate-500 dark:text-slate-400">
        {{ t('app.welcomeSubtitle') }}
      </p>
      <!-- 操作按钮组 -->
      <div class="mt-8 flex flex-col items-center gap-3">
        <button
          class="w-56 px-4 py-2.5 rounded-md bg-green-500 hover:bg-green-600 text-white text-sm font-medium transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
          @click="handleCheckout"
        >
          {{ t('workspace.checkoutNewRepo') }}...
        </button>
        <button
          class="w-56 px-4 py-2 rounded-md border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
          @click="handleOpenWorkspace"
        >
          {{ t('workspace.openExisting') }}
        </button>
      </div>
      <!-- 最近工作副本区 -->
      <div class="mt-8 text-left">
        <p class="text-xs font-medium text-slate-400 dark:text-slate-500 mb-2 uppercase tracking-wider">
          {{ t('workspace.recentWorkspaces') }}
        </p>
        <!-- 无最近工作副本时的提示 -->
        <div v-if="workspaceStore.recentWorkspaces.length === 0" class="text-xs text-slate-400 dark:text-slate-500 italic">
          {{ t('workspace.noRecentWorkspaces') }}
        </div>
        <!-- 最近工作副本列表 -->
        <div v-else class="space-y-1">
          <div
            v-for="wp in workspaceStore.recentWorkspaces" :key="wp"
            class="flex items-center justify-between px-3 py-1.5 rounded-md text-xs text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer group transition-colors duration-150 font-mono"
            @click="handleRecentWorkspaceClick(wp)"
          >
            <span class="truncate">{{ wp }}</span>
            <!-- 移出该工作副本 -->
            <button
              class="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-red-500 transition-opacity duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded"
              :title="t('common.remove')"
              @click.stop="workspaceStore.removeRecent(wp)"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
