use std::fs;
#[cfg(unix)] use std::os::unix::fs::PermissionsExt;
use crate::models::error::AppError;

/// 注册工作副本右键菜单（11 项 + 1 检出）
pub fn register() -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::Repo("无法获取用户目录".into()))?;
    let script_dir = home.join(".local/share/nautilus/scripts/SVN");
    fs::create_dir_all(&script_dir).map_err(AppError::Io)?;

    let exe = std::env::current_exe()
        .map_err(|e| AppError::Io(e))?;
    let exe_str = exe.to_string_lossy().to_string();

    // 11 项 SVN 操作脚本
    let menus = [
        ("Commit", "commit"), ("Update", "update"), ("Log", "log"),
        ("Update to Rev", "update-rev"), ("Diff", "diff"), ("Revert", "revert"),
        ("Switch", "switch"), ("Branch/Tag", "branch-tag"), ("Merge", "merge"),
        ("Cleanup", "cleanup"), ("Export", "export"),
    ];

    for (label, cmd) in &menus {
        let script = script_dir.join(label);
        let content = format!(
            "#!/bin/bash\n\
             FILES=()\n\
             while IFS= read -r file; do FILES+=(\"$file\"); done <<< \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\"\n\
             \"{}\" --svn-cmd {} \"${{FILES[@]}}\"\n",
            exe_str, cmd
        );
        fs::write(&script, &content).map_err(AppError::Io)?;
        #[cfg(unix)]
        fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755))
            .map_err(AppError::Io)?;
    }

    // 普通文件夹 1 项（检出）
    let checkout_script = script_dir.join("Checkout");
    fs::write(&checkout_script, format!(
        "#!/bin/bash\n\
         FILES=()\n\
         while IFS= read -r file; do FILES+=(\"$file\"); done <<< \"$NAUTILUS_SCRIPT_SELECTED_FILE_PATHS\"\n\
         \"{}\" --svn-cmd checkout \"${{FILES[@]}}\"\n",
        exe_str
    )).map_err(AppError::Io)?;
    #[cfg(unix)]
    fs::set_permissions(&checkout_script, std::fs::Permissions::from_mode(0o755))
        .map_err(AppError::Io)?;

    Ok(())
}

pub fn unregister() -> Result<(), AppError> {
    let script_dir = dirs::home_dir()
        .ok_or(AppError::Repo("无法获取用户目录".into()))?
        .join(".local/share/nautilus/scripts/SVN");

    if script_dir.exists() {
        fs::remove_dir_all(&script_dir).map_err(AppError::Io)?;
    }

    Ok(())
}
