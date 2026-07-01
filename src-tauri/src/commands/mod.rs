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
pub mod auth;         // v5 新增：test_connection / save_credentials / clear_credentials（⚠️ auth.rs 在步骤 3.5 创建，本步骤编译会因找不到 auth.rs 报错，属预期行为）
