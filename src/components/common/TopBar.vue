<script setup lang="ts">
import { computed, ref } from 'vue'
import { GitBranch, KeyRound, Moon, Sun } from 'lucide-vue-next'
import { ElMessage } from 'element-plus'
import AuthDialog from '@/components/dialogs/AuthDialog.vue'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSettingsStore } from '@/stores/settings'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const settingsStore = useSettingsStore()
const showCredentialsDialog = ref(false)

// 当前工作副本路径（中间省略）
const displayPath = computed(() => {
  const path = workspaceStore.currentPath
  if (!path) return ''
  if (path.length > 50) {
    const head = path.slice(0, 25)
    const tail = path.slice(-20)
    return `${head}...${tail}`
  }
  return path
})

// 完整路径（用于 tooltip）
const fullPath = computed(() => workspaceStore.currentPath)
const branchName = computed(() => workspaceStore.branchName)

// 暗色模式状态
const isDark = computed(() => document.documentElement.classList.contains('dark'))

function toggleDarkMode() {
  const html = document.documentElement
  if (html.classList.contains('dark')) {
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
  } catch {
    ElMessage.warning(`${t('common.copyFailed')}：${workspaceStore.currentPath}`)
  }
}

function handleSwitchWorkspace(path: string) {
  workspaceStore.switchWorkspace(path)
}
</script>

<template>
  <div class="h-12 px-4 flex items-center gap-3 bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700 text-sm">
    <!-- 左边区域 -->
    <div class="flex items-center gap-2 flex-1 min-w-0">
      <GitBranch class="w-5 h-5 text-green-500 shrink-0" />

      <!-- 有工作副本时：显示路径 + 切换下拉 + 分支名 -->
      <template v-if="workspaceStore.currentPath">
        <span
          class="text-slate-600 dark:text-slate-300 truncate cursor-pointer hover:text-green-500 transition-colors duration-150"
          :title="fullPath"
          @click="copyPath"
        >
          {{ displayPath }}
        </span>

        <!-- 切换下拉 -->
        <el-select
          v-if="workspaceStore.recentWorkspaces.length > 0"
          :model-value="workspaceStore.currentPath"
          size="small"
          class="!w-32"
          placeholder="切换..."
          @change="handleSwitchWorkspace"
        >
          <el-option
            v-for="wp in workspaceStore.recentWorkspaces"
            :key="wp"
            :label="wp"
            :value="wp"
          />
        </el-select>

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

    <!-- 右边区域：凭据管理 + 暗色模式切换 -->
    <button
      class="p-1.5 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
      :aria-label="t('common.credentialManagement')"
      @click="showCredentialsDialog = true"
    >
      <KeyRound class="w-4 h-4 text-slate-500" />
    </button>
    <AuthDialog v-if="showCredentialsDialog" mode="manage" :repo-url="workspaceStore.url" @close="showCredentialsDialog = false" />
    <button
      class="p-1.5 rounded-md hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
      :aria-label="isDark ? t('common.switchToLightMode') : t('common.switchToDarkMode')"
      @click="toggleDarkMode"
    >
      <Moon v-if="!isDark" class="w-4 h-4 text-slate-500" />
      <Sun v-else class="w-4 h-4 text-sky-400" />
    </button>
  </div>
</template>
