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

/** 加载设置（供 settings store 使用） */
export async function loadSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('load_settings')
}

/** 保存设置（供 settings store 使用） */
export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings })
}
