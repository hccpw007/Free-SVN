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
    use super::*;

    #[test]
    fn test_check_network_function_signature() {
        // 验证函数签名正确
    }
}
