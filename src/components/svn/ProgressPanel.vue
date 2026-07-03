<script setup lang="ts">
/** 进度面板——全屏进度弹窗（进度条+取消按钮+错误提示）。 */
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { useSvnStore } from '@/stores/svn'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const svnStore = useSvnStore()

const isCancelling = ref(false)
const errorMessage = ref('')
const progress = computed(() => svnStore.progress)
const isRunning = computed(() => svnStore.isOperationRunning)
const showError = computed(() => errorMessage.value !== '')

async function handleCancel() {
  isCancelling.value = true
  try {
    await svnStore.cancelOperation()
    ElMessage.success(t('progress.cancelConfirmed'))
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e)
    console.error('[ProgressPanel] 取消失败:', msg)
  } finally {
    isCancelling.value = false
  }
}

function showErrorMsg(msg: string) {
  errorMessage.value = msg
}

function dismiss() {
  errorMessage.value = ''
}
</script>

<template>
  <Teleport to="body">
    <!-- Error state -->
    <div v-if="showError" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60">
      <div class="w-80 p-6 rounded-lg bg-white dark:bg-slate-800 shadow-lg border border-slate-200 dark:border-slate-700">
        <h3 class="text-base font-semibold text-slate-900 dark:text-slate-50 mb-4">
          {{ t('progress.error') }}
        </h3>
        <p class="text-sm text-red-600 dark:text-red-400 mb-5">{{ errorMessage }}</p>
        <button
          class="w-full py-2 px-4 rounded-md text-sm font-medium transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400 hover:bg-blue-100 dark:hover:bg-blue-900/40"
          @click="dismiss"
        >
          {{ t('common.close') }}
        </button>
      </div>
    </div>
    <!-- Progress state -->
    <div v-if="isRunning && !showError" class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 dark:bg-black/60">
      <div class="w-80 p-6 rounded-lg bg-white dark:bg-slate-800 shadow-lg border border-slate-200 dark:border-slate-700">
        <h3 class="text-base font-semibold text-slate-900 dark:text-slate-50 mb-4">
          {{ progress?.operation || t('progress.operationInProgress') }}
        </h3>
        <el-progress :percentage="progress?.percent ?? 0" :stroke-width="8" striped striped-flow :duration="3" class="mb-3" />
        <p class="text-sm text-slate-500 dark:text-slate-400 mb-1">{{ progress?.stage || t('progress.processing') }}</p>
        <p class="text-xs text-slate-400 dark:text-slate-500 font-mono mb-5">{{ t('progress.filesProcessed', { count: progress?.fileCount ?? 0 }) }}</p>
        <button
          class="w-full py-2 px-4 rounded-md text-sm font-medium transition-colors duration-150 focus:ring-2 focus:ring-offset-2 focus:ring-blue-400 focus:outline-none"
          :class="isCancelling
            ? 'bg-slate-200 dark:bg-slate-700 text-slate-400 cursor-not-allowed'
            : 'bg-red-50 dark:bg-red-900/20 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-900/40'"
          :disabled="isCancelling"
          @click="handleCancel"
        >
          {{ isCancelling ? t('progress.cancelling') : t('common.cancel') }}
        </button>
      </div>
    </div>
  </Teleport>
</template>
