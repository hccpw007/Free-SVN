// ── 值导入（运行时依赖） ──
import { invoke } from '@tauri-apps/api/core'
// ── 类型导入（编译时类型，与值导入分离避免 lint 警告） ──
import type {
  FileItem, RepoInfo, DiffResult, LogEntry, BlameLine,
  MergeResult, OperationResult, AppSettings, SvnCredentials,
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
