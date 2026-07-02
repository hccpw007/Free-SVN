use serde::{Deserialize, Serialize};

/// 单个文件状态条目（对应 `svn status --xml` 输出中的每个 entry）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileItem {
    pub path: String,
    pub status: String,
    #[serde(default)]
    pub wc_status: Option<String>,
    #[serde(default)]
    pub commit_revision: Option<u64>,
    #[serde(default)]
    pub commit_author: Option<String>,
    #[serde(default)]
    pub commit_date: Option<String>,
    #[serde(default)]
    pub is_binary: bool,
    #[serde(default)]
    pub property_changes: Option<String>,
    #[serde(default)]
    pub moved_from: Option<String>,
    #[serde(default)]
    pub moved_to: Option<String>,
    #[serde(default)]
    pub copied_from: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub lock: Option<super::repo::LockInfo>,
}
