use crate::models::error::AppError;
use std::path::PathBuf;
use std::process::Command;

/// 按名称在 PATH 中查找工具路径
pub fn resolve_tool_path(tool_name: &str) -> Option<PathBuf> {
    which::which(tool_name).ok()
}

/// 检测内置 svn 是否可执行。
///
/// ## 开发环境与生产环境行为差异
///
/// `validate_svn()` 和 `get_svn_version()` 的检测结果依赖于
/// `crate::svn::executor::get_svn_path()` 的返回值，该函数在不同阶段行为不同：
///
/// ### 开发阶段（阶段七之前）
/// `get_svn_path()` 返回 `PathBuf::from("svn")`，即从系统 PATH 中解析 svn。
/// 验证结果是"系统是否安装了 svn"，而非"内置 svn 是否可用"。
/// - macOS/Linux：通常 PATH 中有系统 svn 或 Homebrew 安装的 svn
/// - Windows：需用户手动安装 svn 客户端或通过 Scoop/Chocolatey 安装
/// - 验证通过仅表示 PATH 中有 svn，不代表内置 svn 可用
///
/// ### 生产阶段（阶段七打包后）
/// `get_svn_path()` 返回内置 svn 路径（如 `resources/svn/bin/svn`），
/// 验证结果是"内置 svn 是否可执行且版本正确"。
/// - 内置 svn 与应用一同分发，不依赖系统安装
/// - 验证通过表示用户无需手动安装 svn
/// - 验证失败（如资源文件损坏）时应提示重新安装应用
pub fn validate_svn() -> Result<(), AppError> {
    let svn_path = crate::svn::executor::get_svn_path();
    if !svn_path.exists() {
        return Err(AppError::SvnNotFound);
    }
    Command::new(&svn_path)
        .arg("--version")
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::SvnNotFound
            } else {
                AppError::Io(e)
            }
        })
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(AppError::SvnNotFound)
            }
        })
}

/// 获取 svn 版本号（svn --version --quiet）。
///
/// 开发阶段返回系统 PATH 中 svn 的版本号，生产阶段返回内置 svn 的版本号。
/// 行为差异同 `validate_svn()`。详见该函数的"开发环境与生产环境行为差异"说明。
pub fn get_svn_version() -> Result<String, AppError> {
    let svn_path = crate::svn::executor::get_svn_path();
    Command::new(&svn_path)
        .args(["--version", "--quiet"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::SvnNotFound
            } else {
                AppError::Io(e)
            }
        })
        .and_then(|output| {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
            } else {
                Err(AppError::SvnNotFound)
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_tool_path_nonexistent() {
        assert!(resolve_tool_path("nonexistent-tool-12345").is_none());
    }

    #[test]
    fn test_resolve_tool_path_existing() {
        let result = resolve_tool_path("sh");
        assert!(result.is_some());
        assert!(result.unwrap().exists());
    }
}
