export interface WorkspaceInfo {
  currentPath: string
  recentWorkspaces: string[]
  branchName: string
  currentRevision: number
  lastCommitTime: string
  svnVersion: string
  isOffline: boolean
  isLoading: boolean
}

export const MAX_RECENT_WORKSPACES = 20
