use crate::models::error::AppError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_store::StoreExt;

/// 应用设置（11 个字段，对应业务设计 §5.7 + 前端 AppSettings）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_checkout_dir: String,
    pub global_ignore_pattern: String,
    pub diff_tool: String,
    pub merge_tool: String,
    pub diff_command_template: String,
    pub merge_command_template: String,
    pub fallback_to_builtin: bool,
    pub show_unversioned: bool,
    pub language: String,
    pub auto_start: bool,
    pub dark_mode: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_checkout_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy().to_string(),
            global_ignore_pattern: String::new(),
            diff_tool: "builtin".into(),
            merge_tool: "builtin".into(),
            diff_command_template: String::new(),
            merge_command_template: String::new(),
            fallback_to_builtin: true,
            show_unversioned: true,
            language: "system".into(),
            auto_start: false,
            dark_mode: false,
        }
    }
}

/// 全局 Store 引用
static STORE: once_cell::sync::OnceCell<Arc<tauri_plugin_store::Store<tauri::Wry>>> =
    once_cell::sync::OnceCell::new();

fn store() -> &'static Arc<tauri_plugin_store::Store<tauri::Wry>> {
    STORE.get().expect("Store 未初始化")
}

/// 初始化配置存储（lib.rs setup 中调用）
pub fn init(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let app_dir = app_handle.path().app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    fs::create_dir_all(&app_dir)?;

    let store_path = app_dir.join("settings.json");
    let store = app_handle.store(store_path.clone())?;

    // 损坏检测 + 自动恢复：尝试直接读取 JSON 文件检查是否合法
    if store_path.exists() {
        match fs::read_to_string(&store_path) {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    log::warn!("Store JSON 损坏, 备份并重置默认");
                    let bak = store_path.with_extension("json.bak");
                    let _ = fs::copy(&store_path, &bak);
                    save_settings(&Settings::default()).ok();
                }
            }
            Err(e) => {
                log::warn!("Store 文件读取失败 ({}), 将创建新默认文件", e);
            }
        }
    }

    STORE.set(store).map_err(|_| "Store 重复初始化")?;
    log::info!("配置存储初始化完成");
    Ok(())
}

/// 加载设置（失败返回默认值，不崩溃）
pub fn load_settings() -> Settings {
    store().get("settings")
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// 保存设置
pub fn save_settings(settings: &Settings) -> Result<(), AppError> {
    let value = serde_json::to_value(settings)
        .map_err(|e| AppError::Repo(format!("序列化失败: {}", e)))?;
    let store = store();
    store.set("settings", value);
    store.save().map_err(|e| AppError::Repo(format!("保存失败: {}", e)))?;
    Ok(())
}

/// 获取最近打开的工作副本列表
pub fn recent_workspaces() -> Vec<String> {
    store().get("recentWorkspaces")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .unwrap_or_default()
}

/// 添加工作副本到最近列表（去重 + 前置 + 上限 20）
pub fn add_recent_workspace(path: &str) {
    let mut list = recent_workspaces();
    if let Some(pos) = list.iter().position(|p| p == path) {
        list.remove(pos);
    }
    list.insert(0, path.to_string());
    list.truncate(20);

    let store = store();
    store.set("recentWorkspaces", serde_json::to_value(&list).unwrap_or_default());
    store.save().ok();
}

pub fn current_workspace() -> Option<String> {
    store().get("currentWorkspace")
        .and_then(|v| v.as_str().map(String::from))
}

pub fn set_current_workspace(path: &str) {
    let store = store();
    store.set("currentWorkspace", serde_json::Value::String(path.to_string()));
    store.save().ok();
    add_recent_workspace(path);
}
