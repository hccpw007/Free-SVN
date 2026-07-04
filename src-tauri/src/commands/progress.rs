use tauri::Emitter;
use tauri::Manager;

/// 进度窗口已就绪信号——当 ProgressWindowPage.vue 挂载并完成监听器注册后调用。
/// 后端收到后强制设置进度窗口尺寸并居中，然后发出全局事件，
/// 主窗口（App.vue）接收后会将当前操作状态转发给进度窗口。
#[tauri::command]
pub fn progress_window_ready(app_handle: tauri::AppHandle) {
    if let Some(window) = app_handle.get_webview_window("progress") {
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: 880.0,
            height: 587.0,
        }));
        let _ = window.center();
    }
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
