import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { FileItem } from '@/types/svn'

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
        modified: ['modified'], added: ['added'],
        conflicted: ['conflicted'], deleted: ['deleted'],
        unversioned: ['unversioned'],
      }
      const ss = statusMap[filterStatus.value]
      if (ss) result = result.filter(f => ss.includes(f.status))
    }
    return result
  })

  const selectedCount = computed(() => selectedPaths.value.size)

  /** 刷新（写操作期间返回缓存） */
  async function refresh(fetchFn: () => Promise<FileItem[]>) {
    if (isOperationRunning.value) return lastCachedFiles.value
    try {
      const newFiles = await fetchFn()
      files.value = newFiles
      lastCachedFiles.value = newFiles
      return newFiles
    } catch {
      return files.value
    }
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

  return {
    files, selectedPaths, searchQuery, filterStatus,
    sortField, sortOrder, isLoading, isOperationRunning, lastCachedFiles,
    filteredFiles, selectedCount,
    refresh, reset, clearSelection, toggleSelect, toggleSelectAll,
    applyFilter, clearFilter,
  }
})
