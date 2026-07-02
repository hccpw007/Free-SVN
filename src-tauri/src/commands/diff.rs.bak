use serde::Deserialize;
use crate::models::error::AppError;
use crate::svn;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DiffParams {
    pub path: String,
    pub revision1: Option<u64>,
    pub revision2: Option<u64>,
    /// 认证凭据（可选，用于访问远程仓库）
    pub credentials: Option<crate::svn::types::SvnCredentials>,
}

#[derive(Debug, serde::Serialize)]
pub struct DiffResult {
    pub content: String,
    pub is_binary: bool,
    pub mime_type: Option<String>,
}

/// 获取文件差异对比文本
#[tauri::command]
pub async fn get_diff(params: DiffParams) -> Result<DiffResult, AppError> {
    svn::executor::validate_path(&params.path)?;

    let path = Path::new(&params.path);
    if is_binary_file(path) {
        return Ok(DiffResult {
            content: String::new(),
            is_binary: true,
            mime_type: None,
        });
    }

    // 构造 svn diff 参数
    let mut args: Vec<String> = vec!["diff".to_string()];
    match (params.revision1, params.revision2) {
        (Some(r1), Some(r2)) => {
            args.push("-r".to_string());
            args.push(format!("{}:{}", r1, r2));
        }
        (Some(r1), None) => {
            args.push("-r".to_string());
            args.push(r1.to_string());
        }
        _ => {}
    }
    args.push(params.path.clone());

    let diff_text = svn::executor::run_svn(
        &args.iter().map(String::as_str).collect::<Vec<&str>>(),
        &params.path,
        params.credentials.as_ref(),
    )
    .await?;

    let content = svn::parser::parse_diff(&diff_text)?;
    Ok(DiffResult { content, is_binary: false, mime_type: None })
}

/// 通过扩展名检测二进制文件
fn is_binary_file(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "bmp" | "ico"
                | "pdf" | "doc" | "docx" | "xls" | "xlsx"
                | "ppt" | "pptx"
                | "zip" | "tar" | "gz" | "rar" | "7z"
                | "exe" | "dll" | "so" | "dylib"
                | "jar" | "class"
                | "mp3" | "mp4" | "avi" | "mov" | "wav" | "flac"
                | "ttf" | "otf" | "woff" | "woff2"
        ),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_is_binary_file_png() {
        assert!(is_binary_file(Path::new("image.png")));
    }

    #[test]
    fn test_is_binary_file_pdf() {
        assert!(is_binary_file(Path::new("doc.pdf")));
    }

    #[test]
    fn test_is_binary_file_txt() {
        assert!(!is_binary_file(Path::new("file.txt")));
    }

    #[test]
    fn test_is_binary_file_rs() {
        assert!(!is_binary_file(Path::new("main.rs")));
    }

    #[test]
    fn test_is_binary_file_no_extension() {
        assert!(!is_binary_file(Path::new("Makefile")));
    }

    #[test]
    fn test_diff_params_deserialize() {
        let json = r#"{"path": "/tmp/file.txt", "revision1": 10, "revision2": 20}"#;
        let params: DiffParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/file.txt");
        assert_eq!(params.revision1, Some(10));
        assert_eq!(params.revision2, Some(20));
    }

    #[test]
    fn test_diff_params_defaults() {
        let json = r#"{"path": "/tmp/file.txt"}"#;
        let params: DiffParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.revision1, None);
        assert_eq!(params.revision2, None);
    }
}
