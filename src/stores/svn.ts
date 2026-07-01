import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getErrorMessage } from '@/types/error-codes'
import type {
  FileItem, RepoInfo, DiffResult, OperationProgress,
  OperationResult, BlameLine, LogEntry,
} from '@/types/svn'
import type { AppSettings } from '@/types/settings'
import { useFileListStore } from './fileList'

export const useSvnStore = defineStore('svn', () => {
  const progress = ref<OperationProgress | null>(null)
  const isLoading = ref(false)
  const isOperationRunning = ref(false)
  let initialized = false
  /** 存储 listen() 返回的 UnlistenFn 句柄，用于在 store 销毁时取消注册事件监听 */
  const unlistenFns: Array<() => void> = []

  // ── v5 认证失败→AuthDialog 联动状态（业务设计 §8.3 + 视觉设计 §4.2） ──
  /**
   * 保存认证失败上下文，供前端 App.vue 监听并自动弹出 AuthDialog
   * call() 捕获到 SVN_AUTH_FAILED / E170001 时填充此对象
   */
  interface AuthFailedContext {
    command: string
    args?: Record<string, unknown>
    errorMessage: string
  }
  const authFailed = ref(false)
  const authContext = ref<AuthFailedContext | null>(null)

  /** 使用凭据重试上次失败的操作 */
  async function retryAuth(username: string, password: string, saveToCache: boolean): Promise<boolean> {
    if (!authContext.value) return false
    const ctx = authContext.value
    try {
      // 将 credentials 注入到原 args 中
      const retryArgs = { ...(ctx.args || {}), credentials: { username, password, saveToCache } }
      await invoke(ctx.command, retryArgs)
      authFailed.value = false
      authContext.value = null
      return true
    } catch (err) {
      return false
    }
  }

  /** 取消认证重试 */
  function cancelAuth() {
    authFailed.value = false
    authContext.value = null
  }

  // ── 操作取消/重试错误消息常量（供外部通过错误码而非文本字符串判断） ──
  const AUTH_CANCELLED_MSG = 'operation cancelled'
  const AUTH_RETRY_FAILED_MSG = 'auth retry failed'

  function initEventListeners() {
    if (initialized) return
    initialized = true
    // 使用 Promise.all 收集所有 listen() 返回的 UnlistenFn，供后续清理
    Promise.all([
      listen<OperationProgress>('operation:progress', e => { progress.value = e.payload }),
      listen('operation:started', () => {
        isOperationRunning.value = true
        useFileListStore().isOperationRunning = true
      }),
      listen('operation:completed', () => {
        isOperationRunning.value = false; progress.value = null
        useFileListStore().isOperationRunning = false
      }),
      listen('operation:error', () => {
        isOperationRunning.value = false; progress.value = null
        useFileListStore().isOperationRunning = false
      }),
    ]).then(fns => { unlistenFns.push(...fns) })
      .catch(err => console.error('[svn store] Failed to register event listeners:', err))
  }

  /** 销毁事件监听：调用所有 UnlistenFn 取消注册，防止 HMR 热重载场景下孤立监听 */
  function destroyEventListeners() {
    unlistenFns.forEach(fn => fn())
    unlistenFns.length = 0
    initialized = false
  }

  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    try { return await invoke<T>(command, args) }
    catch (err) {
      const msg = getErrorMessage(err)
      // 提取原始错误码，绕过错码翻译层，避免 locale 依赖导致检测失效
      const rawError: string = err && typeof err === 'object'
        ? String((err as Record<string, unknown>).error ?? '')
        : ''
      // SVN_AUTH_FAILED(E170001) → 保存失败上下文供 AuthDialog 自动弹出
      if (rawError === 'SVN_AUTH_FAILED' || rawError.includes('E170001')) {
        authContext.value = { command, args, errorMessage: msg }
        authFailed.value = true
      }
      // SVN_OPERATION_IN_PROGRESS → 队列冲突，抛出携带错误码的异常供前端 toast 展示
      // 前端组件应 catch 此错误并通过 ElMessage.warning(t('error.SVN_OPERATION_IN_PROGRESS')) 显示提示
      if (rawError === 'SVN_OPERATION_IN_PROGRESS') {
        throw new Error('SVN_OPERATION_IN_PROGRESS')
      }
      throw new Error(msg)
    }
  }

  // ── 每个方法对应一个 Rust command，参数命名统一 camelCase ──
  // 注意：Tauri 2 invoke 传扁平参数（不套 { params: ... } 层），
  // 后端 Rust struct 使用 #[serde(rename_all = "camelCase")] 自动将 invoke 接收到的
  // camelCase 字段名映射为 Rust 的 snake_case 字段名。
  //
  // v2 修复：所有方法统一使用对象参数风格（与 7.2 services/svn.ts 完全对齐）

  async function getStatus(path: string) { return call<FileItem[]>('get_status', { path }) }
  async function getInfo(path: string) { return call<RepoInfo>('get_info', { path }) }
  async function getDiff(params: { path: string; revision1?: number; revision2?: number }) { return call<DiffResult>('get_diff', params) }
  async function getLog(params: { path: string; limit?: number; revision?: number; search?: string }) {
    const entries = await call<LogEntry[]>('get_log', params)
    return { entries, totalCount: entries.length }
  }
  async function getBlame(params: { path: string; revision?: number }) { return call<BlameLine[]>('get_blame', params) }
  async function checkoutRepo(params: { url: string; targetPath: string; depth?: string; ignoreExternals?: boolean; credentials?: { username: string; password: string; saveToCache: boolean } }) {
    return call<string>('checkout_repo', {
      url: params.url,
      target_path: params.targetPath,
      depth: params.depth,
      ignore_externals: params.ignoreExternals,
      credentials: params.credentials,
    })
  }
  async function updateWorkspace(params: { path: string; revision?: number; depth?: string; ignoreExternals?: boolean }) {
    return call<string>('update_workspace', {
      path: params.path,
      revision: params.revision,
      depth: params.depth,
      ignore_externals: params.ignoreExternals,
    })
  }
  async function commit(params: { paths: string[]; message: string; keepLocks?: boolean }) { return call<OperationResult>('create_commit', params) }
  async function addFiles(paths: string[]) { return call<string>('add_files', { paths }) }
  async function deleteFiles(params: { paths: string[]; keepLocal?: boolean }) {
    return call<string>('delete_files', {
      paths: params.paths,
      keep_local: params.keepLocal,
    })
  }
  async function revertFiles(paths: string[]) { return call<string>('revert_files', { paths }) }
  async function resolveConflict(params: { path: string; resolution: string }) { return call<string>('resolve_conflict', params) }
  async function setIgnore(params: { path: string; pattern: string }) { return call<string>('set_ignore', params) }
  async function switchBranch(params: { path: string; targetUrl: string; ignoreAncestry?: boolean }) {
    return call<OperationResult>('switch_branch', {
      path: params.path,
      target_url: params.targetUrl,
      ignore_ancestry: params.ignoreAncestry,
      credentials: (params as any).credentials,
    })
  }
  async function copyBranchTag(params: { srcUrl: string; dstUrl: string; message: string; revision?: number }) {
    return call<OperationResult>('copy_branch_tag', {
      src_url: params.srcUrl,
      dst_url: params.dstUrl,
      message: params.message,
      revision: params.revision,
      credentials: (params as any).credentials,
    })
  }
  async function mergeBranch(params: { srcUrl: string; revStart?: number; revEnd?: number; targetPath?: string }) {
    return call<OperationResult>('merge_branch', {
      src_url: params.srcUrl,
      rev_start: params.revStart,
      rev_end: params.revEnd,
      target_path: params.targetPath,
      credentials: (params as any).credentials,
    })
  }
  async function cleanup(path: string) { return call<string>('cleanup_workspace', { path }) }
  async function exportWorkspace(params: { path: string; targetDir: string; revision?: number; ignoreExternals?: boolean }) {
    return call<string>('export_workspace', {
      path: params.path,
      target_dir: params.targetDir,
      revision: params.revision,
      ignore_externals: params.ignoreExternals,
      credentials: (params as any).credentials,
    })
  }
  async function lockFiles(params: { paths: string[]; message?: string }) { return call<string>('lock_files', params) }
  async function unlockFiles(paths: string[]) { return call<string>('unlock_files', { paths }) }
  async function relocate(params: { path: string; fromUrl: string; toUrl: string }) {
    return call<string>('relocate_repo', {
      path: params.path,
      from_url: params.fromUrl,
      to_url: params.toUrl,
      credentials: (params as any).credentials,
    })
  }
  async function propertyOps(params: { path: string; propName?: string; action?: string; propValue?: string }) { return call<string>('property_ops', params) }
  async function cancelOperation() { return call<string>('cancel_operation') }
  async function getLogs() { return call<string>('get_logs') }
  async function exportLogs(targetPath: string) { return call<void>('export_logs', { target_path: targetPath }) }
  async function loadSettings() { return call<AppSettings>('load_settings') }
  async function saveSettings(settings: AppSettings) { return call<void>('save_settings', { settings }) }
  async function testConnection(params: { url: string; username: string; password: string }) {
    return call<string>('test_connection', {
      url: params.url,
      credentials: { username: params.username, password: params.password, saveToCache: false },
    })
  }
  async function saveCredentials(params: { url: string; username: string; password: string }) {
    return call<void>('save_credentials', {
      url: params.url,
      credentials: { username: params.username, password: params.password, saveToCache: true },
    })
  }
  async function clearCredentials(url: string) { return call<void>('clear_credentials', { url }) }
  async function listBranches(url: string) { return call<string[]>('list_branches', { url }) }

  return {
    progress, isOperationRunning, initEventListeners, destroyEventListeners,
    getStatus, getInfo, getDiff, getLog, getBlame,
    checkoutRepo, updateWorkspace, commit, addFiles, deleteFiles,
    revertFiles, resolveConflict, setIgnore,
    switchBranch, copyBranchTag, mergeBranch,
    cleanup, exportWorkspace, lockFiles, unlockFiles,
    relocate, propertyOps,
    cancelOperation, getLogs, exportLogs,
    loadSettings, saveSettings,
    testConnection, saveCredentials, clearCredentials, listBranches,
    // v5 认证失败→AuthDialog 联动
    authFailed, authContext, retryAuth, cancelAuth,
    // 操作取消/重试错误消息常量
    AUTH_CANCELLED_MSG, AUTH_RETRY_FAILED_MSG,
  }
})
