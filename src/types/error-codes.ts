/** 错误码枚举。与 Rust AppError 完全同步。 */
enum ErrorCode {
  SVN_EXEC_FAILED = 'SVN_EXEC_FAILED',
  SVN_PARSE_FAILED = 'SVN_PARSE_FAILED',
  INVALID_INPUT = 'INVALID_INPUT',
  IO_ERROR = 'IO_ERROR',
  REPO_ERROR = 'REPO_ERROR',
  SVN_TIMEOUT = 'SVN_TIMEOUT',
  SVN_AUTH_FAILED = 'SVN_AUTH_FAILED',
  SVN_NOT_FOUND = 'SVN_NOT_FOUND',
  SVN_NOT_WORKING_COPY = 'SVN_NOT_WORKING_COPY',
  SVN_OPERATION_IN_PROGRESS = 'SVN_OPERATION_IN_PROGRESS',
  SVN_CANCELLED = 'SVN_CANCELLED',
  NETWORK_UNREACHABLE = 'NETWORK_UNREACHABLE',
  TOOL_NOT_FOUND = 'TOOL_NOT_FOUND',
}

/** ErrorCode → i18n key 映射（供 vue-i18n $t() / t() 使用） */
const ERROR_MESSAGE_KEYS: Record<ErrorCode, string> = {
  [ErrorCode.SVN_EXEC_FAILED]: 'error.SVN_EXEC_FAILED',
  [ErrorCode.SVN_PARSE_FAILED]: 'error.SVN_PARSE_FAILED',
  [ErrorCode.INVALID_INPUT]: 'error.INVALID_INPUT',
  [ErrorCode.IO_ERROR]: 'error.IO_ERROR',
  [ErrorCode.REPO_ERROR]: 'error.REPO_ERROR',
  [ErrorCode.SVN_TIMEOUT]: 'error.SVN_TIMEOUT',
  [ErrorCode.SVN_AUTH_FAILED]: 'error.SVN_AUTH_FAILED',
  [ErrorCode.SVN_NOT_FOUND]: 'error.SVN_NOT_FOUND',
  [ErrorCode.SVN_NOT_WORKING_COPY]: 'error.SVN_NOT_WORKING_COPY',
  [ErrorCode.SVN_OPERATION_IN_PROGRESS]: 'error.SVN_OPERATION_IN_PROGRESS',
  [ErrorCode.SVN_CANCELLED]: 'error.SVN_CANCELLED',
  [ErrorCode.NETWORK_UNREACHABLE]: 'error.NETWORK_UNREACHABLE',
  [ErrorCode.TOOL_NOT_FOUND]: 'error.TOOL_NOT_FOUND',
}

/** 回退中文翻译（i18n 不可用时使用，与 locale/zh-CN.json 同步） */
const FALLBACK_MESSAGES_ZH: Record<string, string> = {
  'error.SVN_EXEC_FAILED': 'SVN 命令执行失败',
  'error.SVN_PARSE_FAILED': 'SVN 输出解析错误',
  'error.INVALID_INPUT': '输入参数无效',
  'error.IO_ERROR': '文件系统错误',
  'error.REPO_ERROR': '仓库操作异常',
  'error.SVN_TIMEOUT': '操作超时，请重试',
  'error.SVN_AUTH_FAILED': '认证失败，请检查凭据配置或使用 SSH key / ~/.subversion/auth/ 缓存',
  'error.SVN_NOT_FOUND': 'svn 未找到，请重新安装 Free-SVN',
  'error.SVN_NOT_WORKING_COPY': '不是 SVN 工作副本',
  'error.SVN_OPERATION_IN_PROGRESS': '操作进行中，请稍候',
  'error.SVN_CANCELLED': '操作已取消',
  'error.NETWORK_UNREACHABLE': '网络不可用，请检查网络连接',
  'error.TOOL_NOT_FOUND': '外部工具未找到，请检查设置',
}

/** 回退英文翻译（在 i18n 和中文回退均不可用时使用） */
const FALLBACK_MESSAGES_EN: Record<string, string> = {
  'error.SVN_EXEC_FAILED': 'SVN command execution failed',
  'error.SVN_PARSE_FAILED': 'SVN output parse error',
  'error.INVALID_INPUT': 'Invalid input',
  'error.IO_ERROR': 'File system error',
  'error.REPO_ERROR': 'Repository operation error',
  'error.SVN_TIMEOUT': 'Operation timed out, please retry',
  'error.SVN_AUTH_FAILED': 'Authentication failed. Please configure SSH key or ~/.subversion/auth/ credentials',
  'error.SVN_NOT_FOUND': 'svn not found, please reinstall Free-SVN',
  'error.SVN_NOT_WORKING_COPY': 'Not a SVN working copy',
  'error.SVN_OPERATION_IN_PROGRESS': 'Operation in progress, please wait',
  'error.SVN_CANCELLED': 'Operation cancelled',
  'error.NETWORK_UNREACHABLE': 'Network unreachable, please check your connection',
  'error.TOOL_NOT_FOUND': 'External tool not found, please check settings',
}

/** 从后端错误响应提取消息（通过 i18n key 映射获取，回退链：中文 → 英文 → 原始消息 → Unknown） */
export function getErrorMessage(err: unknown): string {
  if (err && typeof err === 'object') {
    const e = err as Record<string, unknown>
    if (typeof e.error === 'string') {
      const key = ERROR_MESSAGE_KEYS[e.error as ErrorCode]
      return (key && FALLBACK_MESSAGES_ZH[key])
        || (key && FALLBACK_MESSAGES_EN[key])
        || (e.message as string)
        || 'Unknown error'
    }
  }
  return typeof err === 'string' ? err : 'Unknown error'
}
