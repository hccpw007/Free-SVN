// ── 值导入（运行时依赖） ──
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
// ── 类型导入（编译时类型，与值导入分离避免 lint 警告） ──
import type {
  FileItem, RepoInfo, DiffResult, LogEntry, BlameLine,
  MergeResult, OperationProgress, OperationResult, AppSettings, SvnCredentials,
} from '@/types/svn'

/** 统一 invoke 包装：自动 try-catch + 错误码翻译 */
export async function wrappedInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(cmd, args) }
  catch (e) {
    // Tauri 2 可能抛出 Error、字符串、或普通对象 { error, message }（后端 AppError 序列化结果）
    let msg: string
    if (typeof e === 'string') {
      msg = e
    } else if (e instanceof Error) {
      msg = e.message
    } else if (e && typeof e === 'object') {
      // Tauri 2 IPC 错误为 { error: "SVN_EXEC_FAILED", message: "svn: ..." }
      const obj = e as Record<string, unknown>
      msg = (typeof obj.message === 'string' ? obj.message : '') || (typeof obj.error === 'string' ? obj.error : '')
    } else {
      msg = '未知错误'
    }
    // translateError 返回 i18n key，用于 call() 中检测认证/操作特定错误
    // 但始终抛出原始消息，让用户看到实际错误详情而非 i18n key
    throw msg
  }
}

/**
 * 对 Rust 中参数名为 `params` 的命令的调用包装。
 * Tauri 2 要求 invoke 参数 key 与 Rust 参数名一致，
 * 因此需要额外嵌套一层 { params: ... }。
 */
function invokeWithParams<T>(cmd: string, args: Record<string, unknown>): Promise<T> {
  return wrappedInvoke<T>(cmd, { params: args })
}

/**
 * 错误码 → i18n key
 *
 * 本函数返回的是 i18n key 字符串，不是用户可见的翻译文本。
 * 调用方（通常是 store 或页面）需要在拿到返回值后通过
 * `t(translatedError)` 转换为最终用户文本。
 * 这样做是为了保持 services 层的纯净（不依赖 vue-i18n 实例），
 * 让调用方自行决定如何处理国际化。
 */

// ErrorCode 枚举定义 — 集中所有错误码，避免字符串散落各处
enum ErrorCode {
  OperationTimeout = 'error.operationTimeout',
  OperationCancelled = 'error.operationCancelled',
  SvnNotFound = 'error.svnNotFound',
  NetworkUnreachable = 'error.networkUnreachable',
  OperationInProgress = 'error.operationInProgress',
  SvnCommandFailed = 'error.svnCommandFailed',
  AuthenticationFailed = 'error.authenticationFailed',
  InvalidInput = 'error.invalidInput',
  IoError = 'error.ioError',
  RepoError = 'error.repoError',
  ParseFailed = 'error.parseFailed',
  WorkingCopyClean = 'error.workingCopyClean',
  PathNotFound = 'error.pathNotFound',
  NotWorkingCopy = 'error.notWorkingCopy',
  UrlNotFound = 'error.urlNotFound',
  WorkingCopyLocked = 'error.workingCopyLocked',
  InvalidRevision = 'error.invalidRevision',
  ConflictDetected = 'error.conflictDetected',
  PathAlreadyVersioned = 'error.pathAlreadyVersioned',
  EntryNotFound = 'error.entryNotFound',
  Unknown = 'error.unknown',
}

/** ErrorCode 对应的英文回退文本（当语言包中未定义该 key 时使用） */
const ErrorFallbackMap: Record<ErrorCode, string> = {
  [ErrorCode.OperationTimeout]: 'Operation timed out',
  [ErrorCode.OperationCancelled]: 'Operation cancelled',
  [ErrorCode.SvnNotFound]: 'SVN not found, please check installation',
  [ErrorCode.NetworkUnreachable]: 'Network unreachable',
  [ErrorCode.OperationInProgress]: 'Another operation is in progress',
  [ErrorCode.SvnCommandFailed]: 'SVN command failed',
  [ErrorCode.AuthenticationFailed]: 'Authentication failed',
  [ErrorCode.InvalidInput]: 'Invalid input',
  [ErrorCode.IoError]: 'File system error',
  [ErrorCode.RepoError]: 'Repository error',
  [ErrorCode.ParseFailed]: 'Failed to parse SVN output',
  [ErrorCode.WorkingCopyClean]: 'Working copy is clean',
  [ErrorCode.PathNotFound]: 'Path not found',
  [ErrorCode.NotWorkingCopy]: 'Not a working copy',
  [ErrorCode.UrlNotFound]: 'URL not found',
  [ErrorCode.WorkingCopyLocked]: 'Working copy is locked',
  [ErrorCode.InvalidRevision]: 'Invalid revision',
  [ErrorCode.ConflictDetected]: 'Conflict detected',
  [ErrorCode.PathAlreadyVersioned]: 'Path is already versioned',
  [ErrorCode.EntryNotFound]: 'Entry not found in working copy',
  [ErrorCode.Unknown]: 'An unknown error occurred',
}

function translateError(msg: string): string {
  // 1. 优先匹配后端 AppError 错误码前缀
  if (msg.startsWith('SVN_TIMEOUT')) return ErrorCode.OperationTimeout
  if (msg.startsWith('SVN_CANCELLED')) return ErrorCode.OperationCancelled
  if (msg.startsWith('SVN_NOT_FOUND')) return ErrorCode.SvnNotFound
  if (msg.startsWith('NETWORK_UNREACHABLE')) return ErrorCode.NetworkUnreachable
  if (msg.startsWith('SVN_OPERATION_IN_PROGRESS')) return ErrorCode.OperationInProgress
  if (msg.startsWith('SVN_EXEC_FAILED')) return ErrorCode.SvnCommandFailed
  if (msg.startsWith('SVN_AUTH_FAILED')) return ErrorCode.AuthenticationFailed
  if (msg.startsWith('INVALID_INPUT')) return ErrorCode.InvalidInput
  if (msg.startsWith('IO_ERROR')) return ErrorCode.IoError
  if (msg.startsWith('REPO_ERROR')) return ErrorCode.RepoError
  if (msg.startsWith('SVN_PARSE_FAILED')) return ErrorCode.ParseFailed

  // 2. 匹配命令执行错误中的 SVN CLI E- 码
  if (msg.includes('E155037')) return ErrorCode.WorkingCopyClean
  if (msg.includes('E155010') || msg.includes('E200009')) return ErrorCode.PathNotFound
  if (msg.includes('E204899')) return ErrorCode.NotWorkingCopy
  if (msg.includes('E170001') || msg.includes('认证')) return ErrorCode.AuthenticationFailed
  if (msg.includes('E160013') || msg.includes('E731001') || msg.includes('404')) return ErrorCode.UrlNotFound
  if (msg.includes('E155035') || msg.includes('locked')) return ErrorCode.WorkingCopyLocked
  if (msg.includes('E205007') || msg.includes('E155024')) return ErrorCode.InvalidRevision
  if (msg.includes('E165001') || msg.includes('E195020')) return ErrorCode.ConflictDetected
  if (msg.includes('E200012') || msg.includes('E200014')) return ErrorCode.PathAlreadyVersioned
  if (msg.includes('E155036')) return ErrorCode.EntryNotFound
  if (msg.includes('E170013')) return ErrorCode.SvnCommandFailed  // 无法连接到仓库
  if (msg.includes('E230001')) return ErrorCode.SvnCommandFailed  // SSL 证书验证失败
  if (msg.includes('timeout') || msg.includes('超时')) return ErrorCode.OperationTimeout
  return ErrorCode.Unknown
}

// ── 只读操作（5个） ──────────────────────────
export async function getStatus(path: string, ignorePatterns?: string): Promise<FileItem[]> {
  return invokeWithParams<FileItem[]>('get_status', { path, ignorePatterns })
}
export async function getInfo(path: string): Promise<RepoInfo> {
  return invokeWithParams<RepoInfo>('get_info', { path })
}
export async function getDiff(params: { path: string; revision1?: number; revision2?: number }): Promise<DiffResult> {
  return invokeWithParams<DiffResult>('get_diff', params)
}
export async function getLog(params: { path: string; limit?: number; revision?: number; search?: string }): Promise<LogEntry[]> {
  return invokeWithParams<LogEntry[]>('get_log', params)
}
export async function getBlame(params: { path: string; revision?: number }): Promise<BlameLine[]> {
  return invokeWithParams<BlameLine[]>('get_blame', params)
}

// ── 写操作（6个） ──────────────────────────
export async function checkoutRepo(params: { url: string; targetPath: string; depth?: string; ignoreExternals?: boolean; credentials?: SvnCredentials }): Promise<{ revision: number }> {
  return invokeWithParams('checkout_repo', params)
}
export async function updateWorkspace(params: { path: string; revision?: number; depth?: string; ignoreExternals?: boolean; credentials?: SvnCredentials }): Promise<{ revision: number; conflicts: number }> {
  return invokeWithParams('update_workspace', params)
}
export async function createCommit(params: { paths: string[]; message: string; keepLocks?: boolean }): Promise<OperationResult> {
  return invokeWithParams<OperationResult>('create_commit', params)
}
export async function addFiles(paths: string[]): Promise<string> {
  return invokeWithParams<string>('add_files', { paths })
}
export async function deleteFiles(params: { paths: string[]; keepLocal?: boolean }): Promise<string> {
  return invokeWithParams<string>('delete_files', params)
}
export async function revertFiles(paths: string[]): Promise<string> {
  return invokeWithParams<string>('revert_files', { paths })
}

// ── 冲突/忽略（2个） ──────────────────────────
export async function resolveConflict(params: { path: string; resolution: 'mine-full' | 'theirs-full' | 'working' }): Promise<string> {
  return invokeWithParams<string>('resolve_conflict', params)
}
export async function setIgnore(params: { path: string; pattern: string }): Promise<string> {
  return invokeWithParams<string>('set_ignore', params)
}

// ── 分支操作（3个） ──────────────────────────
export async function switchBranch(params: { path: string; targetUrl: string; ignoreAncestry?: boolean; credentials?: SvnCredentials }): Promise<OperationResult> {
  return invokeWithParams('switch_branch', params)
}
export async function copyBranchTag(params: { srcUrl: string; dstUrl: string; message: string; revision?: number; credentials?: SvnCredentials }): Promise<OperationResult> {
  return invokeWithParams('copy_branch_tag', params)
}
export async function mergeBranch(params: { srcUrl: string; revStart?: number; revEnd?: number; targetPath: string; credentials?: SvnCredentials }): Promise<MergeResult> {
  return invokeWithParams('merge_branch', params)
}

// ── 系统操作（5个） ──────────────────────────
export async function cleanupWorkspace(path: string): Promise<string> {
  return wrappedInvoke<string>('cleanup_workspace', { path })
}
export async function exportWorkspace(params: { path: string; targetDir: string; revision?: number; ignoreExternals?: boolean; credentials?: SvnCredentials }): Promise<string> {
  return invokeWithParams<string>('export_workspace', params)
}
export async function lockFiles(params: { paths: string[]; message?: string }): Promise<string> {
  return invokeWithParams<string>('lock_files', params)
}
export async function unlockFiles(paths: string[]): Promise<string> {
  return invokeWithParams<string>('unlock_files', { paths })
}
export async function relocateRepo(params: { path: string; fromUrl: string; toUrl: string }): Promise<string> {
  return invokeWithParams<string>('relocate_repo', params)
}

// ── 属性/取消/网络/日志（5个） ──────────────────
export async function propertyOps(params: { path: string; propName?: string; action?: string }): Promise<string> {
  return invokeWithParams<string>('property_ops', params)
}
export async function cancelOperation(): Promise<void> {
  return wrappedInvoke('cancel_operation')
}
export async function checkNetwork(): Promise<boolean> {
  return wrappedInvoke<boolean>('check_network')
}
export async function getLogs(): Promise<string> {
  return wrappedInvoke<string>('get_logs')
}
export async function exportLogs(targetPath: string): Promise<void> {
  return wrappedInvoke('export_logs', { target_path: targetPath })
}

// ── 设置（2个） ──────────────────────────
export async function loadSettings(): Promise<AppSettings> {
  return wrappedInvoke<AppSettings>('load_settings')
}
export async function saveSettings(settings: AppSettings): Promise<void> {
  return wrappedInvoke('save_settings', { settings })
}

// ── 认证（6个） ──────────────────
export async function testConnection(params: { url: string; username: string; password: string }): Promise<string> {
  return invokeWithParams<string>('test_connection', params)
}
export async function saveCredentials(params: { url: string; username: string; password: string }): Promise<void> {
  return invokeWithParams('save_credentials', params)
}
export async function clearCredentials(url: string): Promise<string> {
  return invokeWithParams<string>('clear_credentials', { url })
}
export async function listCachedCredentials(): Promise<import('@/types/svn').CachedCredential[]> {
  return wrappedInvoke<import('@/types/svn').CachedCredential[]>('list_cached_credentials')
}
export async function deleteCachedCredential(id: string): Promise<string> {
  return invokeWithParams<string>('delete_cached_credential', { id })
}
export async function updateCachedCredentialPassword(id: string, newPassword: string): Promise<string> {
  return invokeWithParams<string>('update_cached_credential_password', { id, newPassword })
}

// ── 分支查询（1个，7.4 新增） ──────────
export async function listBranches(url: string): Promise<string[]> {
  return wrappedInvoke<string[]>('list_branches', { url })
}

// ── Event Listeners（3个） ──────────────────
function onOperationProgress(cb: (p: OperationProgress) => void) {
  return listen<OperationProgress>('operation:progress', (e) => cb(e.payload))
}
function onOperationCompleted(cb: (r: OperationResult) => void) {
  return listen<OperationResult>('operation:completed', (e) => cb(e.payload))
}
function onOperationError(cb: (e: { errorCode: string; message: string }) => void) {
  return listen<{ errorCode: string; message: string }>('operation:error', (e) => cb(e.payload))
}
