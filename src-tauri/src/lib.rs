mod commands;
mod svn;
mod models;
mod services;
mod tools;
mod shell_integration;
mod logging;
mod config;

use tauri::{
    AppHandle, Manager, Emitter,
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
};
use tauri_plugin_notification::NotificationExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // 第二实例启动时激活首实例窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();

                // 解析 --svn-cmd 参数（右键菜单触发）
                if let Some(cmd_idx) = args.iter().position(|a| a == "--svn-cmd") {
                    let cmd = args.get(cmd_idx + 1).cloned().unwrap_or_default();
                    let files: Vec<String> = args.iter()
                        .skip(cmd_idx + 2)
                        .filter(|a| !a.starts_with("--"))
                        .cloned()
                        .collect();
                    if !cmd.is_empty() {
                        let _ = window.emit("svn:shell-cmd", serde_json::json!({
                            "command": cmd,
                            "files": files,
                        }));
                    }
                }
            }
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir { file_name: Some("free-svn.log".into()) },
                ))
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(crate::svn::queue::SvnQueue::new())
        .setup(|app| {
            // ── 1. 初始化日志系统 ──
            crate::logging::logger::init(app.handle())?;

            // ── 2. 初始化配置存储 ──
            crate::config::store::init(app.handle())?;

            // ── 3. 构建系统托盘菜单 ──
            use tauri::menu::{MenuBuilder, MenuItemBuilder};

            let show = MenuItemBuilder::with_id("show", "Show Main Window")
                .accelerator("CmdOrCtrl+Shift+S").build(app)?;
            let update = MenuItemBuilder::with_id("update", "SVN Update").build(app)?;
            let cleanup = MenuItemBuilder::with_id("cleanup", "SVN Cleanup").build(app)?;
            let about = MenuItemBuilder::with_id("about", "About Free-SVN").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit")
                .accelerator("CmdOrCtrl+Q").build(app)?;

            let menu = MenuBuilder::new(app)
                .item(&show)
                .separator()
                .item(&update)
                .item(&cleanup)
                .separator()
                .item(&about)
                .separator()
                .item(&quit)
                .build()?;

            // ── 4. 创建系统托盘图标 ──
            let tray_icon = app.default_window_icon()
                .cloned()
                .unwrap_or_else(|| tauri::image::Image::new(&[0u8; 256], 16, 16));
            TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .tooltip("Free-SVN")
                .on_menu_event(move |app, event| {
                    let app_handle = app.clone();
                    match event.id().as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "update" => {
                            let cwd = crate::config::store::current_workspace()
                                .unwrap_or_default();
                            if !cwd.is_empty() {
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    let result = crate::commands::update::update_workspace(
                                        handle.clone(),
                                        crate::commands::update::UpdateParams {
                                            path: cwd, revision: None,
                                            depth: None, ignore_externals: None,
                                            credentials: None,
                                        },
                                        handle.state::<crate::svn::queue::SvnQueue>(),
                                    ).await;
                                    match result {
                                        Ok(_) => send_os_notification(&handle, "SVN Update", "完成"),
                                        Err(_) => send_os_notification(&handle, "SVN Update", "失败"),
                                    }
                                });
                            }
                        }
                        "cleanup" => {
                            let cwd = crate::config::store::current_workspace()
                                .unwrap_or_default();
                            if !cwd.is_empty() {
                                let handle = app_handle.clone();
                                tauri::async_runtime::spawn(async move {
                                    let _ = crate::commands::cleanup::cleanup_workspace(
                                        cwd,
                                        handle.state::<crate::svn::queue::SvnQueue>(),
                                    ).await;
                                    send_os_notification(&handle, "SVN Cleanup", "完成");
                                });
                            }
                        }
                        "about" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                            let _ = tauri_plugin_dialog::DialogExt::dialog(app)
                                .message("Free-SVN — A cross-platform SVN GUI tool\n\nVersion: 0.1.0")
                                .title("About Free-SVN")
                                .show(|_| {});
                        }
                        "quit" => { app.exit(0); }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up, ..
                    } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── 5. 注册 Shell 右键菜单 ──
            let _ = crate::shell_integration::register();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 3A — 只读操作
            commands::status::get_status,
            commands::status::get_info,
            commands::diff::get_diff,
            commands::log::get_log,
            commands::log::get_blame,
            // 3B — 核心写操作
            commands::checkout::checkout_repo,
            commands::update::update_workspace,
            commands::commit::create_commit,
            commands::file_ops::add_files,
            commands::file_ops::delete_files,
            commands::file_ops::revert_files,
            commands::file_ops::resolve_conflict,
            commands::ignore::set_ignore,
            // 3C — 剩余功能操作
            commands::branch_ops::switch_branch,
            commands::branch_ops::copy_branch_tag,
            commands::branch_ops::merge_branch,
            commands::cleanup::cleanup_workspace,
            commands::cleanup::export_workspace,
            commands::lock::lock_files,
            commands::lock::unlock_files,
            commands::relocate::relocate_repo,
            commands::ignore::property_ops,
            // 3D — 系统操作
            commands::cancel::cancel_operation,
            commands::logs::get_logs,
            commands::logs::export_logs,
            commands::network::check_network,
            commands::settings::load_settings,
            commands::settings::save_settings,
            commands::settings::get_svn_version,
            commands::branches::list_branches,
            // 3E — 认证操作（v5 新增）
            commands::auth::test_connection,
            commands::auth::save_credentials,
            commands::auth::clear_credentials,
            // 3F — 系统托盘 badge 命令
            commands::tray::set_tray_badge,
            // 3G — 开机自启动命令
            commands::auto_start::set_auto_start,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 发送操作系统通知
fn send_os_notification(app: &AppHandle, title: &str, body: &str) {
    let _ = app.notification()
        .builder()
        .title("Free-SVN")
        .body(format!("{}: {}", title, body))
        .show();
}
