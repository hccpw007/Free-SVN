use tauri::AppHandle;

/// 设置系统托盘徽章（操作进行中/完成）
#[tauri::command]
pub fn set_tray_badge(_app: AppHandle, _visible: bool) -> Result<(), String> {
    // 使用 tooltip 文字提示操作状态
    // Tauri 2 中各平台的托盘图标动态切换需要使用底层 tray_icon crate 的 API，
    // 此处保留命令签名供前端调用，后续可扩展实现图标切换
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
