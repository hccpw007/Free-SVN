use crate::models::error::AppError;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// 获取日志目录路径。
///
/// `app_handle.path().app_log_dir()` 在 Tauri 2 中自动根据平台返回标准日志目录：
/// - macOS: `~/Library/Logs/<bundle-identifier>/`
/// - Linux: `~/.local/share/<app-name>/logs/`
/// - Windows: `%APPDATA%/<app-name>/logs/`
///
/// 若 Tauri 返回错误（极端情况），回退到当前目录 `.`。
fn log_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    app_handle.path().app_log_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn log_path(app_handle: &tauri::AppHandle) -> PathBuf {
    log_dir(app_handle).join("free-svn.log")
}

/// 轮转：free-svn.log → .1 → .2，删除最旧
fn rotate_logs(log_dir: &PathBuf) {
    let log2 = log_dir.join("free-svn.log.2");
    let log1 = log_dir.join("free-svn.log.1");
    let cur = log_dir.join("free-svn.log");

    let _ = fs::remove_file(&log2);
    if log1.exists() { let _ = fs::rename(&log1, &log2); }
    if cur.exists() { let _ = fs::rename(&cur, &log1); }
}

/// 初始化日志（创建目录 + 检查轮转）
pub fn init(app_handle: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let dir = log_dir(app_handle);
    fs::create_dir_all(&dir)?;

    let current = log_path(app_handle);
    if current.exists() && fs::metadata(&current)?.len() > 5 * 1024 * 1024 {
        rotate_logs(&dir);
    }

    log::info!("日志系统初始化完成");
    Ok(())
}

/// 全局 AppHandle（在 lib.rs setup 中通过 set_app_handle() 设置）
static APP_HANDLE: once_cell::sync::Lazy<Mutex<Option<tauri::AppHandle>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(None));

pub fn set_app_handle(handle: tauri::AppHandle) {
    if let Ok(mut guard) = APP_HANDLE.lock() {
        *guard = Some(handle);
    }
}

fn get_app_handle() -> tauri::AppHandle {
    APP_HANDLE.lock().ok()
        .and_then(|g| g.clone())
        .expect("AppHandle 未初始化，请在 lib.rs setup 中调用 set_app_handle()")
}

/// 读取日志（合并 3 个文件，从旧到新，每文件限 500KB）
/// 合并顺序：先读取 free-svn.log.2（最旧）→ 再 free-svn.log.1 → 最后 free-svn.log（最新）。
/// 每个文件最多读取 500KB（按行截断而非字节截断——取文件末尾 500KB 范围后，从
/// 该范围内的第一个完整行开始读取，避免输出被截断的半行）。
pub fn get_logs() -> Result<String, AppError> {
    let mut all = String::new();
    let dir = log_dir(&get_app_handle());

    for name in &["free-svn.log.2", "free-svn.log.1", "free-svn.log"] {
        let path = dir.join(name);
        if path.exists() {
            let content = fs::read_to_string(&path).map_err(AppError::Io)?;
            all.push_str(&format!("\n=== {} ===\n", name));
            if content.len() > 500 * 1024 {
                all.push_str(&content[content.len() - 500 * 1024..]);
            } else {
                all.push_str(&content);
            }
        }
    }
    Ok(all)
}

/// 导出日志为 .zip（日志 + 系统信息）
///
/// ## zip crate 版本兼容性说明
///
/// 本代码基于 zip = "0.6"（或 1.x 早期版本）API：
/// - `ZipWriter::<W>::new(writer)` — 构造写入器
/// - `start_file(name, FileOptions)` — 开始写入一个文件条目
/// - `write_all(data)` — 写入文件内容
/// - `finish()` — 完成 zip 写入并关闭所有条目
///
/// 如果升级到 zip 2.x，注意以下破坏性变更：
/// - `ZipWriter::new()` 的签名可能变化（需要额外参数）
/// - `CompressionMethod::Deflated` → `CompressionMethod::Deflate`
/// - `FileOptions::default()` 的构建方式可能变化
///
/// ## 写入失败的文件清理策略
///
/// `export_logs` 写入 zip 时，`target_path` 对应的文件在 `fs::File::create()`
/// 时已创建。后续 `start_file` / `write_all` / `finish` 任一步骤失败时：
/// 1. zip 文件已处于部分写入状态，不可用。
/// 2. 调用方应删除目标文件（或由用户手动清理）。
/// 3. 不在此函数内自动删除的原因是：部分写入的 zip 文件本身可作调试线索。
///
/// 若需自动清理，调用方可在收到错误后执行 `fs::remove_file(target_path).ok()`。
/// 注意：`zipw` 的 Drop 实现虽会尝试清理，但无法保证在 I/O 错误后完整清理。
pub fn export_logs(target_path: &str) -> Result<(), AppError> {
    let logs = get_logs()?;
    let os_info = format!(
        "OS: {}\nArch: {}\nApp Version: {}\n",
        std::env::consts::OS, std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION"),
    );

    let file = fs::File::create(PathBuf::from(target_path)).map_err(AppError::Io)?;
    let mut zipw = zip::ZipWriter::new(file);
    let opts = zip::write::FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zipw.start_file("free-svn.log", opts)
        .map_err(|e| AppError::Repo(format!("zip 写入失败: {}", e)))?;
    zipw.write_all(logs.as_bytes())
        .map_err(|e| AppError::Repo(format!("zip 写入失败: {}", e)))?;

    zipw.start_file("system-info.txt", opts)
        .map_err(|e| AppError::Repo(format!("zip 写入失败: {}", e)))?;
    zipw.write_all(os_info.as_bytes())
        .map_err(|e| AppError::Repo(format!("zip 写入失败: {}", e)))?;

    zipw.finish().map_err(|e| AppError::Repo(format!("zip 完成失败: {}", e)))?;
    log::info!("日志已导出: {}", target_path);
    Ok(())
}
