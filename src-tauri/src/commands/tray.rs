use tauri::AppHandle;
use once_cell::sync::Lazy;
use std::sync::Mutex;

// 缓存默认图标，避免重复创建
static DEFAULT_ICON: Lazy<Mutex<Option<tauri::image::Image<'static>>>> = Lazy::new(|| Mutex::new(None));

/// 设置系统托盘徽章（操作进行中/完成）
#[tauri::command]
pub fn set_tray_badge(app: AppHandle, visible: bool) -> Result<(), String> {
    // 通过 app.tray_by_id 获取系统托盘
    if let Some(tray) = app.tray_by_id("main") {
        if visible {
            // 设置操作进行中状态的 tooltip
            let _ = tray.set_tooltip(Some("SVN 操作进行中..."));
            // 将默认图标转为 'static 生命周期的 owned Image 再缓存
            let mut guard = DEFAULT_ICON.lock().map_err(|e| e.to_string())?;
            if guard.is_none() {
                if let Some(img) = app.default_window_icon() {
                    *guard = Some(img.clone().to_owned());
                }
            }
            if let Some(icon) = guard.as_ref() {
                let _ = tray.set_icon(Some(icon.clone()));
            }
        } else {
            // 恢复默认图标和 tooltip
            let _ = tray.set_tooltip(Some("Free-SVN"));
            if let Some(img) = app.default_window_icon() {
                let _ = tray.set_icon(Some(img.clone().to_owned()));
            }
        }
    }
    Ok(())
}

/// macOS Dock badge 设置（使用 osascript）
#[cfg(target_os = "macos")]
pub fn set_dock_badge(visible: bool) {
    let label = if visible { "●" } else { "" };
    let _ = std::process::Command::new("osascript")
        .args([
            "-e",
            &format!(
                "tell application \"System Events\" to set badge of (first process whose bundle identifier is \"com.free-svn.app\") to \"{}\"",
                label
            ),
        ])
        .output();
}
