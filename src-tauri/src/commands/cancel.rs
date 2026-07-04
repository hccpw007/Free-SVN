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

    // 1. 获取当前操作上下文（用于确定 cleanup 目标目录）
    let ctx = svn::executor::get_current_operation();

    // 2. 设置取消标志
    svn::executor::set_cancelled(true);

    // 3. 获取当前工作副本路径（用于 cleanup）
    let cwd = if let Some(ref c) = ctx {
        c.target_path.clone()
    } else {
        crate::config::store::current_workspace()
            .unwrap_or_default()
    };

    // 4. 终止子进程
    svn::executor::kill_current_process();

    // 5. cleanup 恢复工作副本状态
    if !cwd.is_empty() {
        let _ = svn::executor::run_svn(&["cleanup"], &cwd, None).await;
    }

    // 6. 重置取消标志
    svn::executor::set_cancelled(false);
    svn::executor::clear_current_operation();

    // 7. 释放写操作锁
    state.unlock();

    // 8. 通知前端（附带操作类型和目标目录）
    app_handle.emit("operation:cancelled", &crate::svn::types::CancelledPayload {
        reason: "operation cancelled by user".to_string(),
        operation: ctx.as_ref().map(|c| c.operation.clone()),
        target_path: ctx.as_ref().map(|c| c.target_path.clone()),
    }).ok();

    log::info!("operation cancelled");
    Ok("operation cancelled".to_string())
}

#[cfg(test)]
mod tests {
    // cancel_operation 依赖运行时状态（当前工作副本路径、运行中的子进程），
    // 在端到端测试中验证，单元测试仅确保类型签名正确。
}
