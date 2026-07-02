import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { FileItem } from '@/types/svn'
import { getStatus as fetchStatus, revertFiles as svnRevertFiles, deleteFiles as svnDeleteFiles, setIgnore as svnSetIgnore, unlockFiles as svnUnlockFiles } from '@/services/svn'

export const useFileListStore = defineStore('fileList', () => {
  const files = ref<FileItem[]>([])
  const selectedPaths = ref<Set<string>>(new Set())
  const searchQuery = ref('')
  const filterStatus = ref('all')
  const sortField = ref('status')
  const sortOrder = ref<'asc' | 'desc'>('asc')
  const isLoading = ref(false)
  const isOperationRunning = ref(false)
  const lastCachedFiles = ref<FileItem[]>([])

  const filteredFiles = computed(() => {
    let result = files.value
    if (searchQuery.value) {
      const q = searchQuery.value.toLowerCase()
      result = result.filter(f => f.path.toLowerCase().includes(q))
    }
    if (filterStatus.value !== 'all') {
      const statusMap: Record<string, string[]> = {
        modified: ['modified'],
        pendingAdd: ['added', 'unversioned', 'replaced'],
        conflicted: ['conflicted'],
        pendingDelete: ['deleted', 'missing'],
        ignored: ['ignored', 'external'],
        abnormal: ['obstructed', 'incomplete'],
      }
      const ss = statusMap[filterStatus.value]
      if (ss) result = result.filter(f => ss.includes(f.status))
    }
    return result
  })

  const selectedCount = computed(() => selectedPaths.value.size)

  /** 刷新（写操作期间返回缓存）。若不传 fetchFn，自动使用 fetchStatus + workspace.currentPath */
  async function refresh(fetchFn?: () => Promise<FileItem[]>) {
    if (isOperationRunning.value) return lastCachedFiles.value
    try {
      const newFiles = fetchFn
        ? await fetchFn()
        : await fetchCurrentStatus()
      files.value = newFiles
      lastCachedFiles.value = newFiles
      return newFiles
    } catch (e: unknown) {
      console.error('[fileList store] refresh 失败:', e)
      return files.value
    }
  }

  /** 获取当前工作副本的状态 */
  async function fetchCurrentStatus(): Promise<FileItem[]> {
    const { useWorkspaceStore } = await import('./workspace')
    const ws = useWorkspaceStore()
    if (!ws.currentPath) return []
    return fetchStatus(ws.currentPath)
  }

  function reset() {
    files.value = []
    selectedPaths.value = new Set()
    searchQuery.value = ''
    filterStatus.value = 'all'
    sortField.value = 'status'
    sortOrder.value = 'asc'
    lastCachedFiles.value = []
  }

  function clearSelection() { selectedPaths.value = new Set() }

  function toggleSelect(path: string) {
    const set = new Set(selectedPaths.value)
    set.has(path) ? set.delete(path) : set.add(path)
    selectedPaths.value = set
  }

  function toggleSelectAll(v: boolean) {
    selectedPaths.value = v
      ? new Set(filteredFiles.value.map(f => f.path))
      : new Set()
  }

  /** applyFilter：触发表单搜索/筛选的防抖重新计算（由模板中的 @input/@change 调用） */
  function applyFilter() {
    // filteredFiles 是 computed 响应式属性，searchQuery 和 filterStatus 的变更
    // 已自动触发 filteredFiles 重新计算。此方法作为显式调用入口，便于：
    // 1. 搜索框 @input 事件中调用触发外部副作用
    // 2. 在 watch 或防抖逻辑中作为统一触发点
    // filteredFiles 的实时计算由 computed 保证，无需额外逻辑
  }

  /** clearFilter：重置搜索和筛选条件 */
  function clearFilter() {
    searchQuery.value = ''
    filterStatus.value = 'all'
  }

  /** 还原文件修改 */
  async function revertFile(path: string) {
    try {
      await svnRevertFiles([path])
      await refresh()
    } catch (e: unknown) {
      console.error('[fileList store] revertFile 失败:', e)
    }
  }

  /** 批量还原选中的文件 */
  async function batchRevertFiles() {
    const paths = Array.from(selectedPaths.value)
    if (paths.length === 0) return
    try {
      await svnRevertFiles(paths)
      selectedPaths.value = new Set()
      await refresh()
    } catch (e: unknown) {
      console.error('[fileList store] batchRevertFiles 失败:', e)
    }
  }

  /** 忽略文件 */
  async function ignoreFile(path: string) {
    try {
      const { useWorkspaceStore } = await import('./workspace')
      const ws = useWorkspaceStore()
      await svnSetIgnore({ path: ws.currentPath, pattern: path })
      await refresh()
    } catch (e: unknown) {
      console.error('[fileList store] ignoreFile 失败:', e)
    }
  }

  /** 删除文件 */
  async function deleteFile(path: string) {
    try {
      await svnDeleteFiles({ paths: [path] })
      await refresh()
    } catch (e: unknown) {
      console.error('[fileList store] deleteFile 失败:', e)
    }
  }

  /** 解锁文件 */
  async function unlockFile(path: string) {
    try {
      await svnUnlockFiles([path])
      await refresh()
    } catch (e: unknown) {
      console.error('[fileList store] unlockFile 失败:', e)
    }
  }

  return {
    files, selectedPaths, searchQuery, filterStatus,
    sortField, sortOrder, isLoading, isOperationRunning, lastCachedFiles,
    filteredFiles, selectedCount,
    refresh, reset, clearSelection, toggleSelect, toggleSelectAll,
    applyFilter, clearFilter, revertFile, batchRevertFiles, ignoreFile, deleteFile, unlockFile,
  }
})
