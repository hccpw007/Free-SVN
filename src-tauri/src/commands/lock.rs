use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;

/// 锁定/解锁操作参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockParams {
    pub paths: Vec<String>,
    pub message: Option<String>,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

#[tauri::command]
pub async fn lock_files(
    params: LockParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.paths.is_empty() { return Err(AppError::InvalidInput("至少指定一个文件".into())); }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;
    let mut args = vec!["lock".to_string()];
    if let Some(ref msg) = params.message { args.push("-m".to_string()); args.push(msg.clone()); }
    args.extend(params.paths.clone());
    let r = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(), &params.paths[0],
        params.credentials.as_ref(),
    ).await;
    state.unlock(); r
}

#[tauri::command]
pub async fn unlock_files(
    params: LockParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.paths.is_empty() { return Err(AppError::InvalidInput("至少指定一个文件".into())); }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;
    let mut args = vec!["unlock".to_string()];
    args.extend(params.paths.clone());
    let r = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(), &params.paths[0],
        params.credentials.as_ref(),
    ).await;
    state.unlock(); r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_params_deserialize() {
        let json = r#"{"paths": ["/tmp/file.txt"], "message": "lock for editing"}"#;
        let params: LockParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.paths.len(), 1);
        assert_eq!(params.message.as_deref(), Some("lock for editing"));
    }

    #[test]
    fn test_unlock_params_deserialize() {
        let json = r#"{"paths": ["/tmp/file.txt"]}"#;
        let params: LockParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.paths.len(), 1);
        assert_eq!(params.message, None);
    }
}
