use serde::{Serialize, Deserialize};

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    pub token: String,
    pub owner: String,
    pub comment: Option<String>,
    pub created: String,
    pub expires: Option<String>,
}
