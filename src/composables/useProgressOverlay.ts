// useProgressOverlay composable — 仅管理弹窗本地 UI 状态
// 跨组件共享的前后端通信状态（进度数据、事件监听）保持在 svnStore 中

import { ref, computed } from 'vue'
import { useSvnStore } from '@/stores/svn'

export function useProgressOverlay() {
  const svnStore = useSvnStore()

  // ── 弹窗本地 UI 状态 ──

  const isDragging = ref(false)
  const isCancelling = ref(false)  // 取消按钮 loading 状态
  const panelPosition = ref({ x: 0, y: 0 })
  const panelOffset = ref({ x: 0, y: 0 })     // 拖拽偏移量
  const dragStartPos = ref({ x: 0, y: 0 })     // 拖拽起始鼠标位置
  const dragStartOffset = ref({ x: 0, y: 0 })  // 拖拽起始面板偏移
  const fileListRef = ref<HTMLElement | null>(null)

  // ── 从 svnStore 读取 IPC 通信数据 ──

  const isVisible = computed(() => svnStore.isOperationRunning)
  const progress = computed(() => svnStore.progress)
  const fileLines = computed(() => svnStore.fileLines)

  // ── 操作请求防抖 ──
  const operationRequested = ref(false)

  /** 尝试请求操作（防抖，同一时刻只允许一个进度弹窗激活） */
  function tryRequestOperation(): boolean {
    if (operationRequested.value || svnStore.isOperationRunning) {
      return false  // 已有操作在进行中
    }
    operationRequested.value = true
    return true
  }

  /** 释放操作请求 */
  function releaseOperation() {
    operationRequested.value = false
  }

  // ── 拖拽相关 ──

  function onDragStart(e: MouseEvent) {
    isDragging.value = true
    dragStartPos.value = { x: e.clientX, y: e.clientY }
    dragStartOffset.value = { ...panelOffset.value }
  }

  function onDragMove(e: MouseEvent) {
    if (!isDragging.value) return
    // 使用 CSS transform 而非 top/left，避免重排
    const dx = e.clientX - dragStartPos.value.x + dragStartOffset.value.x
    const dy = e.clientY - dragStartPos.value.y + dragStartOffset.value.y
    // 边界约束：保持面板在视口内（由 CSS max/min 配合 transform 实现）
    panelOffset.value = { x: dx, y: dy }
  }

  function onDragEnd() {
    isDragging.value = false
  }

  // ── 取消操作 ──

  async function cancelOperation() {
    if (isCancelling.value) return
    isCancelling.value = true
    try {
      await svnStore.cancelOperation()
    } catch (e: unknown) {
      console.error('[useProgressOverlay] 取消失败:', e)
    } finally {
      isCancelling.value = false
    }
  }

  // ── 自动滚动 ──

  function autoScrollToBottom() {
    if (!fileListRef.value) return
    const el = fileListRef.value
    // 检测用户是否已手动向上滚动浏览历史行
    const threshold = 20
    const isNearBottom = el.scrollTop + el.clientHeight >= el.scrollHeight - threshold
    if (isNearBottom) {
      el.scrollTop = el.scrollHeight
    }
  }

  return {
    // 读写状态
    isDragging, isCancelling, panelPosition, panelOffset,
    dragStartPos, dragStartOffset, fileListRef,
    // 只读计算状态
    isVisible, progress, fileLines,
    // 操作防抖
    tryRequestOperation, releaseOperation,
    // 方法
    onDragStart, onDragMove, onDragEnd,
    cancelOperation, autoScrollToBottom,
  }
}
