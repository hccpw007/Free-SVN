<script setup lang="ts">
/** 工具栏组件——SVN 操作按钮（diff/update/log/switch/merge 等）。 */
import { ref, computed, onMounted, onUnmounted, type Component } from 'vue'
import { useRouter } from 'vue-router'
import {
  FileDiff, RefreshCw, ArrowUpCircle, History,
  GitBranch, Layers, GitMerge, Eraser, Package,
  Download, MoreHorizontal,
} from 'lucide-vue-next'
import { useWorkspaceStore } from '@/stores/workspace'
import { useFileListStore } from '@/stores/fileList'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const router = useRouter()
const workspaceStore = useWorkspaceStore()
const fileListStore = useFileListStore()
const svnStore = useSvnStore()

// 工具栏按钮事件（父组件处理对话框打开）
const emit = defineEmits<{
  'open-dialog': [dialog: string]
}>()

const toolbarRef = ref<HTMLElement | null>(null)
const overflowCount = ref(0)
let resizeObserver: ResizeObserver | undefined

// 严格索引类型
type IconKey = 'FileDiff' | 'RefreshCw' | 'ArrowUpCircle' | 'History'
  | 'GitBranch' | 'Layers' | 'GitMerge' | 'Eraser' | 'Package'
  | 'Download' | 'MoreHorizontal'

// lucide-vue-next 图标组件映射
const iconMap: Record<IconKey, Component> = {
  FileDiff, RefreshCw, ArrowUpCircle, History,
  GitBranch, Layers, GitMerge, Eraser, Package,
  Download, MoreHorizontal,
}

interface ToolbarButton {
  key: string
  label: string
  desc: string
  iconKey: IconKey
  action: () => void
  priority: number
  getDisabledTooltip: () => string
}

const wpUnavailable = computed(() => !workspaceStore.currentPath)
const globallyDisabled = computed(() => fileListStore.isOperationRunning)

const buttons = computed<ToolbarButton[]>(() => [
  {
    key: 'diff', label: 'toolbar.diff', desc: 'toolbar.descDiff', iconKey: 'FileDiff',
    action: () => router.push('/workspace/diff'), priority: 8,
    getDisabledTooltip: () => {
      if (globallyDisabled.value) return t('toolbar.operationInProgress')
      if (wpUnavailable.value) return t('toolbar.noWorkingCopy')
      if (fileListStore.selectedCount === 0) return t('toolbar.noSelection')
      return ''
    },
  },
  {
    key: 'update', label: 'toolbar.update', desc: 'toolbar.descUpdate', iconKey: 'RefreshCw',
    action: () => emit('open-dialog', 'update'), priority: 7,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'updateToRevision', label: 'toolbar.updateToRevision', desc: 'toolbar.descUpdateToRevision', iconKey: 'ArrowUpCircle',
    action: () => emit('open-dialog', 'update-to-revision'), priority: 6,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'log', label: 'toolbar.log', desc: 'toolbar.descLog', iconKey: 'History',
    action: () => router.push('/workspace/log'), priority: 6,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'switch', label: 'toolbar.switch', desc: 'toolbar.descSwitch', iconKey: 'GitBranch',
    action: () => emit('open-dialog', 'switch'), priority: 5,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'branchTag', label: 'toolbar.branchTag', desc: 'toolbar.descBranchTag', iconKey: 'Layers',
    action: () => emit('open-dialog', 'branch-tag'), priority: 4,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'merge', label: 'toolbar.merge', desc: 'toolbar.descMerge', iconKey: 'GitMerge',
    action: () => emit('open-dialog', 'merge'), priority: 3,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'cleanup', label: 'toolbar.cleanup', desc: 'toolbar.descCleanup', iconKey: 'Eraser',
    action: () => svnStore.cleanup(workspaceStore.currentPath), priority: 1,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
  {
    key: 'export', label: 'toolbar.export', desc: 'toolbar.descExport', iconKey: 'Package',
    action: () => emit('open-dialog', 'export'), priority: 0,
    getDisabledTooltip: () => globallyDisabled.value ? t('toolbar.operationInProgress') : t('toolbar.noWorkingCopy'),
  },
])

const btnDisabled = computed(() => wpUnavailable.value || globallyDisabled.value)

const visibleButtons = computed(() => {
  const count = Math.max(1, buttons.value.length - overflowCount.value)
  return buttons.value.slice(0, count)
})
const overflowButtons = computed(() => buttons.value.slice(visibleButtons.value.length))

onMounted(() => {
  if (!toolbarRef.value) return
  resizeObserver = new ResizeObserver(([entry]) => {
    const avail = entry.contentRect.width - 80
    overflowCount.value = Math.max(0, buttons.value.length - Math.max(1, Math.floor(avail / 88)))
  })
  resizeObserver.observe(toolbarRef.value)
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})
</script>

<template>
  <div ref="toolbarRef" class="h-10 px-3 flex items-center bg-white dark:bg-slate-800 border-b border-slate-200 dark:border-slate-700">
    <div class="flex items-center gap-1 flex-1 overflow-hidden">
      <!-- 检出按钮始终可见（仅操作进行中时禁用） -->
      <button
        class="inline-flex items-center gap-1 px-2.5 py-1 text-xs rounded-md whitespace-nowrap focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
        :class="globallyDisabled
          ? 'opacity-50 cursor-not-allowed text-slate-400 dark:text-slate-500'
          : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-green-600 dark:hover:text-green-400'"
        :disabled="globallyDisabled"
        :aria-label="t('toolbar.checkout')"
        @click="workspaceStore.showCheckoutDialog = true"
      >
        <Download class="w-4 h-4" /><span>{{ t('toolbar.checkout') }}</span>
      </button>

      <!-- 工作副本时：显示所有操作按钮 -->
      <template v-if="workspaceStore.isWorkingCopy">
        <template v-for="btn in visibleButtons" :key="btn.key">
          <el-tooltip
            :content="btnDisabled ? btn.getDisabledTooltip() : t(btn.desc)"
            effect="dark"
            placement="bottom"
          >
            <button
              class="inline-flex items-center gap-1 px-2.5 py-1 text-xs rounded-md whitespace-nowrap focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
              :class="btnDisabled
                ? 'opacity-50 cursor-not-allowed text-slate-400 dark:text-slate-500'
                : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-slate-700 hover:text-green-600 dark:hover:text-green-400'"
              :disabled="btnDisabled"
              :aria-label="t(btn.label)"
              @click="btn.action"
            >
              <component :is="iconMap[btn.iconKey]" class="w-4 h-4" /><span>{{ t(btn.label) }}</span>
            </button>
          </el-tooltip>
        </template>
        <el-dropdown v-if="overflowButtons.length > 0" trigger="click" placement="bottom-end">
          <button class="px-1.5 py-1 text-xs text-slate-400 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none rounded" :aria-label="t('toolbar.moreOperations')">
            <MoreHorizontal class="w-4 h-4" />
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item v-for="btn in overflowButtons" :key="btn.key" :disabled="btnDisabled" @click="btn.action">
                <component :is="iconMap[btn.iconKey]" class="w-4 h-4 mr-2 inline-block align-middle" />
                <span class="align-middle">{{ t(btn.label) }}</span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </template>
    </div>
  </div>
</template>
