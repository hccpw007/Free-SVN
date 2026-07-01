use serde::{Deserialize, Serialize};

/// 工作副本信息（对应 `svn info --xml` 输出结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    pub path: String,
    pub url: String,
    pub relative_url: Option<String>,
    pub root: String,
    pub repository_uuid: String,
    pub revision: u64,
    pub node_kind: String,
    pub last_changed_revision: u64,
    pub last_changed_author: String,
    pub last_changed_date: String,
    pub schedule: Option<String>,
    pub depth: Option<String>,
    pub checksum: Option<String>,
    pub lock: Option<LockInfo>,

    /// 当前工作副本所在分支名（从 URL 中提取，parser 层填充）
    #[serde(skip)]
    pub branch_name: Option<String>,
}

/// 文件锁定信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub token: String,
    pub owner: String,
    pub comment: Option<String>,
    pub created: String,
    #[serde(default)]
    pub expires: Option<String>,
}
