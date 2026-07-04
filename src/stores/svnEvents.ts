/**
 * SVN 事件监听及认证状态 Store——从 svnStore 拆分出的独立状态管理。
 * 包含：进度状态、文件行列表、操作进行中守卫、认证失败联动、事件注册/销毁。
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import type {
  OperationProgress, OperationLine, CancelledPayload, OperationResult,
} from '@/types/svn'
import { useFileListStore } from './fileList'
import { wrappedInvoke } from '@/services/svn'

export interface AuthFailedContext {
  command: string
  args?: Record<string, unknown>
  errorMessage: string
}

/** SVN 事件监听及认证状态 Store */
export const useSvnEventsStore = defineStore('svnEvents', () => {
  // ── 进度与操作进行中状态 ──
  const progress = ref<OperationProgress | null>(null)
  const isOperationRunning = ref(false)
  const fileLines = ref<OperationLine[]>([])

  // ── 事件监听器生命周期 ──
  let initialized = false
  const unlistenFns: Array<() => void> = []

  // ── 认证失败→AuthDialog 联动状态 ──
  const authFailed = ref(false)
  const authContext = ref<AuthFailedContext | null>(null)
  const showUpdateRevisionDialog = ref(false)

  // ── 操作进行中守卫 ──
  function checkOperationRunning(): boolean {
    return isOperationRunning.value
  }

  // ── 认证联动 ──

  /** 使用凭据重试上次失败的操作 */
  async function retryAuth(username: string, password: string, saveToCache: boolean): Promise<boolean> {
    if (!authContext.value) return false
    const ctx = authContext.value
    try {
      // 通过 services/svn.ts 的 wrappedInvoke 重试（经由唯一 invoke 入口）
      // 注意：Tauri 2 要求参数 key 匹配 Rust 参数名（params:），因此需要 { params: ... } 包裹
      const retryArgs = { ...(ctx.args || {}), credentials: { username, password, saveToCache } }
      await wrappedInvoke(ctx.command, { params: retryArgs })
      authFailed.value = false
      authContext.value = null
      return true
    } catch (e) {
      // 抛出实际错误，让 AuthDialog 能显示具体失败原因而非通用提示
      const detail = typeof e === 'string' ? e : e instanceof Error ? e.message : String(e)
      throw new Error(detail)
    }
  }

  function cancelAuth() {
    authFailed.value = false
    authContext.value = null
  }

  // ── 事件监听器注册与销毁 ──

  function initEventListeners() {
    if (initialized) return
    initialized = true

    // ── 预枚举文件模式标记（来自 checkout.rs 的 svn list） ──
    let hasEnumeratedFiles = false

    /** 将第一个 pending 文件标记为 in_progress，其余非 completed 重置为 pending */
    function markNextInProgress() {
      for (const l of fileLines.value) {
        if (l.status === 'in_progress') l.status = 'pending'
      }
      for (const l of fileLines.value) {
        if (l.status === 'pending') {
          l.status = 'in_progress'
          break
        }
      }
    }

    Promise.all([
      listen<OperationProgress>('operation:progress', e => { progress.value = e.payload }),
      listen('operation:started', () => {
        // 忽略重复的 operation:started（checkout_repo 预枚举阶段和
        // run_svn_with_progress 内部都会发送）
        if (isOperationRunning.value) return
        isOperationRunning.value = true
        fileLines.value = []
        hasEnumeratedFiles = false
        useFileListStore().isOperationRunning = true
      }),
      listen<OperationLine>('operation:line', e => {
        const line = e.payload
        if (line.status === 'pending') {
          // 来自 checkout.rs 的 svn list 预枚举 → 追加到文件列表
          hasEnumeratedFiles = true
          fileLines.value.push(line)
          if (fileLines.value.length === 1) {
            fileLines.value[0].status = 'in_progress'
          }
        } else if (line.status === 'completed' && hasEnumeratedFiles) {
          // 从 progress.rs 的检出 stdout 匹配路径更新状态
          const match = fileLines.value.find(
            l => l.status !== 'completed' && line.filePath.endsWith(l.filePath)
          )
          if (match) match.status = 'completed'
          markNextInProgress()
        } else if (line.status === 'completed' && !hasEnumeratedFiles) {
          // 非 checkout 操作（update/commit/switch）：直接追加 completed 行
          fileLines.value.push(line)
        }
        // 文件列表淘汰策略：超过 1000 行时淘汰最旧的 200 行已完成
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
        isOperationRunning.value = false; progress.value = null
        useFileListStore().isOperationRunning = false
      }),
      listen<OperationResult>('operation:completed', () => {
        isOperationRunning.value = false; progress.value = null
        // 操作完成时，将所有剩余文件标记为 completed
        for (const line of fileLines.value) {
          if (line.status !== 'completed') {
            line.status = 'completed'
          }
        }
        useFileListStore().isOperationRunning = false
      }),
      listen('operation:error', () => {
        isOperationRunning.value = false; progress.value = null
        fileLines.value = []
        useFileListStore().isOperationRunning = false
      }),
    ]).then(fns => { unlistenFns.push(...fns) })
      .catch(err => console.error('[svn events] Failed to register event listeners:', err))
  }

  function destroyEventListeners() {
    unlistenFns.forEach(fn => fn())
    unlistenFns.length = 0
    initialized = false
  }

  return {
    progress, isOperationRunning, fileLines,
    authFailed, authContext, showUpdateRevisionDialog,
    checkOperationRunning,
    retryAuth, cancelAuth, initEventListeners, destroyEventListeners,
  }
})
