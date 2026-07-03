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
    Promise.all([
      listen<OperationProgress>('operation:progress', e => { progress.value = e.payload }),
      listen('operation:started', () => {
        isOperationRunning.value = true
        fileLines.value = []
        useFileListStore().isOperationRunning = true
      }),
      listen<OperationLine>('operation:line', e => {
        // v3 新增：operation:line 事件，携带单行文件信息
        fileLines.value.push(e.payload)
      }),
      listen<CancelledPayload>('operation:cancelled', () => {
        // v3 新增：operation:cancelled 事件，替代 operation:completed + result:"cancelled"
        isOperationRunning.value = false; progress.value = null
        useFileListStore().isOperationRunning = false
      }),
      listen<OperationResult>('operation:completed', () => {
        isOperationRunning.value = false; progress.value = null
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
