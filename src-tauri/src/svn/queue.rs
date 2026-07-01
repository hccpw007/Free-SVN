use crate::models::error::AppError;
use std::sync::Mutex;
use tokio::task::JoinHandle;

/// SVN 写操作互斥队列。
/// 同一工作副本同一时间只允许一个写操作，只读操作不经过此队列。
pub struct SvnQueue {
    write_lock: Mutex<Option<JoinHandle<()>>>,
}

impl SvnQueue {
    pub fn new() -> Self {
        Self {
            write_lock: Mutex::new(None),
        }
    }

    /// 尝试获取写操作锁。失败返回 OperationInProgress
    pub fn try_lock(&self) -> Result<(), AppError> {
        let mut guard = self
            .write_lock
            .lock()
            .map_err(|e| AppError::Repo(format!("Mutex 错误: {}", e)))?;
        if guard.is_some() {
            return Err(AppError::OperationInProgress);
        }
        *guard = Some(tokio::spawn(async {}));
        Ok(())
    }

    /// 释放写操作锁
    pub fn unlock(&self) {
        if let Ok(mut guard) = self.write_lock.lock() {
            *guard = None;
        }
    }

    /// 检查写操作锁是否被占用
    pub fn is_locked(&self) -> bool {
        self.write_lock
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }

    /// 设置当前写操作的 JoinHandle（executor 层在启动长操作后调用）
    pub fn set_handle(&self, handle: JoinHandle<()>) {
        if let Ok(mut guard) = self.write_lock.lock() {
            *guard = Some(handle);
        }
    }
}

impl Default for SvnQueue {
    fn default() -> Self {
        Self::new()
    }
}
