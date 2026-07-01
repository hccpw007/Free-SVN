use crate::models::error::AppError;
use crate::models::file::FileItem;
use crate::models::repo::RepoInfo;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LogEntry {
    pub revision: u64,
    pub author: Option<String>,
    pub date: Option<String>,
    pub msg: Option<String>,
    pub paths: Option<Vec<LogPathEntry>>,
}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct LogPathEntry {
    pub action: String,
    #[serde(rename = "$value")]
    pub path: String,
}
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BlameLine {
    pub revision: u64,
    pub author: String,
    pub date: String,
    pub line_number: u64,
    pub content: String,
}

pub fn parse_status(xml: &str) -> Result<Vec<FileItem>, AppError> { todo!() }
pub fn parse_info(xml: &str) -> Result<RepoInfo, AppError> { todo!() }
pub fn parse_log(xml: &str) -> Result<Vec<LogEntry>, AppError> { todo!() }
pub fn parse_blame(xml: &str) -> Result<Vec<BlameLine>, AppError> { todo!() }
