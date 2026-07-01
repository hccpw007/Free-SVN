use crate::models::error::AppError;

pub fn init(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> { todo!() }
pub fn get_logs() -> Result<String, AppError> { todo!() }
pub fn export_logs(target_path: &str) -> Result<(), AppError> { todo!() }
