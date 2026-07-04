use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportParams {
    pub path: String,
    pub target_dir: String,
    pub revision: Option<u64>,
    pub ignore_externals: Option<bool>,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 清理工作副本
#[tauri::command]
pub async fn cleanup_workspace(
    path: String,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&path)?;
    state.try_lock()?;
    let r = svn::executor::run_svn(&["cleanup"], &path, None).await;
    state.unlock(); r
}

/// 导出干净副本（长操作可取消）
#[tauri::command]
pub async fn export_workspace(
    app_handle: AppHandle,
    params: ExportParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;
    // 如果路径是 URL，检测网络可达性
    let svn_schemes = ["svn://", "https://", "svn+ssh://", "http://"];
    if svn_schemes.iter().any(|s| params.path.starts_with(s)) {
        svn::executor::check_network(&params.path).await?;
    }
    if params.target_dir.is_empty() {
        return Err(AppError::InvalidInput("目标路径不能为空".into()));
    }
    state.try_lock()?;

    let mut args = vec!["export".to_string()];
    if let Some(rev) = params.revision { args.push("-r".to_string()); args.push(rev.to_string()); }
    args.push(params.path.clone());
    args.push(params.target_dir.clone());
    if params.ignore_externals.unwrap_or(false) { args.push("--ignore-externals".to_string()); }

    let result = svn::progress::run_svn_with_progress(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
        app_handle,
        "export",
        None,
    ).await;

    state.unlock(); result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_params_deserialize() {
        let json = r#"{"path": "/tmp/repo", "target_dir": "/tmp/export"}"#;
        let params: ExportParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/repo");
        assert_eq!(params.target_dir, "/tmp/export");
    }

    #[test]
    fn test_export_params_with_revision() {
        let json = r#"{"path": "/tmp/repo", "target_dir": "/tmp/export", "revision": 42}"#;
        let params: ExportParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.revision, Some(42));
    }

    #[test]
    fn test_cleanup_path_validation_rejects_empty() {
        // cleanup_workspace 将路径校验委托给 svn::executor::validate_path
        // 验证空路径经过 validate_path 后返回 InvalidInput
        let result = crate::svn::executor::validate_path("");
        assert!(result.is_err());
        match result {
            Err(crate::models::error::AppError::InvalidInput(msg)) => assert!(!msg.is_empty()),
            _ => panic!("expected InvalidInput for empty path"),
        }
    }
}
