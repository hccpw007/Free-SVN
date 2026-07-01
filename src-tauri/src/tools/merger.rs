use crate::models::error::AppError;

pub struct ExternalMergeParams {
    pub mine: String,
    pub base: String,
    pub theirs: String,
    pub output: String,
}

pub fn open_external_merge(tool: &str, params: ExternalMergeParams) -> Result<(), AppError> { todo!() }
