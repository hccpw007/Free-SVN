import { defineStore } from 'pinia'
import { ref } from 'vue'
import { MAX_RECENT_WORKSPACES } from '@/types/workspace'

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

  function reset() {
    currentPath.value = ''
    url.value = ''
    sourceUrl.value = ''
    branchName.value = ''
    currentRevision.value = 0
    lastCommitTime.value = ''
    svnVersion.value = ''
    isOffline.value = false
    isLoading.value = false
  }

  function switchWorkspace(path: string) {
    reset()
    currentPath.value = path
    addRecent(path)
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
    reset, switchWorkspace, addRecent, removeRecent,
  }
})
