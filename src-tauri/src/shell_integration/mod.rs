//! 操作系统 Shell 集成——注册/卸载右键菜单（macOS / Windows / Linux）。

pub mod macos;
pub mod windows;
pub mod linux;

use crate::models::error::AppError;

/// 注册三个平台的右键菜单
pub fn register() -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    macos::register()?;
    #[cfg(target_os = "windows")]
    windows::register()?;
    #[cfg(target_os = "linux")]
    linux::register()?;
    Ok(())
}

/// 卸载三个平台的右键菜单
pub fn unregister() -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    macos::unregister()?;
    #[cfg(target_os = "windows")]
    windows::unregister()?;
    #[cfg(target_os = "linux")]
    linux::unregister()?;
    Ok(())
}
