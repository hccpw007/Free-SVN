use crate::models::error::AppError;
use std::process::Command;

/// 外部差异工具参数
pub struct ExternalDiffParams {
    pub file1: String,
    pub file2: String,
}

/// 打开外部差异工具。
/// 不设超时、不捕获输出、不等待进程（分离模式）
pub fn open_external_diff(tool: &str, params: ExternalDiffParams) -> Result<(), AppError> {
    let f1 = params.file1.as_str();
    let f2 = params.file2.as_str();
    let (cmd, args) = match tool {
        "vscode" => ("code", vec!["--diff", f1, f2]),
        "bcomp" | "beyond-compare" => ("bcomp", vec![f1, f2]),
        "kaleidoscope" => ("ksdiff", vec![f1, f2]),
        custom => (custom, vec![f1, f2]),
    };

    if which::which(cmd).is_err() {
        return Err(AppError::ToolNotFound(
            format!("差异工具 '{}' 未找到，请检查设置", cmd),
        ));
    }

    let child = Command::new(cmd)
        .args(&args)
        .spawn()
        .map_err(|e| AppError::ToolNotFound(format!("启动失败: {}", e)))?;

    log::info!("已打开外部差异工具: {} (pid={})", cmd, child.id());
    Ok(())
}
