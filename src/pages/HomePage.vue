<script setup lang="ts">
/** 工作副本主页——变更列表视图、筛选/搜索、空状态/刷新逻辑。 */
import { useRouter } from 'vue-router'
import { useWorkspaceStore } from '@/stores/workspace'
import { useFileListStore } from '@/stores/fileList'
import { useSvnEventsStore } from '@/stores/svnEvents'
import { useNetworkStatus } from '@/composables/useNetworkStatus'
import { ref, computed, onMounted, watch, inject } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { AlertTriangle, RefreshCw, CheckCircle, Search, GitCommit, RotateCcw, Lock } from 'lucide-vue-next'
import { exists } from '@tauri-apps/plugin-fs'
import { getInfo as fetchInfo } from '@/services/svn'
import { useSvnStore } from '@/stores/svn'
import FileListTable from '@/components/svn/FileListTable.vue'
import UpdateRevisionDialog from '@/components/dialogs/UpdateRevisionDialog.vue'
import SwitchDialog from '@/components/dialogs/SwitchDialog.vue'

const router = useRouter()
const { t } = useI18n()
const workspaceStore = useWorkspaceStore()
const fileListStore = useFileListStore()
const svnEventsStore = useSvnEventsStore()
const svnStore = useSvnStore()

// 从 App.vue 注入的打开提交弹窗函数
const openCommitDialog = inject<() => void>('openCommitDialog', () => {})

// 网络可达性检测：更新 workspaceStore.isOffline
const { checkNetwork } = useNetworkStatus()

const isEmptyChanges = computed(() => !!workspaceStore.currentPath && fileListStore.files.length === 0)
/** 工作副本是否被锁定（检出中断时 wc-locked="true"），需先执行 Cleanup */
const isWcLocked = computed(() => fileListStore.files.some(f => f.wcLocked))
const isSearchEmpty = computed(() => {
  return !!workspaceStore.currentPath
    && fileListStore.files.length > 0
    && fileListStore.filteredFiles.length === 0
})
/** 已勾选的可提交文件数（含未加入/缺失，提交时会自动预处理） */
const selectedCommitCount = computed(() =>
  fileListStore.files.filter(f => fileListStore.selectedPaths.has(f.path)).length
)

const showSwitchDialog = ref(false)
// 不完整检出横幅是否被手动关闭（路径保留供后续切换使用）
const incompleteBannerDismissed = ref(false)

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
    fileListStore.clearSelection()
    await refreshWorkspaceInfo()
    await fileListStore.refresh()
  } else {
    // currentPath 被清空 → 跳转到欢迎页
    router.replace('/workspace/welcome')
    return
  }
  checkNetwork()
})

onMounted(async () => {
  // 挂载时检测网络状态
  checkNetwork()

  if (workspaceStore.currentPath) {
    // 检查工作副本路径是否存在
    try {
      const pathExists = await exists(workspaceStore.currentPath)
      if (!pathExists) {
        ElMessage.warning(t('workspace.pathNotFound'))
        await workspaceStore.switchWorkspace('')
        return
      }
    } catch (e: unknown) {
      console.warn('[HomePage] path exists 检查失败，继续加载:', e)
    }
    // 挂载时刷新工作副本信息（包括 isWorkingCopy 状态）
    await refreshWorkspaceInfo()
    await fileListStore.refresh()
  } else {
    // 没有当前工作副本 → 跳转到欢迎页
    router.replace('/workspace/welcome')
  }
})

// 切换工作副本后自动刷新 workspace 信息（isWorkingCopy 由 workspaceStore.switchWorkspace 负责检测）
async function refreshWorkspaceInfo() {
  if (!workspaceStore.currentPath) return
  try {
    const info = await fetchInfo(workspaceStore.currentPath)
    workspaceStore.lastCommitTime = info.lastChangedDate ?? ''
    workspaceStore.currentRevision = info.revision ?? 0
    workspaceStore.url = info.url ?? ''
    workspaceStore.sourceUrl = info.url ?? ''
    workspaceStore.branchName = info.branchName ?? ''
  } catch (e: unknown) {
    console.warn('[HomePage] refreshWorkspaceInfo 失败:', e)
  }
}

/** 继续更新不完整的检出目录 */
async function handleResumeUpdate() {
  const path = workspaceStore.incompleteCheckoutPath
  if (!path) return
  workspaceStore.clearIncompleteCheckout()
  await workspaceStore.switchWorkspace(path)
  await svnStore.updateWorkspace({ path })
}

/** 忽略不完整检出提示 */
function handleDismissIncomplete() {
  incompleteBannerDismissed.value = true
}

/** 对锁定的工作副本执行 Cleanup */
async function handleWcCleanup() {
  if (!workspaceStore.currentPath) return
  await svnStore.cleanup(workspaceStore.currentPath)
  await fileListStore.refresh()
}
</script>

<template>
  <!-- 更新到版本对话框（由右键菜单 --svn-cmd update-rev 触发） -->
  <UpdateRevisionDialog v-if="svnEventsStore.showUpdateRevisionDialog" @close="svnEventsStore.showUpdateRevisionDialog = false" />
  <!-- 切换分支对话框 -->
  <SwitchDialog v-if="showSwitchDialog" @close="showSwitchDialog = false" />

  <!-- 工作副本：变更列表视图 -->
  <div class="h-full flex flex-col">
    <!-- 不完整检出提示横幅 -->
    <div
      v-if="workspaceStore.incompleteCheckoutPath && !incompleteBannerDismissed"
      class="mx-4 mt-2 px-4 py-3 rounded-lg bg-amber-50 dark:bg-amber-900/20 border border-amber-200 dark:border-amber-700"
    >
      <div class="flex items-start gap-3">
        <AlertTriangle class="w-5 h-5 text-amber-500 shrink-0 mt-0.5" />
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-amber-800 dark:text-amber-300">
            检出操作已取消
          </p>
          <p class="text-xs text-amber-600 dark:text-amber-400 mt-1 break-all font-mono">
            {{ workspaceStore.incompleteCheckoutPath }}
          </p>
          <p class="text-xs text-amber-600 dark:text-amber-400 mt-1">
            此目录已包含部分文件，更新后可继续同步剩余文件
          </p>
        </div>
        <div class="flex gap-2 shrink-0">
          <button
            class="px-3 py-1.5 text-xs font-medium rounded-md bg-amber-600 text-white hover:bg-amber-700 transition-colors focus:ring-2 focus:ring-amber-400 focus:outline-none"
            @click="handleResumeUpdate"
          >继续更新</button>
          <button
            class="px-3 py-1.5 text-xs font-medium rounded-md text-amber-700 dark:text-amber-300 hover:bg-amber-100 dark:hover:bg-amber-800/30 transition-colors focus:ring-2 focus:ring-amber-400 focus:outline-none"
            @click="handleDismissIncomplete"
          >忽略</button>
        </div>
      </div>
    </div>

    <!-- 操作栏：左（提交 + 批量还原）右（搜索 + 筛选 + 刷新） -->
    <div class="px-4 py-2 flex items-center gap-3 bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700">
      <!-- 提交 -->
      <button
        class="inline-flex items-center leading-none gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md whitespace-nowrap transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-emerald-400 focus:outline-none"
        :class="selectedCommitCount > 0
          ? 'bg-emerald-50 text-emerald-700 hover:bg-emerald-100 dark:bg-emerald-900/20 dark:text-emerald-300'
          : 'bg-slate-50 text-slate-400 cursor-not-allowed dark:bg-slate-800 dark:text-slate-500'"
        :disabled="selectedCommitCount === 0"
        :title="selectedCommitCount === 0 ? t('toolbar.noSelection') : t('toolbar.descCommit')"
        :aria-label="t('toolbar.commit')"
        @click="openCommitDialog()"
      >
        <GitCommit class="w-4 h-4" /><span>{{ t('toolbar.commit') }}</span>
        <span class="ml-0.5 tabular-nums">({{ selectedCommitCount }})</span>
      </button>
      <!-- 批量还原 -->
      <button
        class="inline-flex items-center leading-none gap-1.5 px-3 py-1.5 text-xs font-medium rounded-md whitespace-nowrap transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-amber-400 focus:outline-none"
        :class="fileListStore.selectedPaths.size > 0
          ? 'bg-amber-50 text-amber-700 hover:bg-amber-100 dark:bg-amber-900/20 dark:text-amber-300'
          : 'bg-slate-50 text-slate-400 cursor-not-allowed dark:bg-slate-800 dark:text-slate-500'"
        :disabled="fileListStore.selectedPaths.size === 0"
        :title="fileListStore.selectedPaths.size === 0 ? t('toolbar.noSelection') : t('file.batchRevert')"
        :aria-label="t('file.batchRevert')"
        @click="fileListStore.batchRevertFiles()"
      >
        <RotateCcw class="w-4 h-4" /><span>{{ t('file.batchRevert') }}</span>
        <span class="ml-0.5 tabular-nums">({{ fileListStore.selectedPaths.size }})</span>
      </button>

      <div class="flex-1" />

      <!-- 右侧筛选/搜索 -->
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
        <el-option :label="t('file.statusPendingAdd')" value="pendingAdd" />
        <el-option :label="t('file.statusModified')" value="modified" />
        <el-option :label="t('file.statusPendingDelete')" value="pendingDelete" />
        <el-option :label="t('file.statusConflicted')" value="conflicted" />
        <el-option :label="t('file.statusIgnored')" value="ignored" />
        <el-option :label="t('file.statusAbnormal')" value="abnormal" />
      </el-select>
      <button
        class="p-1.5 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
        :title="t('common.refresh')"
        :aria-label="t('common.refresh')"
        @click="fileListStore.refresh()"
      >
        <RefreshCw class="w-4 h-4" />
      </button>
    </div>

    <!-- 工作副本锁定提示（检出中断 wc-locked，需先 Cleanup） -->
    <div v-if="isWcLocked" class="flex-1 flex items-center justify-center">
      <div class="text-center max-w-md px-6">
        <Lock class="w-10 h-10 mx-auto text-red-500" />
        <p class="text-sm font-medium text-slate-700 dark:text-slate-300 mt-3">{{ t('workspace.wcLockedTitle') }}</p>
        <p class="text-xs text-slate-500 dark:text-slate-400 mt-1">{{ t('workspace.wcLockedDesc') }}</p>
        <button
          class="mt-4 px-4 py-2 text-sm font-medium rounded-md bg-red-600 text-white hover:bg-red-700 transition-colors focus:ring-2 focus:ring-red-400 focus:outline-none"
          @click="handleWcCleanup"
        >{{ t('workspace.runCleanup') }}</button>
      </div>
    </div>

    <!-- 空变更状态 -->
    <div v-else-if="isEmptyChanges" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <CheckCircle class="w-8 h-8 mx-auto text-green-500" />
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-2">{{ t('workspace.noChanges') }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1 font-mono">{{ t('workspace.latestVersion') }}: {{ workspaceStore.currentRevision }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 font-mono">{{ t('workspace.lastCommit') }}: {{ workspaceStore.lastCommitTime }}</p>
        <div class="mt-4 flex gap-3 justify-center">
          <button class="text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="fileListStore.refresh()">{{ t('common.refresh') }}</button>
          <button class="text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="showSwitchDialog = true">{{ t('workspace.switchTitle') }}</button>
          <button class="text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="router.push('/workspace/log')">{{ t('workspace.viewLog') }}</button>
        </div>
      </div>
    </div>

    <!-- 搜索/筛选空结果 -->
    <div v-else-if="isSearchEmpty" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <Search class="w-8 h-8 mx-auto text-slate-400" />
        <p class="text-sm text-slate-500 dark:text-slate-400 mt-2">{{ t('workspace.noSearchResult') }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 mt-1">{{ t('workspace.searchFilterHint') }}</p>
        <button class="mt-2 text-xs text-blue-600 hover:underline focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" @click="fileListStore.clearFilter()">{{ t('workspace.clearFilter') }}</button>
      </div>
    </div>

    <!-- 文件列表表格 -->
    <FileListTable v-else class="flex-1" />
  </div>
</template>
