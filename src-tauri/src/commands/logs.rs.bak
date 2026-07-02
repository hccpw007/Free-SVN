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
    use crate::models::error::AppError;

    #[test]
    fn test_export_logs_empty_path_returns_error() {
        // export_logs 在空路径时应在调用 logging::logger::export_logs 之前返回 InvalidInput
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async { export_logs(String::new()).await });
        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => assert!(!msg.is_empty(), "错误消息不能为空"),
            _ => panic!("expected InvalidInput for empty export path"),
        }
    }

    #[test]
    #[ignore = "依赖 Tauri AppHandle 运行时环境，需在集成测试中验证"]
    fn test_get_logs_returns_ok() {
        // get_logs 内部调用 logging::logger::get_logs()，后者通过 get_app_handle() 获取 Tauri AppHandle
        // 在纯单元测试环境中无法构造 AppHandle，标注为 ignore，在集成测试中验证
    }
}
