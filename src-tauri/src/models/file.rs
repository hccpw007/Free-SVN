use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub path: String,
    pub status: String,
    pub wc_status: Option<String>,
    pub commit_revision: Option<u64>,
    pub commit_author: Option<String>,
    pub commit_date: Option<String>,
    pub is_binary: bool,
    pub property_changes: Option<String>,
    pub moved_from: Option<String>,
    pub moved_to: Option<String>,
    pub copied_from: Option<String>,
    pub size: Option<u64>,
    pub lock: Option<super::repo::LockInfo>,
}
