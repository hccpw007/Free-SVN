use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use crate::svn::types::OperationResult;
use tauri::AppHandle;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchParams {
    pub path: String,
    pub target_url: String,
    pub ignore_ancestry: Option<bool>,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyParams {
    pub src_url: String,
    pub dst_url: String,
    pub message: String,
    pub revision: Option<u64>,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeParams {
    pub src_url: String,
    pub rev_start: Option<u64>,
    pub rev_end: Option<u64>,
    pub target_path: String,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 切换分支（长操作可取消）
#[tauri::command]
pub async fn switch_branch(
    app_handle: AppHandle,
    params: SwitchParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<OperationResult, AppError> {
    svn::executor::validate_path(&params.path)?;
    // 如果路径是 URL，检测网络可达性
    let svn_schemes = ["svn://", "https://", "svn+ssh://", "http://"];
    if svn_schemes.iter().any(|s| params.path.starts_with(s)) {
        svn::executor::check_network(&params.path).await?;
    }
    svn::executor::check_network(&params.target_url).await?;
    state.try_lock()?;

    let mut args = vec!["switch".to_string()];
    args.push(params.target_url.clone());
    if params.ignore_ancestry.unwrap_or(false) {
        args.push("--ignore-ancestry".to_string());
    }
    args.push(params.path.clone());

    let result = svn::progress::run_svn_with_progress(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
        app_handle,
        "switch",
        None,
    ).await;

    state.unlock();
    result.map(|_| OperationResult {
        result: "success".to_string(),
        detail: Some("branch switched successfully".to_string()),
    })
}

/// 创建分支/标签（svn copy）
#[tauri::command]
pub async fn copy_branch_tag(
    app_handle: AppHandle,
    params: CopyParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<OperationResult, AppError> {
    svn::executor::check_network(&params.src_url).await?;
    state.try_lock()?;

    let mut args = vec!["copy".to_string()];
    args.push(params.src_url.clone());
    args.push(params.dst_url.clone());
    args.push("-m".to_string()); args.push(params.message.clone());
    if let Some(rev) = params.revision {
        args.push("-r".to_string()); args.push(rev.to_string());
    }

    let result = svn::progress::run_svn_with_progress(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(), ".",
        params.credentials.as_ref(),
        app_handle,
        "copy",
        None,
    ).await;

    state.unlock();
    result.map(|_| OperationResult {
        result: "success".to_string(),
        detail: Some("branch/tag created successfully".to_string()),
    })
}

/// 合并分支（长操作可取消）
#[tauri::command]
pub async fn merge_branch(
    app_handle: AppHandle,
    params: MergeParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<OperationResult, AppError> {
    svn::executor::validate_path(&params.target_path)?;
    svn::executor::check_network(&params.src_url).await?;
    state.try_lock()?;

    let mut args = vec!["merge".to_string()];
    let rev_range = build_merge_rev_range(params.rev_start, params.rev_end);
    if !rev_range.is_empty() { args.push("-r".to_string()); args.push(rev_range); }
    args.push(params.src_url.clone());
    args.push(params.target_path.clone());

    let result = svn::progress::run_svn_with_progress(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.target_path,
        params.credentials.as_ref(),
        app_handle,
        "merge",
        None,
    ).await;

    // 合并完成后读取 status 获取冲突数
    let conflict_count = svn::executor::run_svn(
        &["status", "--xml"], &params.target_path, None,
    ).await.map(|xml| xml.matches("conflicted").count()).unwrap_or(0);

    state.unlock();
    result.map(|_| OperationResult {
        result: if conflict_count > 0 { "conflicts".to_string() } else { "success".to_string() },
        detail: Some(format!("merge completed, {} conflict(s)", conflict_count)),
    })
}

/// 构造 svn merge 的 -r 参数值，根据 rev_start/rev_end 的四种组合返回对应格式
pub(crate) fn build_merge_rev_range(rev_start: Option<u64>, rev_end: Option<u64>) -> String {
    match (rev_start, rev_end) {
        (Some(s), Some(e)) => format!("{}:{}", s, e),
        (Some(s), None) => format!("{}:HEAD", s),
        (None, Some(e)) => format!("0:{}", e),
        (None, None) => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_params_deserialize() {
        let json = r#"{"path": "/tmp/repo", "target_url": "https://svn.example.com/repo/branches/feature"}"#;
        let params: SwitchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.target_url, "https://svn.example.com/repo/branches/feature");
    }

    #[test]
    fn test_copy_params_deserialize() {
        let json = r#"{"src_url": "https://svn.example.com/repo/trunk", "dst_url": "https://svn.example.com/repo/tags/v1.0", "message": "release v1.0"}"#;
        let params: CopyParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.src_url, "https://svn.example.com/repo/trunk");
        assert_eq!(params.dst_url, "https://svn.example.com/repo/tags/v1.0");
        assert_eq!(params.message, "release v1.0");
    }

    #[test]
    fn test_merge_params_deserialize() {
        let json = r#"{"src_url": "https://svn.example.com/repo/branches/feature", "rev_start": 10, "rev_end": 20, "target_path": "/tmp/repo"}"#;
        let params: MergeParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.rev_start, Some(10));
        assert_eq!(params.rev_end, Some(20));
    }

    #[test]
    fn test_build_merge_rev_range_both() {
        // rev_start 和 rev_end 都有值 → "start:end"
        assert_eq!(build_merge_rev_range(Some(10), Some(20)), "10:20");
    }

    #[test]
    fn test_build_merge_rev_range_start_only() {
        // 仅有 rev_start → "start:HEAD"
        assert_eq!(build_merge_rev_range(Some(10), None), "10:HEAD");
    }

    #[test]
    fn test_build_merge_rev_range_end_only() {
        // 仅有 rev_end → "0:end"
        assert_eq!(build_merge_rev_range(None, Some(20)), "0:20");
    }

    #[test]
    fn test_build_merge_rev_range_none() {
        // rev_start 和 rev_end 都无 → 空字符串（不传 -r 参数）
        assert_eq!(build_merge_rev_range(None, None), "");
    }
}
