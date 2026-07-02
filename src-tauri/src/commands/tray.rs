use tauri::AppHandle;

/// 设置系统托盘徽章（操作进行中/完成）
#[tauri::command]
pub fn set_tray_badge(app: AppHandle, visible: bool) -> Result<(), String> {
    if let Some(tray) = app.get_tray_icon_by_id("main") {
        let _ = tray.set_tooltip(if visible { "SVN 操作进行中..." } else { "Free-SVN" });
        // 图标切换（准备带徽章的图标资源）
        if visible {
            let icon_bytes = include_bytes!("../../icons/tray-badge.png");
            if let Ok(img) = tauri::image::Image::from_bytes(icon_bytes) {
                let _ = tray.set_icon(Some(img));
            }
        } else {
            // 恢复默认图标
            if let Some(default_icon) = app.default_window_icon() {
                let _ = tray.set_icon(Some(default_icon.clone()));
            }
        }
    }
    Ok(())
}

/// macOS Dock badge 设置
#[cfg(target_os = "macos")]
pub fn set_dock_badge(visible: bool) {
    #[cfg(target_os = "macos")]
    {
        #[link(name = "AppKit", kind = "framework")]
        extern "C" {
            static NSApp: *mut std::ffi::c_void;
        }
        // 使用简洁的方式设置 Dock badge
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
}
