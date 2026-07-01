use crate::models::error::AppError;
use crate::logging;

/// 获取应用日志内容
#[tauri::command]
pub async fn get_logs() -> Result<String, AppError> {
    logging::logger::get_logs()
}

/// 导出日志为 .zip 文件
#[tauri::command]
pub async fn export_logs(target_path: String) -> Result<(), AppError> {
    if target_path.is_empty() {
        return Err(AppError::InvalidInput("target path cannot be empty".into()));
    }
    logging::logger::export_logs(&target_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_logs_empty_path() {
        // export_logs 的类型签名验证
    }
}
