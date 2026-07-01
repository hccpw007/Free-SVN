use std::sync::Mutex;
use tokio::task::JoinHandle;
use crate::models::error::AppError;

pub struct SvnQueue {
    write_lock: Mutex<Option<JoinHandle<()>>>,
}
impl SvnQueue {
    pub fn new() -> Self { Self { write_lock: Mutex::new(None) } }
    pub fn try_lock(&self) -> Result<(), AppError> { todo!() }
    pub fn unlock(&self) { todo!() }
}
