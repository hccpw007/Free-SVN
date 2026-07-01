use serde::Serialize;
use thiserror::Error;

/// 应用级错误枚举，共 13 个变体
#[derive(Error, Debug)]
pub enum AppError {
    /// SVN CLI 执行返回非零退出码
    #[error("SVN_EXEC_FAILED: {0}")]
    SvnCommand(String),

    /// SVN XML 输出解析失败
    #[error("SVN_PARSE_FAILED: {0}")]
    ParseFailed(String),

    /// 参数校验不通过
    #[error("INVALID_INPUT: {0}")]
    InvalidInput(String),

    /// 文件系统 I/O 错误（自动从 std::io::Error 转换）
    #[error("IO_ERROR: {0}")]
    Io(#[from] std::io::Error),

    /// 仓库操作异常
    #[error("REPO_ERROR: {0}")]
    Repo(String),

    /// SVN 命令执行超时（默认 60s）
    #[error("SVN_TIMEOUT: {0}")]
    Timeout(String),

    /// SVN 认证提示/失败——超时后 stderr 含 auth 关键词时返回此变体
    #[error("SVN_AUTH_FAILED: {0}")]
    SvnAuthFailed(String),

    /// 内置 svn 不可执行/未找到
    #[error("SVN_NOT_FOUND")]
    SvnNotFound,

    /// 路径不是 SVN 工作副本
    #[error("SVN_NOT_WORKING_COPY")]
    NotWorkingCopy,

    /// 写操作进行中，新操作被拒绝
    #[error("SVN_OPERATION_IN_PROGRESS")]
    OperationInProgress,

    /// 用户取消操作
    #[error("SVN_CANCELLED")]
    Cancelled,

    /// 网络不可达（服务端操作前检测）
    #[error("NETWORK_UNREACHABLE")]
    NetworkUnreachable,

    /// 外部工具未找到/不可执行
    #[error("TOOL_NOT_FOUND: {0}")]
    ToolNotFound(String),
}

/// 自定义 Serialize：将 AppError 序列化为 { error: "ERROR_CODE", message: "..." }
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let (code, msg): (&str, String) = match self {
            AppError::SvnCommand(m) => ("SVN_EXEC_FAILED", m.clone()),
            AppError::ParseFailed(m) => ("SVN_PARSE_FAILED", m.clone()),
            AppError::InvalidInput(m) => ("INVALID_INPUT", m.clone()),
            // std::io::Error 通过 Display 格式化保留完整错误链信息（含 source()）
            AppError::Io(m) => ("IO_ERROR", m.to_string()),
            AppError::Repo(m) => ("REPO_ERROR", m.clone()),
            AppError::Timeout(m) => ("SVN_TIMEOUT", m.clone()),
            AppError::SvnAuthFailed(m) => ("SVN_AUTH_FAILED", m.clone()),
            AppError::SvnNotFound => ("SVN_NOT_FOUND", String::new()),
            AppError::NotWorkingCopy => ("SVN_NOT_WORKING_COPY", String::new()),
            AppError::OperationInProgress => ("SVN_OPERATION_IN_PROGRESS", String::new()),
            AppError::Cancelled => ("SVN_CANCELLED", String::new()),
            AppError::NetworkUnreachable => ("NETWORK_UNREACHABLE", String::new()),
            AppError::ToolNotFound(m) => ("TOOL_NOT_FOUND", m.clone()),
        };
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("error", code)?;
        state.serialize_field("message", &msg)?;
        state.end()
    }
}
