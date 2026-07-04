<script setup lang="ts">
/** 更新到版本对话框——选择目标版本号和深度。 */
import { ref, onMounted } from 'vue'
import { useSvnStore } from '@/stores/svn'
import { useWorkspaceStore } from '@/stores/workspace'
import { useFileListStore } from '@/stores/fileList'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import type { LogEntry } from '@/types/svn'

const { t } = useI18n()
const svnStore = useSvnStore()
const workspaceStore = useWorkspaceStore()
const fileListStore = useFileListStore()

const emit = defineEmits<{ close: [] }>()

// 表单状态
const currentRevision = ref(workspaceStore.currentRevision)
const targetRevision = ref<number | undefined>(undefined)
const updateDepth = ref('') // '' = 不修改
const isUpdating = ref(false)
const recentRevisions = ref<LogEntry[]>([])

const SUPPORTED_DEPTHS = [
  { value: '', label: 'dialog.depthUnchanged' },
  { value: 'infinity', label: 'dialog.depthInfinity' },
  { value: 'immediates', label: 'dialog.depthImmediates' },
  { value: 'children', label: 'dialog.depthChildren' },
  { value: 'empty', label: 'dialog.depthEmpty' },
]

/** 格式化修订版本标签 */
function formatRevision(entry: LogEntry): string {
  const rev = `r${entry.revision}`
  const msg = entry.msg ? entry.msg.replace(/\n.*$/s, '').substring(0, 60) : ''
  return `${rev}  ${msg}`
}

// 挂载时获取最近日志
onMounted(async () => {
  if (!workspaceStore.currentPath) return
  try {
    const logResult = await svnStore.getLog({ path: workspaceStore.currentPath, limit: 30 })
    recentRevisions.value = (logResult && Array.isArray(logResult) ? logResult : logResult?.entries) ?? []
  } catch {
    // 日志获取失败不影响对话框使用
  }
})

async function handleUpdate() {
  if (!workspaceStore.currentPath) return
  isUpdating.value = true
  try {
    await svnStore.updateWorkspace({
      path: workspaceStore.currentPath,
      revision: targetRevision.value,
      depth: updateDepth.value || undefined,
    })
    emit('close')
    fileListStore.refresh()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    ElMessage.error(msg)
  } finally {
    isUpdating.value = false
  }
}
</script>

<template>
  <!-- 更新到指定版本的对话框 -->
  <el-dialog :model-value="true" :title="t('dialog.updateRevision')" width="420px"
    :close-on-click-modal="false" @close="emit('close')">
    <!-- 版本更新表单字段 -->
    <div class="flex flex-col gap-4">
      <!-- 当前版本 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.currentRevision') }}</label>
        <el-input :model-value="String(currentRevision)" size="small" disabled />
      </div>

      <!-- 目标版本 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.targetRevision') }}</label>
        <el-select v-model="targetRevision" size="small" class="!w-full" clearable :placeholder="t('dialog.latestVersion')">
          <!-- HEAD 选项 -->
          <el-option :value="undefined" :label="t('dialog.latestVersion')" />
          <el-option disabled :value="0" label="──────────" style="pointer-events: none;" />
          <el-option v-for="entry in recentRevisions" :key="entry.revision" :value="entry.revision" :label="formatRevision(entry)" />
        </el-select>
      </div>

      <!-- 深度 -->
      <div>
        <label class="text-xs font-medium text-slate-700 dark:text-slate-300 mb-1 block">{{ t('dialog.depth') }}</label>
        <el-select v-model="updateDepth" size="small" class="!w-full">
          <el-option v-for="opt in SUPPORTED_DEPTHS" :key="opt.value" :value="opt.value" :label="t(opt.label)" />
        </el-select>
      </div>
    </div>

    <!-- 底部按钮区域 -->
    <template #footer>
      <el-button size="default" class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="emit('close')" :disabled="isUpdating">
        {{ t('dialog.cancel') }}
      </el-button>
      <el-button size="default" type="primary" :loading="isUpdating"
        class="focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none" @click="handleUpdate">
        {{ t('dialog.update') }}
      </el-button>
    </template>
  </el-dialog>
</template>
