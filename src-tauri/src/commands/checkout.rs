use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;
use tauri::Emitter;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckoutParams {
    pub url: String,
    pub target_path: String,
    pub depth: Option<String>,
    pub ignore_externals: Option<bool>,
    /// v5 新增：认证凭据（可选，不传则走 --non-interactive 系统缓存）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 校验 URL 是否为合法的 SVN 仓库 URL（svn:// https:// svn+ssh:// http://）
pub(crate) fn is_valid_svn_url(url: &str) -> bool {
    let valid = ["svn://", "https://", "svn+ssh://", "http://"];
    valid.iter().any(|s| url.starts_with(s))
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
    if !is_valid_svn_url(&params.url) {
        return Err(AppError::InvalidInput("URL must start with svn:// https:// or svn+ssh://".into()));
    }

    svn::executor::check_network(&params.url).await?;
    state.try_lock()?;

    app_handle.emit("operation:started", serde_json::json!({
        "operation": "checkout"
    })).ok();

    // BASE_SVN_ARGS: 由 run_svn() 统一追加，包含 --non-interactive
    // 和 --trust-server-cert-failures=unknown-ca,cn-mismatch,expired,...
    // 命令特定的 args 中不要再加 --trust-server-cert-failures（会覆盖 BASE_SVN_ARGS 的完整列表）
    // --non-interactive 在此显式添加以确保始终存在（run_svn 传凭据时会移除 BASE_SVN_ARGS 中的它）
    let mut args = vec![
        "checkout".to_string(),
        "--non-interactive".to_string(),
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

    #[test]
    fn test_checkout_url_validation() {
        // 4 种合法 SVN URL scheme
        assert!(is_valid_svn_url("svn://svn.example.com/repo"));
        assert!(is_valid_svn_url("https://svn.example.com/repo"));
        assert!(is_valid_svn_url("svn+ssh://svn.example.com/repo"));
        assert!(is_valid_svn_url("http://svn.example.com/repo"));
        // 带路径和端口的合法 URL
        assert!(is_valid_svn_url("https://svn.example.com:8443/repo/trunk"));
        assert!(is_valid_svn_url("svn://localhost/repo"));

        // 非法 scheme
        assert!(!is_valid_svn_url("ftp://example.com/repo"));
        assert!(!is_valid_svn_url("file:///tmp/repo"));
        assert!(!is_valid_svn_url("git+ssh://example.com/repo"));
        assert!(!is_valid_svn_url("ssh://example.com/repo"));
        // 空字符串
        assert!(!is_valid_svn_url(""));
        // 相对路径（不含 scheme）
        assert!(!is_valid_svn_url("relative/path"));
        assert!(!is_valid_svn_url("/absolute/path"));
    }
}
