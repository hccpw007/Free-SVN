use std::path::PathBuf;
use std::fs;
use crate::models::error::AppError;

/// 注册工作副本目录 12 项菜单 + 普通文件夹 1 项菜单
pub fn register() -> Result<(), AppError> {
    let home = dirs::home_dir().ok_or(AppError::Repo("无法获取用户目录".into()))?;
    let svc_dir = home.join("Library/Services");

    let exe = std::env::current_exe()
        .map_err(|e| AppError::Io(e))?;
    let app_bin = exe.to_string_lossy().to_string();

    // 11 项 SVN 操作的工作流
    let menus = [
        ("Commit", "commit"), ("Update", "update"), ("Log", "log"),
        ("Update to Revision", "update-rev"), ("Diff", "diff"), ("Revert", "revert"),
        ("Switch", "switch"), ("Branch/Tag", "branch-tag"), ("Merge", "merge"),
        ("Cleanup", "cleanup"), ("Export", "export"),
    ];

    for (name, cmd) in &menus {
        let bundle = svc_dir.join(format!("SVN-{}.workflow", name));
        if !bundle.exists() {
            generate_workflow_bundle(&bundle, name, cmd, &app_bin)?;
        }
    }

    // 普通文件夹 1 项（检出）
    let checkout_bundle = svc_dir.join("SVN-Checkout.workflow");
    if !checkout_bundle.exists() {
        generate_workflow_bundle(&checkout_bundle, "Checkout", "checkout", &app_bin)?;
    }

    // 通知 Finder 刷新服务
    std::process::Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .output()
        .map_err(AppError::Io)?;

    Ok(())
}

pub fn unregister() -> Result<(), AppError> {
    let svc_dir = dirs::home_dir()
        .ok_or(AppError::Repo("无法获取用户目录".into()))?
        .join("Library/Services");

    if svc_dir.exists() {
        for entry in fs::read_dir(&svc_dir).map_err(AppError::Io)? {
            let entry = entry.map_err(AppError::Io)?;
            if entry.file_name().to_string_lossy().starts_with("SVN-")
                && entry.path().extension() == Some(std::ffi::OsStr::new("workflow"))
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }

    // 刷新服务缓存
    std::process::Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .output()
        .map_err(AppError::Io)?;

    Ok(())
}

/// 生成完整的 .workflow bundle（Info.plist + document.wflow）
fn generate_workflow_bundle(
    bundle_path: &PathBuf,
    menu_name: &str,
    svn_cmd: &str,
    app_path: &str,
) -> Result<(), AppError> {
    let contents_dir = bundle_path.join("Contents");
    fs::create_dir_all(&contents_dir).map_err(AppError::Io)?;

    // Info.plist — 服务注册元信息
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict><key>default</key><string>SVN {name}</string></dict>
            <key>NSMessage</key><string>runWorkflowAsService</string>
            <key>NSRequiredFiles</key><integer>1</integer>
        </dict>
    </array>
</dict>
</plist>"#,
        name = menu_name
    );
    fs::write(contents_dir.join("Info.plist"), &info_plist).map_err(AppError::Io)?;

    // document.wflow — Automator 工作流定义
    let document_wflow = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>AMWorkflowActions</key><array><dict>
<key>action</key><key>AMAction</key><dict>
<key>AMActionName</key><string>Run Shell Script</string>
<key>AMAccepts</key><dict><key>Container</key><string>List</string>
<key>Optional</key><false/>
<key>Types</key><array><string>public.file-url</string></array></dict>
<key>AMParameterProperties</key><dict>
<key>COMMAND_STRING</key><string>{app_bin} --svn-cmd {cmd} "$@"</string>
<key>inputMethod</key><integer>1</integer>
<key>shell</key><integer>0</integer>
</dict></dict></dict></array>
<key>AMWorkflowMetadata</key><dict>
<key>AMWorkflowSummary</key><string>SVN {name}</string>
</dict></dict></plist>"#,
        cmd = svn_cmd,
        name = menu_name,
        app_bin = app_path,
    );
    fs::write(contents_dir.join("document.wflow"), &document_wflow).map_err(AppError::Io)?;

    Ok(())
}
