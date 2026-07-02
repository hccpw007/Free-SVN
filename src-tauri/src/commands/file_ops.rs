use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use std::path::Path;

/// 从文件路径提取父目录作为 cwd，文件位于根目录时返回自身
fn get_cwd(path: &str) -> String {
    Path::new(path).parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileOpsParams {
    pub paths: Vec<String>,
    pub keep_local: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveParams {
    pub path: String,
    pub resolution: String,
}

#[tauri::command]
pub async fn add_files(
    params: FileOpsParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.paths.is_empty() { return Err(AppError::InvalidInput("至少指定一个文件".into())); }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;
    let mut args = vec!["add".to_string()];
    args.extend(params.paths.clone());
    let r = svn::executor::run_svn(&args.iter().map(String::as_str).collect::<Vec<&str>>(), &get_cwd(&params.paths[0]), None).await;
    state.unlock(); r
}

#[tauri::command]
pub async fn delete_files(
    params: FileOpsParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.paths.is_empty() { return Err(AppError::InvalidInput("至少指定一个文件".into())); }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;
    let mut args = vec!["delete".to_string()];
    if params.keep_local.unwrap_or(false) { args.push("--keep-local".to_string()); }
    args.extend(params.paths.clone());
    let r = svn::executor::run_svn(&args.iter().map(String::as_str).collect::<Vec<&str>>(), &get_cwd(&params.paths[0]), None).await;
    state.unlock(); r
}

#[tauri::command]
pub async fn revert_files(
    params: FileOpsParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.paths.is_empty() { return Err(AppError::InvalidInput("至少指定一个文件".into())); }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;
    let mut args = vec!["revert".to_string()];
    args.extend(params.paths.clone());
    let r = svn::executor::run_svn(&args.iter().map(String::as_str).collect::<Vec<&str>>(), &get_cwd(&params.paths[0]), None).await;
    state.unlock(); r
}

#[tauri::command]
pub async fn resolve_conflict(
    params: ResolveParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;
    // resolution 参数校验：仅允许 mine-full/theirs-full/working 三种取值
    // mine-full   — 使用本地文件全量覆盖冲突
    // theirs-full — 使用远端文件全量覆盖冲突
    // working     — 标记已手动编辑完成冲突解决
    match params.resolution.as_str() {
        "mine-full" | "theirs-full" | "working" => {}
        _ => return Err(AppError::InvalidInput("resolution 须为 mine-full/theirs-full/working".into())),
    }
    state.try_lock()?;
    let r = svn::executor::run_svn(
        &["resolve", "--accept", &params.resolution, "--xml", &params.path],
        &params.path,
        None,
    ).await;
    state.unlock(); r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_ops_params_deserialize() {
        let json = r#"{"paths": ["/tmp/file1.txt", "/tmp/file2.txt"]}"#;
        let params: FileOpsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.paths.len(), 2);
        assert_eq!(params.paths[0], "/tmp/file1.txt");
    }

    #[test]
    fn test_file_ops_params_keep_local() {
        let json = r#"{"paths": ["/tmp/file.txt"], "keep_local": true}"#;
        let params: FileOpsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.keep_local, Some(true));
    }

    #[test]
    fn test_resolve_params_deserialize_valid() {
        let json = r#"{"path": "/tmp/file.txt", "resolution": "mine-full"}"#;
        let params: ResolveParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.resolution, "mine-full");
    }
}
