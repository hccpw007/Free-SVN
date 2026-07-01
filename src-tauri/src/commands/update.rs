use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;
use tauri::Emitter;

#[derive(Debug, Deserialize)]
pub struct UpdateParams {
    pub path: String,
    pub revision: Option<u64>,
    pub depth: Option<String>,
    pub ignore_externals: Option<bool>,
    /// v5 新增：认证凭据（可选）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 更新工作副本（长操作，可取消，推送进度事件）
#[tauri::command]
pub async fn update_workspace(
    app_handle: AppHandle,
    params: UpdateParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;

    // 先获取远程 URL 检测网络（info 只读操作不需要写锁）
    let info_xml = svn::executor::run_svn(&["info", "--xml"], &params.path, None).await?;
    let info = svn::parser::parse_info(&info_xml)?;
    svn::executor::check_network(&info.url).await?;

    state.try_lock()?;

    app_handle.emit("operation:started", serde_json::json!({
        "operation": "update"
    })).ok();

    // BASE_SVN_ARGS: 追加 --non-interactive（在 --trust-server-cert-failures 之前）和证书信任参数
    // 参数传递时序必须确保 --non-interactive 在前，--trust-server-cert-failures 在后
    let mut args = vec![
        "update".to_string(),
        "--xml".to_string(),
        "--non-interactive".to_string(),
        "--trust-server-cert-failures=unknown-ca".to_string(),
    ];
    if let Some(rev) = params.revision {
        args.push("-r".to_string()); args.push(rev.to_string());
    }
    if let Some(ref depth) = params.depth {
        if !depth.is_empty() {
            args.push("--depth".to_string()); args.push(depth.clone());
        }
    }
    if params.ignore_externals.unwrap_or(false) {
        args.push("--ignore-externals".to_string());
    }
    args.push(params.path.clone());

    let result = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
    ).await;

    match &result {
        Ok(xml) => {
            let rev = extract_update_revision(xml);
            app_handle.emit("operation:completed", serde_json::json!({
                "result": "success", "detail": format!("Updated to revision {}", rev)
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

fn extract_update_revision(xml: &str) -> u64 {
    xml.lines()
        .find(|l| l.contains("revision=\""))
        .and_then(|l| l.split("revision=\"").nth(1))
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_params_deserialize() {
        let json = r#"{"path": "/tmp/repo", "revision": 100}"#;
        let params: UpdateParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/repo");
        assert_eq!(params.revision, Some(100));
    }

    #[test]
    fn test_extract_update_revision_from_xml() {
        let xml = r#"<?xml version="1.0"?>
<update>
<target revision="42" path=".">
</target>
</update>"#;
        assert_eq!(extract_update_revision(xml), 42);
    }

    #[test]
    fn test_extract_update_revision_no_match() {
        assert_eq!(extract_update_revision("<foo></foo>"), 0);
    }
}
