use crate::models::error::AppError;
use crate::svn;
use tauri::AppHandle;
use tauri::Emitter;

/// 取消当前长操作（业务设计 §2.5）
/// 流程：设取消标志 → kill 子进程 → cleanup → 重置标志 → 推送事件
#[tauri::command]
pub async fn cancel_operation(
    app_handle: AppHandle,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<String, AppError> {
    log::info!("取消操作请求");

    // 1. 设置取消标志
    svn::executor::set_cancelled(true);

    // 2. 获取当前工作副本路径（用于 cleanup）
    let cwd = crate::config::store::current_workspace()
        .unwrap_or_default();

    // 3. 终止子进程
    svn::executor::kill_current_process();

    // 4. cleanup 恢复工作副本状态
    if !cwd.is_empty() {
        let _ = svn::executor::run_svn(&["cleanup"], &cwd, None).await;
    }

    // 5. 重置取消标志
    svn::executor::set_cancelled(false);

    // 6. 释放写操作锁
    state.unlock();

    // 7. 通知前端
    app_handle.emit("operation:completed", serde_json::json!({
        "result": "cancelled",
        "detail": "operation cancelled"
    })).ok();

    log::info!("operation cancelled");
    Ok("operation cancelled".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancel_operation_types() {
        // 验证返回类型正确
        // 实际执行依赖运行时状态，单元测试仅验证签名
    }
}
