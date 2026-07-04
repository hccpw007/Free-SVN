<script setup lang="ts">
/**
 * 进度独立窗口——用于 Tauri 独立窗口的进度展示。
 * 直接监听 Tauri 事件（operation:started/progress/line/completed/cancelled/error），
 * 不依赖主窗口的 Pinia store。
 */
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { invoke } from '@tauri-apps/api/core'
import type { OperationProgress, OperationLine, CancelledPayload, OperationResult } from '@/types/svn'
import { formatNumber } from '@/utils/format'
import FileLineRow from '@/components/svn/FileLineRow.vue'

const { t } = useI18n()

// ── 进度状态 ──
const progress = ref<OperationProgress | null>(null)
const fileLines = ref<OperationLine[]>([])
const isOperationRunning = ref(false)
const isCancelling = ref(false)
const hasEnumeratedFiles = ref(false)
const isOperationCompleted = ref(false)
const wasCancelled = ref(false)
// 取消瞬间的待传输数量（取消后保留显示，不清零）
const cancelledPendingCount = ref(0)
// ── 本地计时器 ──
const elapsedTime = ref('00:00')
let elapsedStartTime = 0
let elapsedTimer: ReturnType<typeof setInterval> | null = null

function startElapsedTimer() {
  stopElapsedTimer()
  elapsedStartTime = Date.now()
  elapsedTimer = setInterval(() => {
    const seconds = Math.floor((Date.now() - elapsedStartTime) / 1000)
    elapsedTime.value = `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`
  }, 1000)
}
function stopElapsedTimer() {
  if (elapsedTimer !== null) { clearInterval(elapsedTimer); elapsedTimer = null }
}

// ── 计算属性 ──
const titleText = computed(() => {
  return progress.value?.operation || t('progress.operationInProgress')
})

// 从文件列表直接统计已完成和待处理数量
const effectiveCompletedCount = computed(() =>
  fileLines.value.filter(l => l.status === 'completed').length
)
const effectivePendingCount = computed(() =>
  wasCancelled.value
    ? cancelledPendingCount.value
    : fileLines.value.filter(l => l.status === 'pending' || l.status === 'in_progress').length
)
// 进度百分比：已完成文件 / 总文件数
const progressPercent = computed(() => {
  const total = fileLines.value.length
  return total > 0 ? Math.round((effectiveCompletedCount.value / total) * 100) : 0
})
// 进度条颜色：取消后为灰色
const progressBarColor = computed(() => {
  if (wasCancelled.value) return '#9ca3af'
  return undefined
})

const sortedFileLines = computed(() => {
  return [...fileLines.value].sort((a, b) => {
    const order = { completed: 0, in_progress: 1, pending: 2, cancelled: 3 }
    return (order[a.status] ?? 2) - (order[b.status] ?? 2)
  })
})

// ── 取消操作 ──
async function cancelOperation() {
  if (isCancelling.value) return
  isCancelling.value = true
  try {
    await invoke('cancel_operation')
  } catch (e: unknown) {
    console.error('[ProgressWindow] 取消失败:', e)
  } finally {
    isCancelling.value = false
  }
}

// ── 窗口拖拽 ──
async function startDrag(e: MouseEvent) {
  if (e.button !== 0) return // 仅左键
  const appWindow = getCurrentWebviewWindow()
  try {
    await appWindow.startDragging()
  } catch (err) {
    console.error('[ProgressWindow] 拖拽失败:', err)
  }
}

// ── 关闭窗口 ──
async function handleClose() {
  // 如果操作仍在运行，只取消不关闭；操作完成了才关闭窗口
  if (isOperationRunning.value) {
    await cancelOperation()
    return
  }
  try {
    // 操作已结束，关闭窗口
    await invoke('close_progress_window')
  } catch (e) {
    console.error('[ProgressWindow] 关闭失败:', e)
  }
}

// ── 事件监听 ──
const unlistenFns: Array<() => void> = []

onMounted(async () => {
  const appWindow = getCurrentWebviewWindow()

  // 确保窗口尺寸正确（macOS Retina 屏下构造参数可能不生效）
  await appWindow.setSize(new LogicalSize(880, 587)).catch(() => {})

  // 窗口关闭请求（原生 X 按钮）：先取消操作，再关闭窗口
  appWindow.onCloseRequested(async (event) => {
    event.preventDefault()
    if (isOperationRunning.value) {
      await cancelOperation()
    }
    // 通过 Rust 端按 label 查找并销毁窗口
    await invoke('close_progress_window')
  })

  const fns = await Promise.all([
    listen<OperationProgress>('operation:progress', (e) => {
      progress.value = e.payload
    }),
    listen('operation:started', () => {
      // 忽略重复的 operation:started（checkout_repo 预枚举阶段和
      // run_svn_with_progress 内部都会发送）
      if (isOperationRunning.value) return
      isOperationRunning.value = true
      isOperationCompleted.value = false
      wasCancelled.value = false
      cancelledPendingCount.value = 0
      fileLines.value = []
      hasEnumeratedFiles.value = false
      isCancelling.value = false
      elapsedTime.value = '00:00'
      startElapsedTimer()
    }),
    listen<OperationLine>('operation:line', (e) => {
      const line = e.payload
      if (line.status === 'pending') {
        hasEnumeratedFiles.value = true
        fileLines.value.push(line)
      } else if (line.status === 'completed' && hasEnumeratedFiles.value) {
        const match = fileLines.value.find(
          l => l.status !== 'completed' && line.filePath.endsWith(l.filePath)
        )
        if (match) {
          for (const l of fileLines.value) {
            if (l.status === 'in_progress') l.status = 'pending'
          }
          match.status = 'in_progress'
          setTimeout(() => { match.status = 'completed' }, 300)
        }
      } else if (line.status === 'completed' && !hasEnumeratedFiles.value) {
        fileLines.value.push(line)
      }
      // 文件列表淘汰策略
      if (fileLines.value.length > 1000) {
        let evictCount = 200
        fileLines.value = fileLines.value.filter(
          (line, i, arr) =>
            line.status !== 'completed' ||
            arr.slice(0, i).filter(l => l.status === 'completed').length > evictCount
        )
      }
    }),
    listen<CancelledPayload>('operation:cancelled', () => {
      isOperationRunning.value = false
      wasCancelled.value = true
      // 保存取消瞬间的待传输数量，不清零
      cancelledPendingCount.value = fileLines.value.filter(
        l => l.status === 'pending' || l.status === 'in_progress'
      ).length
      progress.value = null
      stopElapsedTimer()
      // 将尚未完成的所有文件标记为已取消
      for (const line of fileLines.value) {
        if (line.status !== 'completed') {
          line.status = 'cancelled'
        }
      }
    }),
    listen<OperationResult>('operation:completed', () => {
      isOperationRunning.value = false
      isOperationCompleted.value = true
      progress.value = null
      stopElapsedTimer()
      // 仅将未完成（pending/in_progress）改为 completed，不覆盖已取消的文件
      for (const line of fileLines.value) {
        if (line.status === 'pending' || line.status === 'in_progress') {
          line.status = 'completed'
        }
      }
    }),
    listen('operation:error', () => {
      isOperationRunning.value = false
      isOperationCompleted.value = true
      progress.value = null
      stopElapsedTimer()
      fileLines.value = []
    }),
  ])
  unlistenFns.push(...fns)

  // 通知主窗口进度窗口已就绪，触发补发操作状态
  invoke('progress_window_ready').catch(() => {})

  // 接收主窗口补发的操作状态（解决窗口就绪前已错过的事件）
  listen('progress-window:catchup', (e) => {
    const data = e.payload as {
      isOperationRunning: boolean
      fileLines: OperationLine[]
      progress: OperationProgress | null
    }
    if (data.isOperationRunning) {
      isOperationRunning.value = true
      startElapsedTimer()
      if (data.fileLines?.length > 0) {
        fileLines.value = data.fileLines
        // 仅当被补发的行包含 pending 状态（预枚举场景）时才标记 hasEnumeratedFiles，
        // 否则 update/revert/switch 等非预枚举操作的后续 completed 行会被错误丢弃
        if (data.fileLines.some(l => l.status === 'pending')) {
          hasEnumeratedFiles.value = true
        }
      }
      if (data.progress) {
        progress.value = data.progress
      }
    }
  }).then(fn => unlistenFns.push(fn))
})

onUnmounted(() => {
  unlistenFns.forEach(fn => fn())
})

// ── 自动滚动 ──
const fileListRef = ref<HTMLElement | null>(null)
watch(() => fileLines.value.length, () => {
  if (fileListRef.value) {
    const el = fileListRef.value
    el.scrollTop = el.scrollHeight
  }
})
</script>

<template>
  <div class="h-screen flex flex-col bg-slate-900 select-none overflow-hidden">
    <!-- 标题栏 -->
    <div
      class="flex items-center justify-between px-4 py-3 bg-slate-800 shrink-0"
      @mousedown.prevent="startDrag"
    >
      <span class="text-sm font-semibold text-slate-100 truncate">
        {{ titleText }}
      </span>
    </div>

    <!-- 进度条 -->
    <div class="px-4 pt-4 pb-2">
      <el-progress
        :percentage="progressPercent"
        :stroke-width="12"
        striped
        :striped-flow="isOperationRunning"
        :duration="0.3"
        :color="progressBarColor"
      />
    </div>

    <!-- 统计栏 -->
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-1 px-4 py-2 text-xs text-slate-400 font-mono">
      <div class="truncate">
        <span class="text-slate-500">{{ t('progress.downloadSpeed') }}:</span>
        {{ progress?.speed || '-' }}
      </div>
      <div class="truncate">
        <span class="text-slate-500">{{ t('progress.completedCount') }}:</span>
        {{ formatNumber(effectiveCompletedCount) }}
      </div>
      <div class="truncate">
        <span class="text-slate-500">{{ t('progress.pendingCount') }}:</span>
        {{ formatNumber(effectivePendingCount) }}
      </div>
      <div class="truncate">
        <span class="text-slate-500">{{ t('progress.elapsed') }}:</span>
        {{ elapsedTime }}
      </div>
    </div>

    <!-- 分隔线 -->
    <div class="mx-4 border-t border-slate-700"></div>

    <!-- 文件滚动区 -->
    <div
      ref="fileListRef"
      class="flex-1 overflow-y-auto px-2 py-1 min-h-0"
    >
      <div
        v-if="fileLines.length === 0"
        class="flex items-center justify-center h-20 text-sm text-slate-500"
      >
        {{ progress?.stage || t('progress.processing') }}
      </div>
      <FileLineRow
        v-for="(line) in sortedFileLines"
        :key="line.filePath"
        :filePath="line.filePath"
        :status="line.status"
      />
    </div>

    <!-- 分隔线 -->
    <div class="mx-4 border-t border-slate-700"></div>

    <!-- 底栏 -->
    <div class="flex items-center justify-between px-4 py-3 shrink-0">
      <span class="text-xs text-slate-500">
        {{ isOperationCompleted ? t('progress.completed') : t('progress.autoClose') }}
      </span>
      <button
        v-if="!isCancelling"
        class="px-4 py-1.5 rounded-md text-sm font-medium transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-slate-800"
        :class="isOperationRunning
          ? 'bg-red-800/40 text-red-400 hover:bg-red-800/60 focus:ring-red-400'
          : 'bg-green-800/40 text-green-400 hover:bg-green-800/60 focus:ring-green-400'"
        @click="handleClose"
      >
        {{ isOperationRunning ? t('common.cancel') : t('common.close') }}
      </button>
      <button
        v-else
        class="px-4 py-1.5 rounded-md text-sm font-medium bg-slate-700 text-slate-400 cursor-not-allowed"
        disabled
      >
        {{ t('progress.cancelling') }}
      </button>
    </div>
  </div>
</template>
