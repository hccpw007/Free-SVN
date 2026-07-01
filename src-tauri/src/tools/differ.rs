use crate::models::error::AppError;

pub struct ExternalDiffParams {
    pub file1: String,
    pub file2: String,
}

pub fn open_external_diff(tool: &str, params: ExternalDiffParams) -> Result<(), AppError> { todo!() }
