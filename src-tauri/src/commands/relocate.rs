use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use super::checkout::is_valid_svn_url;

/// 仓库重定位参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelocateParams {
    pub path: String,
    pub from_url: String,
    pub to_url: String,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 重定位仓库 URL
#[tauri::command]
pub async fn relocate_repo(
    params: RelocateParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    svn::executor::validate_path(&params.path)?;
    // 如果路径是 URL，检测网络可达性
    let svn_schemes = ["svn://", "https://", "svn+ssh://", "http://"];
    if svn_schemes.iter().any(|s| params.path.starts_with(s)) {
        svn::executor::check_network(&params.path).await?;
    }
    // URL scheme 校验（复用 checkout::is_valid_svn_url）
    if !is_valid_svn_url(&params.from_url) || !is_valid_svn_url(&params.to_url) {
        return Err(AppError::InvalidInput("URL 格式无效".into()));
    }
    svn::executor::check_network(&params.to_url).await?;
    state.try_lock()?;
    let r = svn::executor::run_svn(
        &["relocate", &params.from_url, &params.to_url, &params.path],
        &params.path,
        params.credentials.as_ref(),
    ).await;
    state.unlock(); r
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relocate_params_deserialize() {
        let json = r#"{
            "path": "/tmp/repo",
            "from_url": "https://svn.example.com/old-repo",
            "to_url": "https://svn.example.com/new-repo"
        }"#;
        let params: RelocateParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.from_url, "https://svn.example.com/old-repo");
        assert_eq!(params.to_url, "https://svn.example.com/new-repo");
    }

    #[test]
    fn test_relocate_url_validation() {
        // 使用 checkout::is_valid_svn_url（relocate_repo 内部也使用同一函数）
        // from_url 和 to_url 都必须为合法 SVN URL

        // 4 种合法 scheme
        assert!(crate::commands::checkout::is_valid_svn_url("svn://svn.example.com/old-repo"));
        assert!(crate::commands::checkout::is_valid_svn_url("https://svn.example.com/new-repo"));
        assert!(crate::commands::checkout::is_valid_svn_url("svn+ssh://svn.example.com/repo"));
        assert!(crate::commands::checkout::is_valid_svn_url("http://svn.example.com/repo"));

        // 非法 scheme
        assert!(!crate::commands::checkout::is_valid_svn_url("ftp://example.com/repo"));
        assert!(!crate::commands::checkout::is_valid_svn_url("file:///tmp/repo"));
        assert!(!crate::commands::checkout::is_valid_svn_url(""));
    }
}
