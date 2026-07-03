import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getErrorMessage } from '@/types/error-codes'
import type {
  FileItem, RepoInfo, DiffResult, OperationProgress, OperationLine,
  CancelledPayload, OperationResult,
  BlameLine, LogEntry, SvnCredentials,
} from '@/types/svn'
import type { AppSettings } from '@/types/settings'
import { useFileListStore } from './fileList'
// ── 统一从 services/svn.ts 导入（唯一 invoke 入口） ──
import {
  wrappedInvoke,
  getStatus as servicesGetStatus,
  getInfo as servicesGetInfo,
  getDiff as servicesGetDiff,
  getLog as servicesGetLog,
  getBlame as servicesGetBlame,
  checkoutRepo as servicesCheckoutRepo,
  updateWorkspace as servicesUpdateWorkspace,
  createCommit as servicesCreateCommit,
  addFiles as servicesAddFiles,
  deleteFiles as servicesDeleteFiles,
  revertFiles as servicesRevertFiles,
  resolveConflict as servicesResolveConflict,
  setIgnore as servicesSetIgnore,
  switchBranch as servicesSwitchBranch,
  copyBranchTag as servicesCopyBranchTag,
  mergeBranch as servicesMergeBranch,
  cleanupWorkspace as servicesCleanup,
  exportWorkspace as servicesExport,
  lockFiles as servicesLockFiles,
  unlockFiles as servicesUnlockFiles,
  relocateRepo as servicesRelocate,
  propertyOps as servicesPropertyOps,
  cancelOperation as servicesCancel,
  getLogs as servicesGetLogs,
  exportLogs as servicesExportLogs,
  loadSettings as servicesLoadSettings,
  saveSettings as servicesSaveSettings,
  testConnection as servicesTestConnection,
  saveCredentials as servicesSaveCredentials,
  clearCredentials as servicesClearCredentials,
} from '@/services/svn'

export const useSvnStore = defineStore('svn', () => {
  const progress = ref<OperationProgress | null>(null)
  const isLoading = ref(false)
  const isOperationRunning = ref(false)
  const fileLines = ref<OperationLine[]>([])  // v3 新增：文件行列表
  let initialized = false
  const unlistenFns: Array<() => void> = []

  // ── 认证失败→AuthDialog 联动状态 ──
  interface AuthFailedContext {
    command: string
    args?: Record<string, unknown>
    errorMessage: string
  }
  const authFailed = ref(false)
  const authContext = ref<AuthFailedContext | null>(null)
  const showUpdateRevisionDialog = ref(false)

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
      .catch(err => console.error('[svn store] Failed to register event listeners:', err))
  }

  function destroyEventListeners() {
    unlistenFns.forEach(fn => fn())
    unlistenFns.length = 0
    initialized = false
  }

  /** 统一调用包装：捕获服务端错误、检测认证失败、翻译错误码
   * 仅 stores/svn.ts 内部使用，外部 page/component 通过 store 方法调用 */
  async function call<T>(commandFn: () => Promise<T>, command?: string, args?: Record<string, unknown>): Promise<T> {
    try { return await commandFn() }
    catch (err) {
      // wrappedInvoke 始终抛出原始消息字符串（来自后端 AppError 的 .message）
      const rawMsg = typeof err === 'string' ? err
        : err instanceof Error ? err.message
        : (typeof err === 'object' && err && (err as Record<string, unknown>).message as string) || 'Unknown error'
      // 认证失败（SVN_AUTH_FAILED）→ 保存失败上下文供 AuthDialog 自动弹出
      // E170013 = Unable to connect（连接失败，不一定是认证问题）
      // E175013 = Access forbidden（无访问权限，需重新认证）
      // E215004/E170001 = 无凭据/认证失败（需弹窗让用户输入凭据）
      if (rawMsg.includes('SVN_AUTH_FAILED') || rawMsg.includes('E215004') || rawMsg.includes('E170001')
        || rawMsg.includes('E175013')
        || rawMsg.includes('Authentication failed') || rawMsg.includes('认证失败')
        || rawMsg.includes('authorization failed') || rawMsg.includes('No credentials')) {
        authContext.value = { command: command || '', args: args || {}, errorMessage: rawMsg }
        authFailed.value = true
      }
      // SVN_OPERATION_IN_PROGRESS → 队列冲突
      if (rawMsg.includes('SVN_OPERATION_IN_PROGRESS')) {
        throw new Error('SVN_OPERATION_IN_PROGRESS')
      }
      throw new Error(rawMsg)
    }
  }

  // ── 每个方法调用 services/svn.ts 中的具体业务方法 ──
  async function getStatus(path: string) { return call(() => servicesGetStatus(path), 'get_status', { path }) }
  async function getInfo(path: string) { return call(() => servicesGetInfo(path), 'get_info', { path }) }
  async function getDiff(params: { path: string; revision1?: number; revision2?: number }) { return call(() => servicesGetDiff(params), 'get_diff', params) }
  async function getLog(params: { path: string; limit?: number; revision?: number; search?: string }) {
    const entries = await call(() => servicesGetLog(params), 'get_log', params)
    return { entries, totalCount: entries.length }
  }
  async function getBlame(params: { path: string; revision?: number }) { return call(() => servicesGetBlame(params), 'get_blame', params) }
  async function checkoutRepo(params: { url: string; targetPath: string; depth?: string; ignoreExternals?: boolean; credentials?: { username: string; password: string; saveToCache: boolean } }) {
    return call(async () => {
      const r = await servicesCheckoutRepo({
        url: params.url, targetPath: params.targetPath, depth: params.depth,
        ignoreExternals: params.ignoreExternals, credentials: params.credentials,
      })
      return JSON.stringify(r)
    }, 'checkout_repo', params as unknown as Record<string, unknown>)
  }
  async function updateWorkspace(params: { path: string; revision?: number; depth?: string; ignoreExternals?: boolean }) {
    return call(async () => {
      const r = await servicesUpdateWorkspace({
        path: params.path, revision: params.revision,
        depth: params.depth, ignoreExternals: params.ignoreExternals,
      })
      return JSON.stringify(r)
    }, 'update_workspace', params as unknown as Record<string, unknown>)
  }
  async function commit(params: { paths: string[]; message: string; keepLocks?: boolean }) {
    return call(() => servicesCreateCommit(params), 'create_commit', params as unknown as Record<string, unknown>)
  }
  async function addFiles(paths: string[]) { return call(() => servicesAddFiles(paths), 'add_files', { paths }) }
  async function deleteFiles(params: { paths: string[]; keepLocal?: boolean }) { return call(() => servicesDeleteFiles(params), 'delete_files', params as unknown as Record<string, unknown>) }
  async function revertFiles(paths: string[]) { return call(() => servicesRevertFiles(paths), 'revert_files', { paths }) }
  async function resolveConflict(params: { path: string; resolution: string }) {
    return call(() => servicesResolveConflict({ path: params.path, resolution: params.resolution as 'mine-full' | 'theirs-full' | 'working' }), 'resolve_conflict', params)
  }
  async function setIgnore(params: { path: string; pattern: string }) { return call(() => servicesSetIgnore(params), 'set_ignore', params) }
  async function switchBranch(params: { path: string; targetUrl: string; ignoreAncestry?: boolean; credentials?: SvnCredentials }) {
    return call(() => servicesSwitchBranch(params), 'switch_branch', params as unknown as Record<string, unknown>)
  }
  async function copyBranchTag(params: { srcUrl: string; dstUrl: string; message: string; revision?: number; credentials?: SvnCredentials }) {
    return call(() => servicesCopyBranchTag(params), 'copy_branch_tag', params as unknown as Record<string, unknown>)
  }
  async function mergeBranch(params: { srcUrl: string; revStart?: number; revEnd?: number; targetPath?: string; credentials?: SvnCredentials }) {
    return call(() => servicesMergeBranch(params), 'merge_branch', params as unknown as Record<string, unknown>)
  }
  async function cleanup(path: string) { return call(() => servicesCleanup(path), 'cleanup_workspace', { path }) }
  async function exportWorkspace(params: { path: string; targetDir: string; revision?: number; ignoreExternals?: boolean; credentials?: SvnCredentials }) {
    return call(() => servicesExport(params), 'export_workspace', params as unknown as Record<string, unknown>)
  }
  async function lockFiles(params: { paths: string[]; message?: string }) { return call(() => servicesLockFiles(params), 'lock_files', params) }
  async function unlockFiles(paths: string[]) { return call(() => servicesUnlockFiles(paths), 'unlock_files', { paths }) }
  async function relocate(params: { path: string; fromUrl: string; toUrl: string; credentials?: SvnCredentials }) {
    return call(() => servicesRelocate({ path: params.path, fromUrl: params.fromUrl, toUrl: params.toUrl }), 'relocate_repo', params as unknown as Record<string, unknown>)
  }
  async function propertyOps(params: { path: string; propName?: string; action?: string; propValue?: string }) { return call(() => servicesPropertyOps(params), 'property_ops', params) }
  async function cancelOperation() { return call(() => servicesCancel(), 'cancel_operation') }
  async function getLogs() { return call(() => servicesGetLogs(), 'get_logs') }
  async function exportLogs(targetPath: string) { return call(() => servicesExportLogs(targetPath), 'export_logs', { targetPath }) }
  async function loadSettings() { return call(() => servicesLoadSettings(), 'load_settings') }
  async function saveSettings(settings: AppSettings) { return call(() => servicesSaveSettings(settings), 'save_settings', { settings }) }
  async function testConnection(params: { url: string; username: string; password: string }) { return call(() => servicesTestConnection(params), 'test_connection', params) }
  async function saveCredentials(params: { url: string; username: string; password: string }) { return call(() => servicesSaveCredentials(params), 'save_credentials', params) }
  async function clearCredentials(url: string) { return call(() => servicesClearCredentials(url), 'clear_credentials', { url }) }

  return {
    progress, isLoading, isOperationRunning, fileLines, authFailed, authContext, showUpdateRevisionDialog,
    getStatus, getInfo, getDiff, getLog, getBlame,
    checkoutRepo, updateWorkspace, commit, addFiles, deleteFiles, revertFiles,
    resolveConflict, setIgnore,
    switchBranch, copyBranchTag, mergeBranch,
    cleanup, exportWorkspace, lockFiles, unlockFiles, relocate, propertyOps,
    cancelOperation, getLogs, exportLogs,
    loadSettings, saveSettings,
    testConnection, saveCredentials, clearCredentials,
    retryAuth, cancelAuth, initEventListeners, destroyEventListeners,
  }
})
