<script setup lang="ts">
import { ref } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useWorkspaceStore } from '@/stores/workspace'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const fileListStore = useFileListStore()
const workspaceStore = useWorkspaceStore()
const svnStore = useSvnStore()

// 工作副本绝对路径，用于将相对路径转为绝对路径
const cwd = workspaceStore.currentPath

const commitMessage = ref('')
const isSubmitting = ref(false)
const submitError = ref('')

// 将文件列表中的相对路径转为基于工作副本根目录的绝对路径
function toAbsolute(relativePaths: string[]): string[] {
  return relativePaths.map(p => `${cwd}/${p}`)
}

// 提交逻辑：自动 add unversioned + delete missing → commit
async function handleCommit() {
  const selectedPaths = Array.from(fileListStore.selectedPaths)
  if (selectedPaths.length === 0) return

  // 提交信息为空时二次确认
  if (!commitMessage.value.trim()) {
    try {
      await ElMessageBox.confirm(
        t('file.emptyCommitConfirm'),
        t('common.confirm'),
        { confirmButtonText: t('common.confirm'), cancelButtonText: t('common.cancel'), type: 'warning' }
      )
    } catch { return }
  }

  isSubmitting.value = true
  submitError.value = ''
  try {
    // 构建 path → status 映射，无须重新调用 SVN
    const pathStatus = new Map(
      fileListStore.files
        .filter(f => fileListStore.selectedPaths.has(f.path))
        .map(f => [f.path, f.status])
    )

    // 分离需要预处理的文件
    const unversionedPaths = selectedPaths.filter(p => pathStatus.get(p) === 'unversioned')
    const missingPaths = selectedPaths.filter(p => pathStatus.get(p) === 'missing')

    // 对未加入文件自动执行 svn add（使用绝对路径，避免 cwd 错误）
    if (unversionedPaths.length > 0) {
      await svnStore.addFiles(toAbsolute(unversionedPaths))
    }

    // 对缺失文件自动执行 svn delete
    if (missingPaths.length > 0) {
      await svnStore.deleteFiles({ paths: toAbsolute(missingPaths), keepLocal: false })
    }

    // 提交所有文件（使用绝对路径）
    const result = await svnStore.commit({
      paths: toAbsolute(selectedPaths),
      message: commitMessage.value,
      keepLocks: false,
    })
    fileListStore.selectedPaths = new Set()
    commitMessage.value = ''
    const revision = result.detail?.replace(/[^0-9]/g, '')
    ElMessage.success(t('file.commitSuccess', { rev: revision || '' }))
    emit('close')
  } catch (e: unknown) {
    const error = e as { errorCode?: string; code?: string }
    const errorCode = error?.errorCode || error?.code || ''
    const errorMap: Record<string, string> = {
      'SVN_EXEC_FAILED': 'error.SVN_EXEC_FAILED',
      'SVN_NOT_WORKING_COPY': 'error.notWorkingCopy',
      'SVN_AUTH_FAILED': 'error.authFailed',
      'SVN_CANCELED': 'error.canceled',
      'SVN_OUT_OF_DATE': 'error.outOfDate',
      'SVN_CONFLICT': 'error.conflictDetected',
    }
    const errorKey = errorMap[errorCode] || 'error.SVN_EXEC_FAILED'
    submitError.value = t(errorKey)
  } finally { isSubmitting.value = false }
}
</script>

<template>
  <el-dialog
    :model-value="true"
    :title="t('dialog.commitTitle', { count: fileListStore.selectedPaths.size })"
    width="480px"
    :close-on-click-modal="false"
    :close-on-press-escape="!isSubmitting"
    @close="emit('close')"
  >
    <div class="flex flex-col gap-3">
      <!-- 多行提交信息输入 -->
      <textarea
        v-model="commitMessage"
        :placeholder="t('file.commitPlaceholder')"
        class="w-full min-h-[120px] max-h-[300px] p-3 text-xs font-mono rounded-md border border-slate-300 dark:border-slate-600 bg-white dark:bg-slate-800 text-slate-800 dark:text-slate-200 resize-y focus:outline-none focus:ring-2 focus:ring-green-500/30 focus:border-green-500 transition-colors"
        :class="commitMessage.split('\n').some(l => l.length > 72) ? '!border-amber-400' : ''"
      />
      <!-- 错误提示 -->
      <div v-if="submitError" class="text-xs text-red-600 bg-red-50 dark:bg-red-900/20 p-2 rounded border border-red-200 dark:border-red-800">
        {{ submitError }}
      </div>
    </div>

    <template #footer>
      <el-button
        size="default"
        @click="emit('close')"
        :disabled="isSubmitting"
      >
        {{ t('common.cancel') }}
      </el-button>
      <el-button
        size="default"
        type="primary"
        :loading="isSubmitting"
        :disabled="fileListStore.selectedPaths.size === 0"
        @click="handleCommit"
      >
        {{ isSubmitting ? t('common.submitting') : t('common.submit') }}
      </el-button>
    </template>
  </el-dialog>
</template>
