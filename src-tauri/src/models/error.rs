use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("SVN_EXEC_FAILED: {0}")]
    SvnCommand(String),
    #[error("SVN_PARSE_FAILED: {0}")]
    ParseFailed(String),
    #[error("INVALID_INPUT: {0}")]
    InvalidInput(String),
    #[error("IO_ERROR: {0}")]
    Io(#[from] std::io::Error),
    #[error("REPO_ERROR: {0}")]
    Repo(String),
    #[error("SVN_TIMEOUT: {0}")]
    Timeout(String),
    #[error("SVN_AUTH_FAILED: {0}")]
    SvnAuthFailed(String),
    #[error("SVN_NOT_FOUND")]
    SvnNotFound,
    #[error("SVN_NOT_WORKING_COPY")]
    NotWorkingCopy,
    #[error("SVN_OPERATION_IN_PROGRESS")]
    OperationInProgress,
    #[error("SVN_CANCELLED")]
    Cancelled,
    #[error("NETWORK_UNREACHABLE")]
    NetworkUnreachable,
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        use serde::ser::SerializeStruct;
        let (code, msg) = match self {
            AppError::SvnCommand(m) => ("SVN_EXEC_FAILED", m.clone()),
            AppError::ParseFailed(m) => ("SVN_PARSE_FAILED", m.clone()),
            AppError::InvalidInput(m) => ("INVALID_INPUT", m.clone()),
            AppError::Io(m) => ("IO_ERROR", m.to_string()),
            AppError::Repo(m) => ("REPO_ERROR", m.clone()),
            AppError::Timeout(m) => ("SVN_TIMEOUT", m.clone()),
            AppError::SvnAuthFailed(m) => ("SVN_AUTH_FAILED", m.clone()),
            AppError::SvnNotFound => ("SVN_NOT_FOUND", String::new()),
            AppError::NotWorkingCopy => ("SVN_NOT_WORKING_COPY", String::new()),
            AppError::OperationInProgress => ("SVN_OPERATION_IN_PROGRESS", String::new()),
            AppError::Cancelled => ("SVN_CANCELLED", String::new()),
            AppError::NetworkUnreachable => ("NETWORK_UNREACHABLE", String::new()),
        };
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("error", code)?;
        state.serialize_field("message", &msg)?;
        state.end()
    }
}
