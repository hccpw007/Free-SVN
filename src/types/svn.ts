/** 认证凭据（v5 新增：用于检出/重试/凭据管理） */
export interface SvnCredentials {
  username: string
  password: string
  saveToCache: boolean
}

/** 缓存的凭据条目 */
export interface CachedCredential {
  id: string
  username: string
  realm: string
}

export interface FileItem {
  path: string
  status: string
  wcStatus?: string
  commitRevision?: number
  commitAuthor?: string
  commitDate?: string
  isBinary: boolean
  propertyChanges?: string
  movedFrom?: string
  movedTo?: string
  copiedFrom?: string
  size?: number
  lock?: LockInfo
}

interface LockInfo {
  token: string
  owner: string
  comment?: string
  created: string
  expires?: string
}

export interface RepoInfo {
  path: string
  url: string
  relativeUrl?: string
  root: string
  repositoryUuid: string
  revision: number
  nodeKind: string
  lastChangedRevision: number
  lastChangedAuthor: string
  lastChangedDate: string
  schedule?: string
  depth?: string
  checksum?: string
  lock?: LockInfo
  branchName?: string
}

export interface DiffResult {
  content: string
  isBinary: boolean
  mimeType?: string
  /** 前端展示用修订版本号。
   *  revision1（旧版本）和 revision2（新版本）为前端 diff 展示的参考值。
   *  后端 get_diff 返回的实际 content 已包含对应版本的差异文本。
   *  若后端返回中不包含具体版本号，前端应设为 undefined 或由调用方自行填充。 */
  revision1?: number
  /** 前端展示用修订版本号（同 revision1 说明） */
  revision2?: number
  /** 差异作者信息（如可用） */
  author?: string
}

export interface OperationProgress {
  operation: string
  percent: number
  stage: string
  fileCount: number
  completedCount: number
  pendingCount: number
  speed: string | null
  elapsed: string | null
  currentLines: string[]
}

/** 单行文件操作信息（operation:line 事件 payload） */
export interface OperationLine {
  operation: string
  filePath: string
  status: 'completed' | 'in_progress' | 'pending'
  /** 是否是标记行（如管道中断提示），前端可据此判断是否展示 */
  isMarker?: boolean
}

/** 操作取消信息（operation:cancelled 事件 payload） */
export interface CancelledPayload {
  reason: string
}

export interface OperationResult {
  result: string
  detail?: string
}

export interface BlameLine {
  revision: number
  author: string
  date: string
  lineNumber: number
  content: string
}

export interface LogEntry {
  revision: number
  author?: string
  date?: string
  msg?: string
  paths?: LogPathEntry[]
}

interface LogPathEntry {
  action: string
  path: string
}

interface ConflictInfo {
  path: string
  conflictType: 'text' | 'tree' | 'property'
  mineFile?: string
  baseFile?: string
  theirsFile?: string
}

export interface MergeResult {
  result: 'success' | 'conflicts' | 'error'
  conflictCount: number
  conflicts?: ConflictInfo[]
  detail?: string
}

/** 认证连接测试结果 */
interface ConnectionTestResult {
  success: boolean
  realm?: string
  error?: string
}
