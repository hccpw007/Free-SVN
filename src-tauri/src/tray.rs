//! 系统托盘通知模块——发送操作系统通知。
//! 被 lib.rs 的托盘菜单事件回调调用。

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

/// 发送操作系统通知
pub fn send_os_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app
        .notification()
        .builder()
        .title("Free-SVN")
        .body(format!("{}: {}", title, body))
        .show();
}
