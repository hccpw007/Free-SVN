// Tauri Command 模块入口——阶段三填充具体操作
pub mod status;
pub mod diff;
pub mod log;
pub mod checkout;
pub mod update;
pub mod commit;
pub mod file_ops;
pub mod ignore;
pub mod branch_ops;
pub mod cleanup;
pub mod lock;
pub mod relocate;
pub mod cancel;
pub mod logs;
pub mod network;      // check_network
pub mod settings;     // load_settings / save_settings
pub mod branches;     // list_branches
pub mod auth;         // v5 新增：test_connection / save_credentials / clear_credentials
pub mod auth_account; // 账号管理：list_cached_credentials / delete_cached_credential / update_cached_credential_password
pub mod tray;         // 系统托盘徽章命令
pub mod auto_start;   // 开机自启动命令
pub mod progress;     // 进度窗口同步命令
