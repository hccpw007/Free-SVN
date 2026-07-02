import { defineStore } from 'pinia'
import { ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getErrorMessage } from '@/types/error-codes'
import type {
  FileItem, RepoInfo, DiffResult, OperationProgress,
  OperationResult, BlameLine, LogEntry, SvnCredentials,
} from '@/types/svn'
import type { AppSettings } from '@/types/settings'
import { useFileListStore } from './fileList'
// ── 统一从 services/svn.ts 导入具体业务方法（唯一 invoke 入口） ──
import {
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
  wrappedInvoke,
} from '@/services/svn'

export const useSvnStore = defineStore('svn', () => {
  const progress = ref<OperationProgress | null>(null)
  const isLoading = ref(false)
  const isOperationRunning = ref(false)
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
      // 使用 services 的统一 invoke 包装（wrappedInvoke）重试
      const retryArgs = { ...(ctx.args || {}), credentials: { username, password, saveToCache } }
      await wrappedInvoke(ctx.command, retryArgs)
      authFailed.value = false
      authContext.value = null
      return true
    } catch { return false }
  }

  function cancelAuth() {
    authFailed.value = false
    authContext.value = null
  }

  const AUTH_CANCELLED_MSG = 'operation cancelled'
  const AUTH_RETRY_FAILED_MSG = 'auth retry failed'

  function initEventListeners() {
    if (initialized) return
    initialized = true
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

  function destroyEventListeners() {
    unlistenFns.forEach(fn => fn())
    unlistenFns.length = 0
    initialized = false
  }

  /** 统一调用包装：捕获服务端错误、检测认证失败、翻译错误码
   * 仅 stores/svn.ts 内部使用，外部 page/component 通过 store 方法调用 */
  async function call<T>(commandFn: () => Promise<T>): Promise<T> {
    try { return await commandFn() }
    catch (err) {
      const errorCode = typeof err === 'string' ? err : ''
      const msg = errorCode || (typeof err === 'object' ? getErrorMessage(err) : 'Unknown error')
      // SVN_AUTH_FAILED → 保存失败上下文供 AuthDialog 自动弹出
      if (errorCode === 'error.authenticationFailed') {
        authContext.value = { command: '', args: {}, errorMessage: msg }
        authFailed.value = true
      }
      // SVN_OPERATION_IN_PROGRESS → 队列冲突
      if (errorCode === 'error.operationInProgress') {
        throw new Error('SVN_OPERATION_IN_PROGRESS')
      }
      throw new Error(msg)
    }
  }

  // ── 每个方法调用 services/svn.ts 中的具体业务方法 ──
  async function getStatus(path: string) { return call(() => servicesGetStatus(path)) }
  async function getInfo(path: string) { return call(() => servicesGetInfo(path)) }
  async function getDiff(params: { path: string; revision1?: number; revision2?: number }) { return call(() => servicesGetDiff(params)) }
  async function getLog(params: { path: string; limit?: number; revision?: number; search?: string }) {
    const entries = await call(() => servicesGetLog(params))
    return { entries, totalCount: entries.length }
  }
  async function getBlame(params: { path: string; revision?: number }) { return call(() => servicesGetBlame(params)) }
  async function checkoutRepo(params: { url: string; targetPath: string; depth?: string; ignoreExternals?: boolean; credentials?: { username: string; password: string; saveToCache: boolean } }) {
    return call(async () => {
      const r = await servicesCheckoutRepo({
        url: params.url, targetPath: params.targetPath, depth: params.depth,
        ignoreExternals: params.ignoreExternals, credentials: params.credentials,
      })
      return JSON.stringify(r)
    })
  }
  async function updateWorkspace(params: { path: string; revision?: number; depth?: string; ignoreExternals?: boolean }) {
    return call(async () => {
      const r = await servicesUpdateWorkspace({
        path: params.path, revision: params.revision,
        depth: params.depth, ignoreExternals: params.ignoreExternals,
      })
      return JSON.stringify(r)
    })
  }
  async function commit(params: { paths: string[]; message: string; keepLocks?: boolean }) {
    return call(async () => {
      const rev = await servicesCreateCommit(params)
      return { result: 'success', detail: `Committed revision ${rev}` } as OperationResult
    })
  }
  async function addFiles(paths: string[]) { return call(() => servicesAddFiles(paths)) }
  async function deleteFiles(params: { paths: string[]; keepLocal?: boolean }) { return call(() => servicesDeleteFiles(params)) }
  async function revertFiles(paths: string[]) { return call(() => servicesRevertFiles(paths)) }
  async function resolveConflict(params: { path: string; resolution: string }) {
    return call(() => servicesResolveConflict({ path: params.path, resolution: params.resolution as 'mine-full' | 'theirs-full' | 'working' }))
  }
  async function setIgnore(params: { path: string; pattern: string }) { return call(() => servicesSetIgnore(params)) }
  async function switchBranch(params: { path: string; targetUrl: string; ignoreAncestry?: boolean; credentials?: SvnCredentials }) {
    return call(() => servicesSwitchBranch(params))
  }
  async function copyBranchTag(params: { srcUrl: string; dstUrl: string; message: string; revision?: number; credentials?: SvnCredentials }) {
    return call(() => servicesCopyBranchTag(params))
  }
  async function mergeBranch(params: { srcUrl: string; revStart?: number; revEnd?: number; targetPath?: string; credentials?: SvnCredentials }) {
    return call(() => servicesMergeBranch(params))
  }
  async function cleanup(path: string) { return call(() => servicesCleanup(path)) }
  async function exportWorkspace(params: { path: string; targetDir: string; revision?: number; ignoreExternals?: boolean; credentials?: SvnCredentials }) {
    return call(() => servicesExport(params))
  }
  async function lockFiles(params: { paths: string[]; message?: string }) { return call(() => servicesLockFiles(params)) }
  async function unlockFiles(paths: string[]) { return call(() => servicesUnlockFiles(paths)) }
  async function relocate(params: { path: string; fromUrl: string; toUrl: string; credentials?: SvnCredentials }) {
    return call(() => servicesRelocate({ path: params.path, fromUrl: params.fromUrl, toUrl: params.toUrl }))
  }
  async function propertyOps(params: { path: string; propName?: string; action?: string; propValue?: string }) { return call(() => servicesPropertyOps(params)) }
  async function cancelOperation() { return call(() => servicesCancel()) }
  async function getLogs() { return call(() => servicesGetLogs()) }
  async function exportLogs(targetPath: string) { return call(() => servicesExportLogs(targetPath)) }
  async function loadSettings() { return call(() => servicesLoadSettings()) }
  async function saveSettings(settings: AppSettings) { return call(() => servicesSaveSettings(settings)) }
  async function testConnection(params: { url: string; username: string; password: string }) { return call(() => servicesTestConnection(params)) }
  async function saveCredentials(params: { url: string; username: string; password: string }) { return call(() => servicesSaveCredentials(params)) }
  async function clearCredentials(url: string) { return call(() => servicesClearCredentials(url)) }

  return {
    progress, isLoading, isOperationRunning, authFailed, authContext, showUpdateRevisionDialog,
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
