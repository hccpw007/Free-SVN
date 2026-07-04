use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;
use tauri::Emitter;
use std::io::Write;
use std::process::{Command, Stdio};

/// 检出操作参数
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

/// 执行 svn list --recursive 获取仓库的文件列表（过滤目录，仅返回文件路径）。
fn get_file_list(
    url: &str,
    credentials: Option<&crate::svn::types::SvnCredentials>,
) -> Result<Vec<String>, AppError> {
    let svn_path = svn::executor::get_svn_path();
    let svn_path_str = svn_path.to_string_lossy().to_string();

    // 构造参数
    let mut all_args: Vec<String> = Vec::new();
    all_args.extend(svn::executor::BASE_SVN_ARGS.iter().map(|s| s.to_string()));
    all_args.push("list".to_string());
    all_args.push("--recursive".to_string());
    if let Some(creds) = credentials {
        all_args.push("--username".to_string());
        all_args.push(creds.username.clone());
        all_args.push("--password-from-stdin".to_string());
    }
    all_args.push(url.to_string());

    let mut cmd = Command::new(&svn_path_str);
    cmd.args(&all_args)
        .envs(svn::executor::get_svn_env())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // 有凭据：通过 stdin 传递密码
    if credentials.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd.spawn().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::SvnNotFound
        } else {
            AppError::Io(e)
        }
    })?;

    if let Some(creds) = credentials {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(creds.password.as_bytes());
            let _ = stdin.flush();
        }
    }

    let output = child.wait_with_output().map_err(AppError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::SvnCommand(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = stdout
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.ends_with('/')
        })
        .map(|line| line.to_string())
        .collect();

    Ok(files)
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

    // ── 阶段 1：枚举文件列表 ──
    // 在真正检出前，先 svn list --recursive 获取所有文件并展示在弹窗中
    app_handle.emit("operation:started", serde_json::json!({
        "operation": "checkout"
    })).ok();

    // 获取文件列表（忽略 svn:externals 等无法 list 的场景）
    let file_list = match get_file_list(&params.url, params.credentials.as_ref()) {
        Ok(list) => list,
        Err(e) => {
            log::warn!("get_file_list 失败，回退到无预先枚举模式: {}", e);
            Vec::new()
        }
    };
    let total_count = file_list.len() as u32;

    // 发送所有 pending 文件行
    for file_path in &file_list {
        app_handle.emit("operation:line", serde_json::json!({
            "operation": "checkout",
            "filePath": file_path,
            "status": "pending"
        })).ok();
    }

    // 发送初始进度（0%，pendingCount = 全部文件数）
    app_handle.emit("operation:progress", serde_json::json!({
        "operation": "checkout",
        "percent": 0,
        "stage": "processing",
        "fileCount": total_count,
        "completedCount": 0,
        "pendingCount": total_count,
        "speed": null,
        "elapsed": null,
        "currentLines": []
    })).ok();

    // ── 阶段 2：实际检出 ──
    // 构造 SVN args
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

    let result = svn::progress::run_svn_with_progress(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        ".",
        params.credentials.as_ref(),
        app_handle,
        "checkout",
        Some(&params.target_path),
    ).await;

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
