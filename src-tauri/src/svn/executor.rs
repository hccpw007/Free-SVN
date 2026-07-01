use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::process::Child;
use crate::models::error::AppError;

pub static CANCELLED: AtomicBool = AtomicBool::new(false);
pub static CURRENT_CHILD: Mutex<Option<Child>> = Mutex::new(None);

pub fn get_svn_path() -> std::path::PathBuf { todo!() }
pub async fn run_svn(args: &[&str], cwd: &str) -> Result<String, AppError> { todo!() }
pub fn is_cancelled() -> bool { CANCELLED.load(std::sync::atomic::Ordering::SeqCst) }
pub fn set_cancelled(val: bool) { CANCELLED.store(val, std::sync::atomic::Ordering::SeqCst) }
// 注：以下 check_network / validate_path 的返回类型为当前骨架定义，
//     2.2 完整实现时将保持一致，若有变更会在该步骤同步更新
pub async fn check_network(server_url: &str) -> Result<(), AppError> { todo!() }
pub fn validate_path(path: &str) -> Result<&str, AppError> { todo!() }
