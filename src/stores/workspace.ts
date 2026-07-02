import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MAX_RECENT_WORKSPACES } from '@/types/workspace'
import { getInfo } from '@/services/svn'

export const useWorkspaceStore = defineStore('workspace', () => {
  const currentPath = ref('')
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
  const isWorkingCopy = ref(false)
  // TopBar 触发检出弹窗的信号（HomePage/WelcomePage watch 消费后重置）
  const showCheckoutDialog = ref(false)

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
  }

  async function switchWorkspace(path: string) {
    reset()
    currentPath.value = path
    addRecent(path)

    // 当路径非空时，自动检测是否为 SVN 工作副本并填充仓库信息
    if (path) {
      try {
        const info = await getInfo(path)
        lastCommitTime.value = info.lastChangedDate ?? ''
        currentRevision.value = info.revision ?? 0
        url.value = info.url ?? ''
        sourceUrl.value = info.url ?? ''
        branchName.value = info.branchName ?? ''
        isWorkingCopy.value = true
      } catch {
        // getInfo 失败 → 不是有效的 SVN 工作副本（reset 已将 isWorkingCopy 设为 false）
      }
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
    isWorkingCopy, showCheckoutDialog,
    reset, switchWorkspace, addRecent, removeRecent,
  }
})
