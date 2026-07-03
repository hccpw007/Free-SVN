import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MAX_RECENT_WORKSPACES } from '@/types/workspace'
import { getInfo } from '@/services/svn'

/** 工作空间 Store——当前路径/仓库信息/最近工作副本管理 */

const STORAGE_KEY_PATH = 'free-svn:currentPath'
const STORAGE_KEY_WC = 'free-svn:isWorkingCopy'

export const useWorkspaceStore = defineStore('workspace', () => {
  // 尝试从 sessionStorage 恢复 path 和 isWorkingCopy（页面刷新时保留）
  const savedPath = sessionStorage.getItem(STORAGE_KEY_PATH) || ''
  const savedWC = savedPath ? sessionStorage.getItem(STORAGE_KEY_WC) === 'true' : false

  const currentPath = ref(savedPath)
  const recentWorkspaces = ref<string[]>([])
  const url = ref('')           /* 仓库 URL，通过 getInfo 填充 */
  const sourceUrl = ref('')     /* 来源 URL（getInfo.url），用于 BranchTagDialog */
  const branchName = ref('')
  const currentRevision = ref(0)
  const lastCommitTime = ref('')
  const svnVersion = ref('')
  const isOffline = ref(false)
  const isLoading = ref(false)
  // 当前路径是否为有效 SVN 工作副本（switchWorkspace 时自动检测）
  const isWorkingCopy = ref(savedWC)
  // TopBar 触发检出弹窗的信号（App.vue watch 消费后重置）
  const showCheckoutDialog = ref(false)
  // 检出的预填路径（由 TopBar 非工作副本时设置）
  const checkoutInitialPath = ref('')

  function reset() {
    url.value = ''
    sourceUrl.value = ''
    branchName.value = ''
    currentRevision.value = 0
    lastCommitTime.value = ''
    svnVersion.value = ''
    isOffline.value = false
    isLoading.value = false
    isWorkingCopy.value = false
    showCheckoutDialog.value = false
    sessionStorage.removeItem(STORAGE_KEY_PATH)
    sessionStorage.removeItem(STORAGE_KEY_WC)
  }

  async function switchWorkspace(path: string) {
    // 先异步检测是否为 SVN 工作副本（在修改 currentPath 之前完成检测，
    // 避免 HomePage watch 在检测结果未确定时提前触发 refreshWorkspaceInfo）
    let info: Awaited<ReturnType<typeof getInfo>> | null = null
    if (path) {
      try {
        info = await getInfo(path)
      } catch {
        // 非 SVN 工作副本，info 保持 null
      }
    }

    // 检测完成后统一更新状态
    reset()
    currentPath.value = path
    addRecent(path)

    if (info) {
      lastCommitTime.value = info.lastChangedDate ?? ''
      currentRevision.value = info.revision ?? 0
      url.value = info.url ?? ''
      sourceUrl.value = info.url ?? ''
      branchName.value = info.branchName ?? ''
      isWorkingCopy.value = true
    }
    // else: isWorkingCopy 保持 reset 后的 false

    // 持久化到 sessionStorage（页面刷新时保留状态）
    if (path) {
      sessionStorage.setItem(STORAGE_KEY_PATH, path)
      sessionStorage.setItem(STORAGE_KEY_WC, String(isWorkingCopy.value))
    }
  }

  function addRecent(path: string) {
    const idx = recentWorkspaces.value.indexOf(path)
    if (idx >= 0) recentWorkspaces.value.splice(idx, 1)
    recentWorkspaces.value.unshift(path)
    if (recentWorkspaces.value.length > MAX_RECENT_WORKSPACES) {
      recentWorkspaces.value.pop()
    }
  }

  function removeRecent(path: string) {
    const idx = recentWorkspaces.value.indexOf(path)
    if (idx >= 0) recentWorkspaces.value.splice(idx, 1)
  }

  return {
    currentPath, recentWorkspaces, url, sourceUrl, branchName, currentRevision,
    lastCommitTime, svnVersion, isOffline, isLoading,
    isWorkingCopy, showCheckoutDialog, checkoutInitialPath,
    reset, switchWorkspace, addRecent, removeRecent,
  }
})
