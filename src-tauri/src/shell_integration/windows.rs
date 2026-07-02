use crate::models::error::AppError;

const MENUS: &[(&str, &str)] = &[
    ("Commit", "commit"), ("Update", "update"), ("Log", "log"),
    ("Update to Rev", "update-rev"), ("Diff", "diff"), ("Revert", "revert"),
    ("Switch", "switch"), ("Branch/Tag", "branch-tag"), ("Merge", "merge"),
    ("Cleanup", "cleanup"), ("Export", "export"),
];

/// 注册工作副本右键菜单（11 项 + 1 检出）
pub fn register() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
            .map_err(|e| AppError::Repo(format!("无法打开注册表: {}", e)))?;

        let exe = std::env::current_exe()
            .map_err(|e| AppError::Io(e))?;
        let exe_str = exe.to_string_lossy().to_string();

        // 三组注册表路径（目录背景、目录本身、文件）
        let parents = [
            "Directory\\Background\\shell",
            "Directory\\shell",
            "*\\shell",
        ];

        for (name, cmd) in MENUS {
            for parent in &parents {
                let key_path = format!("{}\\SVN-{}", parent, name);
                let (key, _) = classes.create_subkey(&key_path)
                    .map_err(|e| AppError::Repo(format!("创建注册表键失败: {}", e)))?;
                key.set_value("", &format!("SVN {}", name))
                    .map_err(|e| AppError::Repo(format!("设置注册表值失败: {}", e)))?;
                let (cmd_key, _) = classes.create_subkey(format!("{}\\command", key_path))
                    .map_err(|e| AppError::Repo(format!("创建 command 键失败: {}", e)))?;
                cmd_key.set_value("", &format!(
                    r#""{}" --svn-cmd {} "%*""#,
                    exe_str, cmd
                ))
                .map_err(|e| AppError::Repo(format!("设置命令值失败: {}", e)))?;
            }
        }

        // 检出菜单（仅用于 Directory\shell）
        let checkout_path = "Directory\\shell\\SVN-Checkout";
        let (key, _) = classes.create_subkey(checkout_path)
            .map_err(|e| AppError::Repo(format!("创建检出键失败: {}", e)))?;
        key.set_value("", "SVN Checkout")
            .map_err(|e| AppError::Repo(format!("设置检出值失败: {}", e)))?;
        let (cmd_key, _) = classes.create_subkey(format!("{}\\command", checkout_path))
            .map_err(|e| AppError::Repo(format!("创建检出命令键失败: {}", e)))?;
        cmd_key.set_value("", &format!(
            r#""{}" --svn-cmd checkout "%*""#,
            exe_str
        ))
        .map_err(|e| AppError::Repo(format!("设置检出命令值失败: {}", e)))?;
    }

    Ok(())
}

/// 卸载右键菜单（精确删除 SVN 相关键值）
pub fn unregister() -> Result<(), AppError> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let classes = hkcu.open_subkey_with_flags("Software\\Classes", KEY_WRITE)
            .map_err(|e| AppError::Repo(format!("无法打开注册表: {}", e)))?;

        let parents = [
            "Directory\\Background\\shell",
            "Directory\\shell",
            "*\\shell",
        ];

        for (name, _) in MENUS {
            for parent in &parents {
                let key_path = format!("{}\\SVN-{}", parent, name);
                let _ = classes.delete_subkey_all(&key_path);
            }
        }

        let _ = classes.delete_subkey_all("Directory\\shell\\SVN-Checkout");
    }

    Ok(())
}
