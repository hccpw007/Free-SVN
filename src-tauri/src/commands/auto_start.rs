use tauri::AppHandle;
use tauri::Manager;
use std::fs;

/// 设置开机自启动
///
/// macOS：写入 ~/Library/LaunchAgents/ 下的 plist 文件
/// Windows：写入 HKEY_CURRENT_USER\\Software\\Microsoft\\Windows\\CurrentVersion\\Run
/// Linux：创建/删除 ~/.config/autostart/ 下的 .desktop 文件
#[tauri::command]
pub fn set_auto_start(app: AppHandle, enabled: bool) -> Result<(), String> {
    let app_name = "com.free-svn.app";
    let app_exe = app.path().app_local_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?
        .join("Free-SVN");
    let app_exe_str = app_exe.to_string_lossy().to_string();

    #[cfg(target_os = "macos")]
    {
        let plist_dir = dirs::home_dir()
            .ok_or("Cannot find home directory")?
            .join("Library/LaunchAgents");
        fs::create_dir_all(&plist_dir)
            .map_err(|e| format!("Failed to create LaunchAgents dir: {}", e))?;
        let plist_path = plist_dir.join(format!("{}.plist", app_name));
        if enabled {
            let plist_content = format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
    <key>Label</key><string>{}</string>
    <key>ProgramArguments</key><array><string>{}</string></array>
    <key>RunAtLoad</key><true/>
    <key>KeepAlive</key><false/>
</dict></plist>"#,
                app_name, app_exe_str
            );
            fs::write(&plist_path, plist_content)
                .map_err(|e| format!("Failed to write plist: {}", e))?;
        } else {
            let _ = fs::remove_file(&plist_path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let run_key = hkcu.open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            winreg::enums::KEY_SET_VALUE,
        ).map_err(|e| format!("Failed to open registry key: {}", e))?;
        if enabled {
            run_key.set_value(app_name, &app_exe_str)
                .map_err(|e| format!("Failed to set registry value: {}", e))?;
        } else {
            let _ = run_key.delete_value(app_name);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let autostart_dir = dirs::config_dir()
            .ok_or("Cannot find config directory")?
            .join("autostart");
        fs::create_dir_all(&autostart_dir)
            .map_err(|e| format!("Failed to create autostart dir: {}", e))?;
        let desktop_path = autostart_dir.join(format!("{}.desktop", app_name));
        if enabled {
            let desktop_content = format!(
                "[Desktop Entry]\nType=Application\nName=Free-SVN\nExec={}\nX-GNOME-Autostart-enabled=true\n",
                app_exe_str
            );
            fs::write(&desktop_path, desktop_content)
                .map_err(|e| format!("Failed to write .desktop: {}", e))?;
        } else {
            let _ = fs::remove_file(&desktop_path);
        }
    }

    Ok(())
}
