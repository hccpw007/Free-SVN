use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;

#[derive(Debug, Deserialize)]
pub struct IgnoreParams {
    pub path: String,
    pub pattern: String,
}

/// 添加 svn:ignore 规则：先读取现有列表，追加去重后写回
#[tauri::command]
pub async fn set_ignore(
    params: IgnoreParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;
    state.try_lock()?;

    // 读取现有 svn:ignore
    let current = svn::executor::run_svn(
        &["propget", "svn:ignore", &params.path],
        &params.path,
        None,
    ).await.unwrap_or_default();

    // 去重后追加
    let mut lines: Vec<&str> = current.lines().collect();
    let pt = params.pattern.trim();
    if !lines.iter().any(|l| l.trim() == pt) {
        lines.push(pt);
    }

    // propset 写回（通过 stdin 传递内容）
    let result = svn::executor::run_svn(
        &["propset", "svn:ignore", &lines.join("\n"), &params.path],
        &params.path,
        None,
    ).await;

    state.unlock();
    result
}

/// 属性操作参数（统一使用 struct Params 模式，与 3.1/3.2 中其他 command 一致）
#[derive(Debug, Deserialize)]
pub struct PropertyOpsParams {
    pub path: String,
    pub prop_name: Option<String>,
    pub action: Option<String>,
    pub prop_value: Option<String>,
}

/// 属性操作（get/set/delete 三种 action）
#[tauri::command]
pub async fn property_ops(
    params: PropertyOpsParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;
    // 如果路径是 URL，检测网络可达性
    let svn_schemes = ["svn://", "https://", "svn+ssh://", "http://"];
    if svn_schemes.iter().any(|s| params.path.starts_with(s)) {
        svn::executor::check_network(&params.path).await?;
    }
    match params.action.as_deref().unwrap_or("get") {
        "get" => {
            let name = params.prop_name.as_deref().unwrap_or("svn:ignore");
            svn::executor::run_svn(&["propget", name, &params.path], &params.path, None).await
        }
        "set" => {
            state.try_lock()?;
            let name = params.prop_name.as_deref().unwrap_or("svn:ignore");
            let value = params.prop_value.as_deref().unwrap_or("");
            let r = svn::executor::run_svn(&["propset", name, value, &params.path], &params.path, None).await;
            state.unlock(); r
        }
        "delete" => {
            state.try_lock()?;
            let name = params.prop_name.as_deref().unwrap_or("svn:ignore");
            let r = svn::executor::run_svn(&["propdel", name, &params.path], &params.path, None).await;
            state.unlock(); r
        }
        _ => Err(AppError::InvalidInput("action 须为 get/set/delete".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ignore_params_deserialize() {
        let json = r#"{"path": "/tmp/repo", "pattern": "*.log"}"#;
        let params: IgnoreParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.pattern, "*.log");
    }

    #[test]
    fn test_property_ops_params_deserialize_get() {
        let json = r#"{"path": "/tmp/repo", "action": "get"}"#;
        let params: PropertyOpsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.action.as_deref(), Some("get"));
    }

    #[test]
    fn test_property_ops_params_deserialize_set() {
        let json = r#"{"path": "/tmp/repo", "action": "set", "prop_name": "svn:ignore", "prop_value": "*.tmp"}"#;
        let params: PropertyOpsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.action.as_deref(), Some("set"));
        assert_eq!(params.prop_value.as_deref(), Some("*.tmp"));
    }
}
