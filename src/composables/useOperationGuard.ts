/**
 * useOperationGuard - 读取 fileListStore.isOperationRunning 提供全局禁用状态
 *
 * 使用示例：
 * const { isBlocked, tooltip } = useOperationGuard()
 * // ToolBar: <button :disabled="isBlocked" :title="tooltip">
 */
import { computed } from 'vue'
import { useFileListStore } from '@/stores/fileList'
import { useI18n } from 'vue-i18n'

export function useOperationGuard() {
  const fileListStore = useFileListStore()
  const { t } = useI18n()

  const blocked = computed(() => fileListStore.isOperationRunning)

  return {
    isBlocked: blocked,
    tooltip: computed(() => (blocked.value ? t('progress.operationInProgress') : '')),
  }
}
