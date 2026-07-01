use crate::config::store::Settings;
use crate::models::error::AppError;

/// 加载应用设置
#[tauri::command]
pub async fn load_settings() -> Result<Settings, AppError> {
    Ok(crate::config::store::load_settings())
}

/// 保存应用设置
#[tauri::command]
pub async fn save_settings(settings: Settings) -> Result<(), AppError> {
    crate::config::store::save_settings(&settings)
}

/// 获取 SVN CLI 版本号
/// 包装 tools::resolver::get_svn_version()，供 SettingsPage（步骤 6.9）调用
#[tauri::command]
pub async fn get_svn_version() -> Result<String, AppError> {
    crate::tools::resolver::get_svn_version()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_deserialize() {
        let json = r#"{
            "default_checkout_dir": "/tmp",
            "global_ignore_pattern": "",
            "diff_tool": "builtin",
            "merge_tool": "builtin",
            "diff_command_template": "",
            "merge_command_template": "",
            "fallback_to_builtin": true,
            "show_unversioned": true,
            "language": "system",
            "auto_start": false,
            "dark_mode": false
        }"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.default_checkout_dir, "/tmp");
        assert_eq!(settings.diff_tool, "builtin");
    }
}
