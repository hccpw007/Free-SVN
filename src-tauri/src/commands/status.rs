use serde::Deserialize;
use crate::models::error::AppError;
use crate::models::file::FileItem;
use crate::models::repo::RepoInfo;
use crate::svn;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusParams {
    pub path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoParams {
    pub path: String,
}

/// 获取文件变更列表（纯本地，不检测网络）
#[tauri::command]
pub async fn get_status(params: StatusParams) -> Result<Vec<FileItem>, AppError> {
    svn::executor::validate_path(&params.path)?;
    let xml = svn::executor::run_svn(&["status", "--xml", "--depth", "infinity"], &params.path, None).await?;
    let mut items = svn::parser::parse_status(&xml)?;
    // svn status --xml 不包含文件大小，通过文件系统 stat 获取
    let base = std::path::Path::new(&params.path);
    for item in items.iter_mut() {
        let full = base.join(&item.path);
        // 目录和已删除的文件无法 stat，保持 size=None
        if let Ok(meta) = std::fs::metadata(&full) {
            if meta.is_file() {
                item.size = Some(meta.len());
            }
        }
    }
    Ok(items)
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
