use serde::{Deserialize, Serialize};

/// 长操作进度（后端 → 前端，通过 Tauri event 推送）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationProgress {
    pub operation: String,
    pub percent: u8,
    pub stage: String,
    pub file_count: u32,
}

/// 长操作完成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub result: String,
    pub detail: Option<String>,
}

/// 认证凭据（v5 新增）
/// 通过 --password-from-stdin 安全传递，不进入命令行参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SvnCredentials {
    pub username: String,
    pub password: String,
    pub save_to_cache: bool,
}

/// 只读操作——可并发执行
pub const READONLY_OPERATIONS: &[&str] = &[
    "status", "info", "diff", "log", "blame", "check_network",
];

/// 写操作——互斥执行
pub const WRITE_OPERATIONS: &[&str] = &[
    "checkout", "commit", "update", "switch", "merge", "cleanup",
    "revert", "resolve", "add", "delete", "ignore", "lock", "unlock",
    "copy", "export", "relocate", "property",
];
