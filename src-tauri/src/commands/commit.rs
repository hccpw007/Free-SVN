use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use crate::svn::types::OperationResult;

/// 提交参数（v5 新增 credentials）
#[derive(Debug, Deserialize)]
pub struct CommitParams {
    pub paths: Vec<String>,
    pub message: String,
    pub keep_locks: Option<bool>,
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

/// 提交变更（原子操作，不可取消）
#[tauri::command]
pub async fn create_commit(
    params: CommitParams,
    state: tauri::State<'_, crate::svn::queue::SvnQueue>,
) -> Result<OperationResult, AppError> {
    if params.paths.is_empty() {
        return Err(AppError::InvalidInput("至少选择一个文件".into()));
    }
    svn::executor::validate_path(&params.paths[0])?;
    state.try_lock()?;

    let mut args = vec!["commit".to_string(), "--xml".to_string()];
    args.push("-m".to_string()); args.push(params.message.clone());
    if params.keep_locks.unwrap_or(false) {
        args.push("--no-unlock".to_string());
    }
    args.extend(params.paths.clone());

    let result = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &get_working_copy_root(&params.paths[0]),
        params.credentials.as_ref(),
    ).await;

    state.unlock();

    match result {
        Ok(xml) => {
            let rev = extract_commit_revision(&xml);
            Ok(OperationResult {
                result: "success".to_string(),
                detail: Some(format!("Committed revision {}", rev)),
            })
        }
        Err(e) => Err(e),
    }
}

/// 获取工作副本根目录（从 paths[0] 逐级向上查找 .svn 目录）
/// 避免直接使用 paths[0] 作为 cwd 导致子目录下执行 svn commit 失败
fn get_working_copy_root(path: &str) -> String {
    let mut p = std::path::PathBuf::from(path);
    loop {
        if p.join(".svn").exists() {
            return p.to_string_lossy().to_string();
        }
        if !p.pop() {
            return path.to_string();
        }
    }
}

fn extract_commit_revision(xml: &str) -> u64 {
    xml.lines()
        .find(|l| l.contains("revision=\""))
        .and_then(|l| l.split("revision=\"").nth(1))
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commit_params_deserialize() {
        let json = r#"{
            "paths": ["/tmp/repo/file.txt"],
            "message": "bug fix",
            "keep_locks": false
        }"#;
        let params: CommitParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.paths.len(), 1);
        assert_eq!(params.message, "bug fix");
        assert_eq!(params.keep_locks, Some(false));
    }

    #[test]
    fn test_extract_commit_revision_from_xml() {
        let xml = r#"<?xml version="1.0"?>
<commit revision="123">
</commit>"#;
        assert_eq!(extract_commit_revision(xml), 123);
    }

    #[test]
    fn test_extract_commit_revision_no_match() {
        assert_eq!(extract_commit_revision("<foo></foo>"), 0);
    }
}
