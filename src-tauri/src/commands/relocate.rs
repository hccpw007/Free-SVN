use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;

#[derive(Debug, Deserialize)]
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
    // URL scheme 校验
    if !svn_schemes.iter().any(|s| params.from_url.starts_with(s)) ||
       !svn_schemes.iter().any(|s| params.to_url.starts_with(s)) {
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
}
