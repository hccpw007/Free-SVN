use serde::{Deserialize, Serialize};

/// 工作副本信息（对应 `svn info --xml` 输出结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfo {
    /// 文件系统路径
    pub path: String,
    /// 仓库远程 URL
    pub url: String,
    /// 相对于仓库根目录的 URL
    pub relative_url: Option<String>,
    /// 仓库根目录 URL
    pub root: String,
    /// 仓库 UUID
    pub repository_uuid: String,
    /// 当前修订版本号
    pub revision: u64,
    /// 节点类型（file/dir）
    pub node_kind: String,
    /// 最后变更的修订版本号
    pub last_changed_revision: u64,
    /// 最后变更的作者
    pub last_changed_author: String,
    /// 最后变更的日期
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
