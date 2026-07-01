use crate::models::error::AppError;

pub struct Settings {
    pub default_checkout_dir: String,
    pub global_ignore_pattern: String,
    pub diff_tool: String,
    pub merge_tool: String,
    pub show_unversioned: bool,
    pub language: String,
    pub auto_start: bool,
}

pub fn init(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> { todo!() }
pub fn load_settings() -> Result<Settings, AppError> { todo!() }
pub fn save_settings(settings: &Settings) -> Result<(), AppError> { todo!() }
pub fn recent_workspaces() -> Vec<String> { todo!() }
pub fn add_recent_workspace(path: &str) { todo!() }
