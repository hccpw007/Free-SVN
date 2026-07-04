use tauri::Emitter;
use tauri::Manager;

/// 进度窗口已就绪信号——当 ProgressWindowPage.vue 挂载并完成监听器注册后调用。
/// 后端收到后发出全局事件，主窗口（App.vue）接收后会将当前操作状态
/// （待操作文件列表、进度等）转发给进度窗口，防止因窗口创建时序导致的
/// pending 文件列表丢失。
#[tauri::command]
pub fn progress_window_ready(app_handle: tauri::AppHandle) {
    app_handle.emit("progress-window:ready", ()).ok();
}

/// 关闭进度窗口——由进度窗口内部调用，通过 Rust 端按 label 查找并销毁窗口，
/// 避免从窗口内部自销毁导致的不可预期行为。
#[tauri::command]
pub fn close_progress_window(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("progress") {
        let _ = window.destroy();
    }
}
