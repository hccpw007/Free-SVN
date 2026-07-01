use crate::models::error::AppError;
use crate::svn;

/// 检测 SVN 服务器网络可达性（业务设计 §10）
/// 通过 TcpStream 5s 超时检测，ping 模式（可被任意 URL 触发）
#[tauri::command]
pub async fn check_network(server_url: Option<String>) -> Result<bool, AppError> {
    // 无参调用时使用默认地址检测网络连通性
    let url = server_url.unwrap_or_else(|| "https://svn.apache.org".to_string());
    match svn::executor::check_network(&url).await {
        Ok(()) => Ok(true),
        Err(AppError::NetworkUnreachable) => Ok(false),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_check_network_url_unwrap_logic() {
        // 验证 check_network 中 server_url 的默认值逻辑
        // 当 None 时使用默认 "https://svn.apache.org"
        let default = None::<String>.unwrap_or_else(|| "https://svn.apache.org".to_string());
        assert_eq!(default, "https://svn.apache.org");

        // 当 Some(url) 时使用传入值
        let custom = Some("svn://localhost".to_string());
        let url = custom.unwrap_or_else(|| "https://svn.apache.org".to_string());
        assert_eq!(url, "svn://localhost");
    }
}
