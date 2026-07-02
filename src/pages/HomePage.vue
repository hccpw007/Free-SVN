<script setup lang="ts">
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '@/stores/workspace'
import { useFileListStore } from '@/stores/fileList'
import { ref, computed, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { RefreshCw, X, CircleCheck, Search } from 'lucide-vue-next'
import { exists } from '@tauri-apps/plugin-fs'
import { open } from '@tauri-apps/plugin-dialog'
import { fetchInfo } from '@/services/svn'
import FileListTable from '@/components/svn/FileListTable.vue'
import CheckoutDialog from '@/components/dialogs/CheckoutDialog.vue'

const router = useRouter()
const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const fileListStore = useFileListStore()

const isWelcomePage = computed(() => !workspaceStore.currentPath)
const isEmptyChanges = computed(() => !!workspaceStore.currentPath && fileListStore.files.length === 0)
const isSearchEmpty = computed(() => {
  return !!workspaceStore.currentPath
    && fileListStore.files.length > 0
    && fileListStore.filteredFiles.length === 0
})

const showCheckoutDialog = ref(false)

// 搜索框 300ms 防抖
let searchTimer: ReturnType<typeof setTimeout>
watch(() => fileListStore.searchQuery, () => {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    fileListStore.applyFilter()
  }, 300)
})

// 工作副本切换后自动刷新
watch(() => workspaceStore.currentPath, async (newPath) => {
  if (newPath) {
    await refreshWorkspaceInfo()
    await fileListStore.refresh()
  }
})

onMounted(async () => {
  if (workspaceStore.currentPath) {
    // 检查工作副本路径是否存在
    try {
      const pathExists = await exists(workspaceStore.currentPath)
      if (!pathExists) {
        ElMessage.warning(t('workspace.pathNotFound'))
        await workspaceStore.switchWorkspace('')
        return
      }
    } catch {
      // fallback: 路径检查失败时继续加载
    }
    await fileListStore.refresh()
  }
})

function handleCheckout() {
  showCheckoutDialog.value = true
}

// 切换工作副本后自动刷新 workspace 信息
async function refreshWorkspaceInfo() {
  if (!workspaceStore.currentPath) return
  try {
    const info = await fetchInfo(workspaceStore.currentPath)
    workspaceStore.lastCommitTime = info.lastChangedDate ?? ''
    workspaceStore.currentRevision = info.revision ?? 0
    workspaceStore.url = info.url ?? ''
    workspaceStore.sourceUrl = info.url ?? ''
    workspaceStore.branchName = info.branchName ?? ''
  } catch { /* 静默处理 */ }
}

async function handleOpenWorkspace() {
  try {
    const selected = await open({ directory: true })
    if (selected && typeof selected === 'string') {
      await workspaceStore.switchWorkspace(selected)
    }
  } catch {
    // fallback
  }
}
</script>

<template>
  <!-- 欢迎页（无工作副本） -->
  <div v-if="isWelcomePage" class="h-full flex items-center justify-center">
    <div class="text-center max-w-md px-8">
      <h1 class="text-3xl font-bold text-slate-800 dark:text-slate-100">
        {{ t('app.welcomeTitle') }}
      </h1>
      <p class="mt-2 text-sm text-slate-500 dark:text-slate-400">
        {{ t('app.welcomeSubtitle') }}
      </p>
      <div class="mt-8 flex flex-col items-center gap-3">
        <button
          class="w-56 px-4 py-2.5 rounded-md bg-green-500 hover:bg-green-600 text-white text-sm font-medium transition-colors duration-150 focus:ring-2 focus:ring-blue-400 focus:outline-none"
          @click="handleCheckout"
        >
          {{ t('common.checkout') }}...
        </button>
        <button
          class="w-56 px-4 py-2 rounded-md border border-slate-300 dark:border-slate-600 text-slate-700 dark:text-slate-300 text-sm hover:bg-slate-50 dark:hover:bg-slate-800 transition-colors duration-150 focus:ring-2 focus:ring-blue-400 focus:outline-none"
          @click="handleOpenWorkspace"
        >
          {{ t('workspace.openExisting') }}
        </button>
      </div>
      <div class="mt-8 text-left">
        <p class="text-xs font-medium text-slate-400 dark:text-slate-500 mb-2 uppercase tracking-wider">
          {{ t('workspace.recentWorkspaces') }}
        </p>
        <div v-if="workspaceStore.recentWorkspaces.length === 0" class="text-xs text-slate-400 dark:text-slate-500 italic">
          {{ t('workspace.noRecentWorkspaces') }}
        </div>
        <div v-else class="space-y-1">
          <div
            v-for="wp in workspaceStore.recentWorkspaces" :key="wp"
            class="flex items-center justify-between px-3 py-1.5 rounded-md text-xs text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-slate-800 cursor-pointer group transition-colors duration-150 font-mono"
            @click="workspaceStore.switchWorkspace(wp)"
          >
            <span class="truncate">{{ wp }}</span>
            <button
              class="opacity-0 group-hover:opacity-100 text-slate-400 hover:text-red-500 transition-opacity duration-150 focus:ring-2 focus:ring-blue-400 focus:outline-none rounded"
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

  <!-- 检出对话框 -->
  <CheckoutDialog v-if="showCheckoutDialog" @close="showCheckoutDialog = false" />

  <!-- 有工作副本：变更列表视图 -->
  <div v-else class="h-full flex flex-col">
    <!-- 搜索栏 + 筛选 + 刷新 -->
    <div class="px-4 py-2 flex items-center gap-3 bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700">
      <el-input
        v-model="fileListStore.searchQuery"
        :placeholder="t('workspace.searchPlaceholder')"
        size="default"
        clearable
        class="max-w-xs"
      />
      <el-select
        v-model="fileListStore.filterStatus"
        size="default"
        class="!w-36"
        @change="fileListStore.applyFilter()"
      >
        <el-option :label="t('file.filterAll')" value="all" />
        <el-option :label="t('file.statusModified')" value="modified" />
        <el-option :label="t('file.statusAdded')" value="added" />
        <el-option :label="t('file.statusConflicted')" value="conflicted" />
        <el-option :label="t('file.statusDeleted')" value="deleted" />
        <el-option :label="t('file.statusUnversioned')" value="unversioned" />
      </el-select>
      <button
        class="p-1.5 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-blue-400 focus:outline-none"
        :title="t('common.refresh')"
        :aria-label="t('common.refresh')"
        @click="fileListStore.refresh()"
      >
        <RefreshCw class="w-4 h-4" />
      </button>
    </div>

    <!-- 空变更状态 -->
    <div v-if="isEmptyChanges" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <CircleCheck class="w-8 h-8 mx-auto text-green-500" />
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-2">{{ t('workspace.noChanges') }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1 font-mono">{{ t('workspace.latestVersion') }}: {{ workspaceStore.currentRevision }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 font-mono">{{ t('workspace.lastCommit') }}: {{ workspaceStore.lastCommitTime }}</p>
        <div class="mt-4 flex gap-3 justify-center">
          <button class="text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="fileListStore.refresh()">{{ t('common.refresh') }}</button>
          <button class="text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="router.push('/workspace/log')">{{ t('workspace.viewLog') }}</button>
        </div>
      </div>
    </div>

    <!-- 搜索/筛选空结果 -->
    <div v-else-if="isSearchEmpty" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <Search class="w-8 h-8 mx-auto text-slate-400" />
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-2">{{ t('workspace.noSearchResult') }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ t('workspace.searchFilterHint') }}</p>
        <button class="mt-2 text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-blue-400 focus:outline-none rounded" @click="fileListStore.clearFilter()">{{ t('workspace.clearFilter') }}</button>
      </div>
    </div>

    <!-- 文件列表表格 -->
    <FileListTable v-else class="flex-1" />
  </div>
</template>
