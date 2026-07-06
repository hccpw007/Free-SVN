//! macOS Shell 集成——注册 Finder 右键菜单。
//!
//! 方案：在 `~/Library/Services/` 生成 Automator Quick Action（`.workflow`）。
//! macOS 会把这些服务呈现在 Finder 右键「快速操作 / 服务」子菜单，
//! 对文件夹与文件均生效。服务触发时通过 `--svn-cmd` 调起本应用二进制，
//! 由 `tauri-plugin-single-instance` 把参数转交首实例处理（见 `lib.rs`）。
//!
//! 选择 Quick Action 而非 Finder Sync Extension 的原因：
//! 无需 Xcode / `.appex` / 代码签名即可工作，零外部依赖、纯 Rust 生成。

use std::fs;
use std::path::PathBuf;

use crate::models::error::AppError;

/// 一项右键菜单：菜单文案 + 传给 `--svn-cmd` 的命令字面量。
struct MenuEntry {
    name: &'static str,
    cmd: &'static str,
}

/// 计算用于被 Automator 服务调起的本应用二进制绝对路径。
///
/// 生产模式：`…/Free-SVN.app/Contents/MacOS/free-svn`。
/// 开发模式：`target/{debug,release}/free-svn`（裸二进制，仍在该路径被启动）。
fn resolve_app_binary() -> Result<String, AppError> {
    let exe = std::env::current_exe().map_err(AppError::Io)?;
    Ok(exe.to_string_lossy().to_string())
}

/// 对字符串做最小 XML 转义（路径中可能出现的 `&`/`<`/`>`）。
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// 生成单个 Quick Action `.workflow` bundle。
///
/// `bundle_path`：`~/Library/Services/SVN-<cmd>.workflow`
/// `name`：服务菜单文案（中文）
/// `cmd`：传给 `--svn-cmd` 的命令字面量
/// `app_bin`：本应用二进制绝对路径
fn generate_workflow_bundle(
    bundle_path: &PathBuf,
    name: &str,
    cmd: &str,
    app_bin: &str,
) -> Result<(), AppError> {
    let contents = bundle_path.join("Contents");
    fs::create_dir_all(&contents).map_err(AppError::Io)?;

    // —— Info.plist：服务注册元信息 ——
    // NSSendFileTypes 同时声明 public.folder 与 public.item，
    // 使服务对文件夹与文件右键均出现。
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>NSServices</key>
    <array>
        <dict>
            <key>NSMenuItem</key>
            <dict><key>default</key><string>{name}</string></dict>
            <key>NSMessage</key><string>runWorkflowAsService</string>
            <key>NSSendFileTypes</key>
            <array>
                <string>public.folder</string>
                <string>public.item</string>
            </array>
            <key>NSReturnTypes</key>
            <array/>
            <key>NSRequiredFiles</key>
            <true/>
        </dict>
    </array>
</dict>
</plist>"#,
        name = xml_escape(name),
    );
    fs::write(contents.join("Info.plist"), info_plist).map_err(AppError::Io)?;

    // —— document.wflow：Automator 工作流定义 ——
    // 含一个「Run Shell Script」action：shell=/bin/sh、传递输入作为参数（"$@"）。
    let cmd_line = format!(r#""{}" --svn-cmd {} "$@""#, xml_escape(app_bin), cmd);
    let document_wflow = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0">
<dict>
    <key>AMWorkflowBuildVersion</key><string>1.0</string>
    <key>AMWorkflowActions</key>
    <array>
        <dict>
            <key>action</key>
            <dict>
                <key>AMActionName</key><string>Run Shell Script</string>
                <key>AMAccepts</key>
                <dict>
                    <key>Container</key><string>List</string>
                    <key>Optional</key><false/>
                    <key>Types</key>
                    <array><string>com.apple.cocoa.path</string></array>
                </dict>
                <key>AMApplication</key>
                <array>
                    <string>Automator</string>
                    <string>1.1</string>
                </array>
                <key>AMBundleIdentifier</key><string>com.apple.RunShellScript</string>
                <key>AMParameterProperties</key>
                <dict>
                    <key>COMMAND_STRING</key>
                    <dict><key>default value</key><string>{cmd_line}</string></dict>
                    <key>CheckedString</key>
                    <dict><key>default value</key><string>0</string></dict>
                    <key>inputMethod</key>
                    <dict><key>default value</key><integer>1</integer></dict>
                    <key>shell</key>
                    <dict><key>default value</key><string>/bin/sh</string></dict>
                    <key>source</key><dict/>
                </dict>
                <key>AMProvides</key>
                <dict>
                    <key>Container</key><string>List</string>
                    <key>Types</key>
                    <array><string>com.apple.cocoa.path</string></array>
                </dict>
                <key>ActionBundlePath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>ActionName</key><string>Run Shell Script</string>
                <key>ActionPath</key>
                <string>/System/Library/Automator/Run Shell Script.action</string>
                <key>Input</key><dict><key>Class</key><string>InlineView</string></dict>
                <key>Result</key><dict><key>Class</key><string>NSView</string></dict>
                <key>ActionUUID</key><string>00000000-0000-0000-0000-000000000000</string>
            </dict>
        </dict>
    </array>
    <key>AMWorkflowMetadata</key>
    <dict>
        <key>AMWorkflowSummary</key><string>{summary}</string>
        <key>applicationBundleIdentifier</key><string>com.free-svn.app</string>
    </dict>
</dict>
</plist>"#,
        cmd_line = xml_escape(&cmd_line),
        summary = xml_escape(&format!("SVN {name}")),
    );
    fs::write(contents.join("document.wflow"), document_wflow).map_err(AppError::Io)?;

    Ok(())
}

/// 删除一个已存在的 `.workflow` bundle 后重建——保证每次注册都写入最新版。
fn recreate_bundle(bundle: &PathBuf) -> Result<(), AppError> {
    if bundle.exists() {
        fs::remove_dir_all(bundle).map_err(AppError::Io)?;
    }
    fs::create_dir_all(bundle).map_err(AppError::Io)?;
    Ok(())
}

/// 注册全部 Quick Action：工作副本 11 项 + 普通文件夹检出 1 项，共 12 个菜单。
///
/// 菜单文案与设计文档 §3.1 对齐。无法像 Finder Sync 那样按「是否为
/// 工作副本」动态显隐菜单项，故 12 项对任意文件夹/文件均显示；
/// 具体行为在前端 `handleShellCommand` 内按路径是否为工作副本决定。
pub fn register() -> Result<(), AppError> {
    let home =
        dirs::home_dir().ok_or_else(|| AppError::Repo("无法获取用户目录".into()))?;
    let svc_dir = home.join("Library/Services");
    fs::create_dir_all(&svc_dir).map_err(AppError::Io)?;

    let app_bin = resolve_app_binary()?;

    let menus = [
        MenuEntry { name: "SVN 提交...", cmd: "commit" },
        MenuEntry { name: "SVN 更新", cmd: "update" },
        MenuEntry { name: "SVN 查看日志", cmd: "log" },
        MenuEntry { name: "SVN 更新到版本...", cmd: "update-rev" },
        MenuEntry { name: "SVN 差异对比", cmd: "diff" },
        MenuEntry { name: "SVN 还原...", cmd: "revert" },
        MenuEntry { name: "SVN 切换(Switch)...", cmd: "switch" },
        MenuEntry { name: "SVN 分支/标签...", cmd: "branch-tag" },
        MenuEntry { name: "SVN 合并...", cmd: "merge" },
        MenuEntry { name: "SVN 清理", cmd: "cleanup" },
        MenuEntry { name: "SVN 导出...", cmd: "export" },
        MenuEntry { name: "SVN 检出...", cmd: "checkout" },
    ];

    for m in &menus {
        let bundle = svc_dir.join(format!("SVN-{}.workflow", m.cmd));
        recreate_bundle(&bundle)?;
        generate_workflow_bundle(&bundle, m.name, m.cmd, &app_bin)?;
    }

    // 通知 Services 缓存刷新，使新服务被 Finder/LaunchServices 识别。
    let _ = std::process::Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .output();
    // 同步刷新 LaunchServices 注册表（部分 macOS 版本需此步才显示服务）。
    let _ = std::process::Command::new(
        "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister",
    )
    .arg("-f")
    .arg(&app_bin)
    .output();

    log::info!(
        "macOS Finder 右键菜单注册成功（{} 个 Quick Action）",
        menus.len()
    );

    Ok(())
}

/// 卸载全部 SVN Quick Action。
pub fn unregister() -> Result<(), AppError> {
    let home =
        dirs::home_dir().ok_or_else(|| AppError::Repo("无法获取用户目录".into()))?;
    let svc_dir = home.join("Library/Services");

    if svc_dir.exists() {
        for entry in fs::read_dir(&svc_dir).map_err(AppError::Io)? {
            let entry = entry.map_err(AppError::Io)?;
            let name = entry.file_name().to_string_lossy().to_string();
            // 移除本项目生成的所有 SVN-*.workflow
            if name.starts_with("SVN-")
                && entry.path().extension() == Some(std::ffi::OsStr::new("workflow"))
            {
                let _ = fs::remove_dir_all(entry.path());
            }
        }
    }

    let _ = std::process::Command::new("/System/Library/CoreServices/pbs")
        .arg("-flush")
        .output();

    log::info!("macOS Finder 右键菜单已卸载");

    Ok(())
}