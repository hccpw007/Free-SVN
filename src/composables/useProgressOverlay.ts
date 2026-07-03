import { ref, computed } from 'vue'
import { useSvnStore } from '@/stores/svn'
import type { OperationProgress } from '@/types/svn'

interface ProgressOverlayState {
  showProgress: boolean
  progressOperation: string
  progressPercent: number
  progressSpeed: string
  progressElapsed: string
  progressCompleted: number
  progressPending: number
  fileLines: Array<{ operation: string; line: string; status: 'completed' | 'in_progress' }>
  errorMessage: string
}

export function useProgressOverlay() {
  const svnStore = useSvnStore()

  const overlayState = ref<ProgressOverlayState>({
    showProgress: false,
    progressOperation: '',
    progressPercent: 0,
    progressSpeed: '',
    progressElapsed: '',
    progressCompleted: 0,
    progressPending: 0,
    fileLines: [],
    errorMessage: '',
  })

  const isVisible = computed(() => svnStore.isOperationRunning || overlayState.value.errorMessage !== '')

  const progress = computed<OperationProgress | null>(() => {
    if (!svnStore.progress) return null
    return {
      operation: svnStore.progress.operation,
      percent: svnStore.progress.percent,
      stage: svnStore.progress.stage,
      fileCount: svnStore.progress.fileCount,
      completedCount: svnStore.progress.completedCount,
      pendingCount: svnStore.progress.pendingCount,
      speed: svnStore.progress.speed,
      elapsed: svnStore.progress.elapsed,
      currentLines: svnStore.progress.currentLines,
    }
  })

  function showError(message: string) {
    overlayState.value.errorMessage = message
    overlayState.value.showProgress = true
  }

  function dismiss() {
    overlayState.value.showProgress = false
    overlayState.value.fileLines = []
    overlayState.value.errorMessage = ''
  }

  return {
    overlayState,
    isVisible,
    progress,
    showError,
    dismiss,
    cancelOperation: () => svnStore.cancelOperation(),
  }
}
