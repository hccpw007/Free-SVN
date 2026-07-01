use serde::Deserialize;
use crate::models::error::AppError;
use crate::models::file::FileItem;
use crate::models::repo::RepoInfo;
use crate::svn;

#[derive(Debug, Deserialize)]
pub struct StatusParams {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct InfoParams {
    pub path: String,
}

/// 获取文件变更列表（纯本地，不检测网络）
#[tauri::command]
pub async fn get_status(params: StatusParams) -> Result<Vec<FileItem>, AppError> {
    svn::executor::validate_path(&params.path)?;
    let xml = svn::executor::run_svn(&["status", "--xml"], &params.path, None).await?;
    svn::parser::parse_status(&xml)
}

/// 获取工作副本信息
#[tauri::command]
pub async fn get_info(params: InfoParams) -> Result<RepoInfo, AppError> {
    svn::executor::validate_path(&params.path)?;
    let xml = svn::executor::run_svn(&["info", "--xml"], &params.path, None).await?;
    svn::parser::parse_info(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_params_deserialize_valid() {
        let json = r#"{"path": "/tmp/test"}"#;
        let params: StatusParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/test");
    }

    #[test]
    fn test_status_params_deserialize_empty() {
        let json = r#"{"path": ""}"#;
        let params: StatusParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "");
    }

    #[test]
    fn test_info_params_deserialize_valid() {
        let json = r#"{"path": "/tmp/test"}"#;
        let params: InfoParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/test");
    }

    #[test]
    fn test_info_params_deserialize_empty() {
        let json = r#"{"path": ""}"#;
        let params: InfoParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "");
    }
}
