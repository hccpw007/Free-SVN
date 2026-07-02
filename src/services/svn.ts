import { invoke } from '@tauri-apps/api/core'
import type { FileItem, RepoInfo, DiffResult, LogEntry, BlameLine, AppSettings } from '@/types/svn'

export async function fetchStatus(path: string): Promise<FileItem[]> {
  return invoke('get_status', { path })
}
export async function fetchInfo(path: string): Promise<RepoInfo> {
  return invoke('get_info', { path })
}
export async function fetchDiff(path: string, r1?: number, r2?: number): Promise<DiffResult> {
  return invoke('get_diff', { path, revision1: r1, revision2: r2 })
}
export async function fetchLog(path: string, limit?: number, rev?: number, search?: string): Promise<LogEntry[]> {
  return invoke('get_log', { path, limit, revision: rev, search })
}
export async function fetchBlame(path: string, rev?: number): Promise<BlameLine[]> {
  return invoke('get_blame', { path, revision: rev })
}
export async function cancelOperation(): Promise<string> {
  return invoke('cancel_operation')
}

/** 添加文件到版本控制 */
export async function addFiles(paths: string[]): Promise<void> {
  return invoke('add_files', { paths })
}

/** 从版本控制删除文件 */
export async function deleteFiles(params: { paths: string[]; keepLocal?: boolean }): Promise<void> {
  return invoke('delete_files', params)
}

/** 还原文件修改 */
export async function revertFiles(paths: string[]): Promise<void> {
  return invoke('revert_files', { paths })
}

/** 标记冲突已解决 */
export async function resolveConflict(params: { path: string; resolution: string }): Promise<void> {
  return invoke('resolve_conflict', params)
}

/** 设置 svn:ignore 忽略模式 */
export async function setIgnore(params: { path: string; pattern: string }): Promise<void> {
  return invoke('set_ignore', params)
}

/** 锁定文件 */
export async function lockFiles(params: { paths: string[]; message?: string }): Promise<void> {
  return invoke('lock_files', params)
}

/** 解锁文件 */
export async function unlockFiles(paths: string[]): Promise<void> {
  return invoke('unlock_files', { paths })
}

/** 测试 SVN 连接（验证凭据） */
export async function testConnection(url: string, username: string, password: string): Promise<string> {
  return invoke('test_connection', {
    url,
    credentials: { username, password, saveToCache: false },
  })
}

/** 清除缓存的凭据 */
export async function clearCredentials(url: string): Promise<string> {
  return invoke('clear_credentials', { url })
}

/** 加载设置（供 settings store 使用） */
export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings')
}

/** 保存设置（供 settings store 使用） */
export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings })
}
