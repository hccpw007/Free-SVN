mod commands;
mod svn;
mod models;
mod services;
mod tools;
mod shell_integration;
mod logging;
mod config;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(crate::svn::queue::SvnQueue::new())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            logging::logger::init(app.handle())?;
            logging::logger::set_app_handle(app.handle().clone());
            config::store::init(app.handle())?;
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
            commands::network::check_network,    // check_network 网络检测
            commands::settings::load_settings,   // 配置加载
            commands::settings::save_settings,   // 配置保存
            commands::settings::get_svn_version, // SVN 版本号（P0 修复）
            commands::branches::list_branches,  // 列出分支（P0-2 修复）
            // 3E — 认证操作（v5 新增）
            commands::auth::test_connection,
            commands::auth::save_credentials,
            commands::auth::clear_credentials,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
