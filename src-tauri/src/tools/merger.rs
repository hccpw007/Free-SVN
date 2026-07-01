use crate::models::error::AppError;
use std::process::Command;

/// 外部合并工具参数（mine/base/theirs/output）
pub struct ExternalMergeParams {
    pub mine: String,
    pub base: String,
    pub theirs: String,
    pub output: String,
}

/// 打开外部合并工具，传入 <mine> <base> <theirs> <output> 参数。
/// 不设超时、不捕获输出、不等待进程。
pub fn open_external_merge(tool: &str, params: ExternalMergeParams) -> Result<(), AppError> {
    let m = params.mine.as_str();
    let b = params.base.as_str();
    let t = params.theirs.as_str();
    let o = params.output.as_str();
    let (cmd, args): (&str, Vec<&str>) = match tool {
        "vscode" => ("code", vec![o]),
        "bcomp" | "beyond-compare" => {
            ("bcomp", vec![m, b, t, o])
        }
        "kaleidoscope" => {
            return Err(AppError::ToolNotFound(
                "Kaleidoscope 合并模式未实现，请使用 Beyond Compare 或自定义工具".into(),
            ));
        }
        custom => (custom, vec![m, b, t, o]),
    };

    if which::which(cmd).is_err() {
        return Err(AppError::ToolNotFound(
            format!("合并工具 '{}' 未找到，请检查设置", cmd),
        ));
    }

    let child = Command::new(cmd)
        .args(&args)
        .spawn()
        .map_err(|e| AppError::ToolNotFound(format!("启动失败: {}", e)))?;

    log::info!("已打开外部合并工具: {} (pid={})", cmd, child.id());
    Ok(())
}
