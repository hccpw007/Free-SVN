use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use crate::svn::parser::{BlameLine, LogEntry};

/// 日志查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogParams {
    pub path: String,
    pub limit: Option<u64>,
    pub revision: Option<u64>,
    pub search: Option<String>,
    /// 认证凭据（可选，用于访问远程仓库）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 注解查询参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlameParams {
    pub path: String,
    pub revision: Option<u64>,
    /// 认证凭据（可选）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 获取版本历史（limit 默认 50，search 仅搜索提交信息）
#[tauri::command]
pub async fn get_log(params: LogParams) -> Result<Vec<LogEntry>, AppError> {
    svn::executor::validate_path(&params.path)?;

    let limit = params.limit.unwrap_or(50);
    let mut args: Vec<String> = vec![
        "log".to_string(),
        "--xml".to_string(),
        "--verbose".to_string(),
        "--limit".to_string(),
        limit.to_string(),
    ];

    if let Some(rev) = params.revision {
        args.push("-r".to_string());
        args.push(format!("{}:1", rev));
    }
    if let Some(ref keyword) = params.search {
        if !keyword.is_empty() {
            args.push("--search".to_string());
            args.push(keyword.clone());
        }
    }
    args.push(params.path.clone());

    let xml = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
    )
    .await?;

    svn::parser::parse_log(&xml)
}

/// 获取文件注解（大文件单独超时 120s）
#[tauri::command]
pub async fn get_blame(params: BlameParams) -> Result<Vec<BlameLine>, AppError> {
    svn::executor::validate_path(&params.path)?;

    let mut args: Vec<String> = vec!["blame".to_string(), "--xml".to_string()];
    if let Some(rev) = params.revision {
        args.push("-r".to_string());
        args.push(rev.to_string());
    }
    args.push(params.path.clone());

    let xml = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
    )
    .await?;

    svn::parser::parse_blame(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_params_with_limit() {
        let json = r#"{"path": "/tmp/repo", "limit": 10}"#;
        let params: LogParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/repo");
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_log_params_no_limit() {
        let json = r#"{"path": "/tmp/repo"}"#;
        let params: LogParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.limit, None);
    }

    #[test]
    fn test_log_params_with_search() {
        let json = r#"{"path": "/tmp/repo", "search": "bugfix"}"#;
        let params: LogParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.search.as_deref(), Some("bugfix"));
    }

    #[test]
    fn test_blame_params_with_revision() {
        let json = r#"{"path": "/tmp/file.rs", "revision": 42}"#;
        let params: BlameParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.revision, Some(42));
    }

    #[test]
    fn test_blame_params_no_revision() {
        let json = r#"{"path": "/tmp/file.rs"}"#;
        let params: BlameParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.revision, None);
    }
}
