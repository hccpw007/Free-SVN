use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct OperationProgress {
    pub operation: String,
    pub percent: u8,
    pub stage: String,
    pub file_count: u32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub result: String,
    pub detail: Option<String>,
}
