use serde::{Deserialize, Serialize};
use crate::models::error::AppError;
use crate::svn;
use std::fs;
use std::path::PathBuf;

/// 认证凭据参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
    pub save_to_cache: bool,
}

/// 连接测试参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionParams {
    pub url: String,
    pub credentials: AuthCredentials,
}

/// 保存凭据参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCredentialsParams {
    pub url: String,
    pub credentials: AuthCredentials,
}

/// 清除凭据参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearCredentialsParams {
    pub url: String,
}

/// 测试与 SVN 服务器的连接。
/// 使用提供的凭据执行 `svn info --non-interactive --username xxx --password-from-stdin`。
/// 成功返回连接的仓库 realm 信息，失败返回错误。
#[tauri::command]
pub async fn test_connection(params: TestConnectionParams) -> Result<String, AppError> {
    if params.url.is_empty() {
        return Err(AppError::InvalidInput("仓库 URL 不能为空".into()));
    }
    if params.credentials.username.is_empty() || params.credentials.password.is_empty() {
        return Err(AppError::InvalidInput("用户名和密码不能为空".into()));
    }

    // 先检测网络
    svn::executor::check_network(&params.url).await?;

    // 执行 svn info 测试连接
    let svn_creds = crate::svn::types::SvnCredentials {
        username: params.credentials.username,
        password: params.credentials.password,
        save_to_cache: params.credentials.save_to_cache,
    };

    let result = svn::executor::run_svn(
        &["info", "--xml", &params.url],
        ".",
        Some(&svn_creds),
    ).await.map_err(|e| {
        // 认证失败不直接传播 AppError::SvnAuthFailed，而是转为 SvnCommand
        // 以便前端统一按 SVN_EXEC_FAILED 错误码处理并触发认证重试流程
        match e {
            AppError::SvnAuthFailed(msg) => {
                log::warn!("test_connection 认证失败: {}", msg);
                AppError::SvnCommand(format!("SVN 认证失败 - {}", msg))
            }
            other => other,
        }
    })?;

    // 解析并返回 realm 信息
    let info = svn::parser::parse_info(&result)?;
    Ok(format!("连接成功 - 仓库: {} (revision {})", info.url, info.revision))
}

/// 保存凭据到 ~/.subversion/auth/ 缓存。
/// 通过执行一次 svn info 使用提供的凭据触发认证，认证成功后 svn 自动写入缓存。
#[tauri::command]
pub async fn save_credentials(params: SaveCredentialsParams) -> Result<String, AppError> {
    if params.url.is_empty() {
        return Err(AppError::InvalidInput("仓库 URL 不能为空".into()));
    }
    if params.credentials.username.is_empty() || params.credentials.password.is_empty() {
        return Err(AppError::InvalidInput("用户名和密码不能为空".into()));
    }

    let svn_creds = crate::svn::types::SvnCredentials {
        username: params.credentials.username,
        password: params.credentials.password,
        save_to_cache: true, // 此操作目的就是保存到缓存
    };

    // 执行 svn info 触发认证（不带 --non-interactive 以允许 svn 写入缓存）
    svn::executor::run_svn(
        &["info", "--xml", &params.url],
        ".",
        Some(&svn_creds),
    ).await?;

    log::info!("凭据已缓存到 ~/.subversion/auth/");
    Ok("凭据已保存".to_string())
}

/// 清除 ~/.subversion/auth/ 中匹配给定 URL 仓库的缓存凭据。
/// 通过扫描 auth 缓存文件并删除匹配当前仓库 realm 的文件。
#[tauri::command]
pub async fn clear_credentials(params: ClearCredentialsParams) -> Result<String, AppError> {
    if params.url.is_empty() {
        return Err(AppError::InvalidInput("仓库 URL 不能为空".into()));
    }

    // 获取 svn auth 缓存目录
    let home = dirs::home_dir().ok_or_else(|| {
        AppError::Repo("无法获取用户主目录".into())
    })?;
    let auth_dir = home.join(".subversion").join("auth").join("svn.simple");

    if !auth_dir.exists() {
        return Ok("无缓存凭据需要清除".to_string());
    }

    // 从 URL 中提取主机名用于匹配
    let host = extract_host_for_match(&params.url);

    let mut removed = 0u32;
    // 读取所有缓存文件
    if let Ok(entries) = fs::read_dir(&auth_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                // 读取文件内容，检查是否包含目标 URL 或主机名
                if let Ok(content) = fs::read_to_string(&path) {
                    if (content.contains(&host) || content.contains(&params.url))
                        && fs::remove_file(&path).is_ok() {
                            removed += 1;
                            log::info!("已删除凭据缓存文件: {:?}", path);
                        }
                }
            }
        }
    }

    Ok(format!("已清除 {} 个凭据缓存文件", removed))
}

/// 从 URL 中提取主机名用于缓存匹配
pub(crate) fn extract_host_for_match(url: &str) -> String {
    url.trim_start_matches("svn://")
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("svn+ssh://")
        .split('/')
        .next()
        .unwrap_or(url)
        .to_string()
}

// ── 账号管理 ──────────────────────────────────────

/// 缓存的凭据条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachedCredential {
    pub id: String,
    pub username: String,
    pub realm: String,
}

/// 删除缓存的凭据参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteCachedCredentialParams {
    pub id: String,
}

/// 更新凭据密码参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCachedCredentialPasswordParams {
    pub id: String,
    pub new_password: String,
}

/// 获取 ~/.subversion/auth/svn.simple/ 目录路径
fn get_svn_auth_simple_dir() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir().ok_or_else(|| {
        AppError::Repo("无法获取用户主目录".into())
    })?;
    let auth_dir = home.join(".subversion").join("auth").join("svn.simple");
    Ok(auth_dir)
}

/// 解析 SVN simple auth 缓存文件内容，提取 username 和 realm。
///
/// 文件格式（SVN wc_utf8 格式）：
/// ```text
/// K 8
/// passtype
/// V 8
/// keychain
/// K 15
/// svn:realmstring
/// V 44
/// <https://example.com:443> SVN Server
/// K 8
/// username
/// V 5
/// user1
/// END
/// ```
fn parse_svn_simple_auth(content: &str) -> Option<(String, String)> {
    let mut lines = content.lines();
    let mut username = None;
    let mut realm = None;

    loop {
        let line = lines.next()?;
        if line == "END" {
            break;
        }
        if line.starts_with("K ") {
            // 下一行是 key 名
            if let Some(key) = lines.next() {
                // key 后是 V <length> 行
                if let Some(_v_line) = lines.next() {
                    // 再下一行是 value
                    if let Some(value) = lines.next() {
                        match key {
                            "username" => username = Some(value.to_string()),
                            "svn:realmstring" => realm = Some(value.to_string()),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    match (username, realm) {
        (Some(u), Some(r)) => Some((u, r)),
        _ => None,
    }
}

/// 列出所有缓存的 SVN 凭据（不含密码）
#[tauri::command]
pub async fn list_cached_credentials() -> Result<Vec<CachedCredential>, AppError> {
    let auth_dir = get_svn_auth_simple_dir()?;

    if !auth_dir.exists() {
        return Ok(Vec::new());
    }

    let mut credentials = Vec::new();

    if let Ok(entries) = fs::read_dir(&auth_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let id = path.file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            if let Ok(content) = fs::read_to_string(&path) {
                if let Some((username, realm)) = parse_svn_simple_auth(&content) {
                    credentials.push(CachedCredential {
                        id,
                        username,
                        realm,
                    });
                }
            }
        }
    }

    Ok(credentials)
}

/// 删除指定的缓存凭据
#[tauri::command]
pub async fn delete_cached_credential(params: DeleteCachedCredentialParams) -> Result<String, AppError> {
    let auth_dir = get_svn_auth_simple_dir()?;
    let target_path = auth_dir.join(&params.id);

    if !target_path.exists() {
        return Err(AppError::InvalidInput(format!("凭据缓存文件不存在: {}", params.id)));
    }

    // 安全检查：确保目标文件在 svn.simple 目录内
    let canonical = target_path.canonicalize().map_err(|_| {
        AppError::InvalidInput("无法访问凭据缓存文件".into())
    })?;
    let canonical_dir = auth_dir.canonicalize().map_err(|_| {
        AppError::Repo("无法访问 SVN 认证目录".into())
    })?;
    if !canonical.starts_with(&canonical_dir) {
        return Err(AppError::InvalidInput("非法的凭据缓存文件路径".into()));
    }

    fs::remove_file(&target_path).map_err(|e| AppError::Io(e))?;
    log::info!("已删除凭据缓存: {}", params.id);
    Ok("凭据已删除".to_string())
}

/// 更新指定缓存凭据的密码。
/// 读取旧缓存获取用户名和 realm，删除旧缓存，用新密码重新触发认证来更新缓存。
#[tauri::command]
pub async fn update_cached_credential_password(params: UpdateCachedCredentialPasswordParams) -> Result<String, AppError> {
    let auth_dir = get_svn_auth_simple_dir()?;
    let target_path = auth_dir.join(&params.id);

    if !target_path.exists() {
        return Err(AppError::InvalidInput(format!("凭据缓存文件不存在: {}", params.id)));
    }

    // 读取旧缓存获取 username 和 realm
    let content = fs::read_to_string(&target_path).map_err(AppError::Io)?;
    let (username, realm) = parse_svn_simple_auth(&content).ok_or_else(|| {
        AppError::InvalidInput("无法解析凭据缓存文件".into())
    })?;

    // 从 realm 中提取 URL（格式如: <https://example.com:443> SVN Server）
    let url = realm
        .split('>')
        .next()
        .and_then(|s| s.strip_prefix('<'))
        .map(|s| s.trim())
        .ok_or_else(|| AppError::InvalidInput("无法从 realm 提取仓库 URL".into()))?;

    // 删除旧缓存
    fs::remove_file(&target_path).map_err(AppError::Io)?;

    // 用新密码重新触发认证（保存到缓存）
    let svn_creds = crate::svn::types::SvnCredentials {
        username: username.clone(),
        password: params.new_password,
        save_to_cache: true,
    };

    svn::executor::run_svn(
        &["info", "--xml", url],
        ".",
        Some(&svn_creds),
    ).await?;

    log::info!("已更新凭据密码: {}", params.id);
    Ok("密码已更新".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_host_for_match_https() {
        let host = extract_host_for_match("https://svn.example.com/repo/trunk");
        assert_eq!(host, "svn.example.com");
    }

    #[test]
    fn test_extract_host_for_match_svn() {
        let host = extract_host_for_match("svn://svn.example.com/repo");
        assert_eq!(host, "svn.example.com");
    }

    #[test]
    fn test_extract_host_for_match_ssh() {
        let host = extract_host_for_match("svn+ssh://svn.example.com/repo");
        assert_eq!(host, "svn.example.com");
    }

    #[test]
    fn test_test_connection_params_deserialize() {
        let json = r#"{
            "url": "https://svn.example.com/repo",
            "credentials": {
                "username": "user1",
                "password": "pass1",
                "saveToCache": true
            }
        }"#;
        let params: TestConnectionParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://svn.example.com/repo");
        assert_eq!(params.credentials.username, "user1");
        assert!(params.credentials.save_to_cache);
    }

    #[test]
    fn test_save_credentials_params_deserialize() {
        let json = r#"{
            "url": "https://svn.example.com/repo",
            "credentials": {
                "username": "user1",
                "password": "pass1",
                "saveToCache": true
            }
        }"#;
        let params: SaveCredentialsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.credentials.password, "pass1");
    }

    #[test]
    fn test_clear_credentials_params_deserialize() {
        let json = r#"{"url": "https://svn.example.com/repo"}"#;
        let params: ClearCredentialsParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://svn.example.com/repo");
    }
}
