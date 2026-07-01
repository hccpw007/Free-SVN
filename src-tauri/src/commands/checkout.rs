use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;
use tauri::Emitter;

#[derive(Debug, Deserialize)]
pub struct CheckoutParams {
    pub url: String,
    pub target_path: String,
    pub depth: Option<String>,
    pub ignore_externals: Option<bool>,
    /// v5 新增：认证凭据（可选，不传则走 --non-interactive 系统缓存）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 从仓库检出（长操作，可取消，推送进度事件）
#[tauri::command]
pub async fn checkout_repo(
    app_handle: AppHandle,
    params: CheckoutParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    if params.url.is_empty() {
        return Err(AppError::InvalidInput("repository URL cannot be empty".into()));
    }
    if params.target_path.is_empty() {
        return Err(AppError::InvalidInput("target path cannot be empty".into()));
    }
    let valid = ["svn://", "https://", "svn+ssh://", "http://"];
    if !valid.iter().any(|s| params.url.starts_with(s)) {
        return Err(AppError::InvalidInput("URL must start with svn:// https:// or svn+ssh://".into()));
    }

    svn::executor::check_network(&params.url).await?;
    state.try_lock()?;

    app_handle.emit("operation:started", serde_json::json!({
        "operation": "checkout"
    })).ok();

    // BASE_SVN_ARGS: 所有服务端操作统一追加 --non-interactive 和 --trust-server-cert-failures=unknown-ca
    // --non-interactive 在 --trust-server-cert-failures 之前传递，确保 svn CLI 不弹出交互式提示
    // 参数传递时序：--non-interactive 必须在 --trust-server-cert-failures 之前，否则 svn 会忽略证书信任策略
    let mut args = vec![
        "checkout".to_string(),
        "--non-interactive".to_string(),
        "--trust-server-cert-failures=unknown-ca".to_string(),
    ];
    args.push(params.url.clone());
    args.push(params.target_path.clone());
    if let Some(ref depth) = params.depth {
        if !depth.is_empty() {
            args.push("--depth".to_string()); args.push(depth.clone());
        }
    }
    if params.ignore_externals.unwrap_or(false) {
        args.push("--ignore-externals".to_string());
    }

    let result = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        ".",
        params.credentials.as_ref(),
    ).await;

    match &result {
        Ok(output) => {
            let rev = output.lines()
                .find(|l| l.contains("revision"))
                .and_then(|l| l.split_whitespace().last())
                .and_then(|s| s.trim_end_matches('.').parse::<u64>().ok())
                .unwrap_or(0);
            app_handle.emit("operation:completed", serde_json::json!({
                "result": "success", "detail": format!("Checked out revision {}", rev)
            })).ok();
        }
        Err(e) => {
            app_handle.emit("operation:error", serde_json::json!({
                "errorCode": format!("{}", e), "message": format!("{}", e)
            })).ok();
        }
    }

    state.unlock();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkout_params_deserialize() {
        let json = r#"{"url": "https://svn.example.com/repo", "target_path": "/tmp/repo"}"#;
        let params: CheckoutParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://svn.example.com/repo");
        assert_eq!(params.target_path, "/tmp/repo");
    }

    #[test]
    fn test_checkout_params_with_depth() {
        let json = r#"{
            "url": "svn://svn.example.com/repo",
            "target_path": "/tmp/repo",
            "depth": "infinity",
            "ignore_externals": true
        }"#;
        let params: CheckoutParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.depth.as_deref(), Some("infinity"));
        assert_eq!(params.ignore_externals, Some(true));
    }
}
