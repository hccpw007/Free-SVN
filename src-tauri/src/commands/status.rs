use serde::Deserialize;
use crate::models::error::AppError;
use crate::models::file::FileItem;
use crate::models::repo::RepoInfo;
use crate::svn;
use std::path::Path;

/// 获取状态参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusParams {
    pub path: String,
    #[serde(default)]
    pub ignore_patterns: Option<String>,
}

/// 获取信息参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoParams {
    pub path: String,
}

/// 获取文件变更列表（纯本地，不检测网络）
#[tauri::command]
pub async fn get_status(params: StatusParams) -> Result<Vec<FileItem>, AppError> {
    svn::executor::validate_path(&params.path)?;
    let xml = svn::executor::run_svn(&["status", "--xml", "--depth", "infinity"], &params.path, None).await?;
    let mut items = svn::parser::parse_status(&xml)?;
    let base = std::path::Path::new(&params.path);
    // svn status --xml 不包含文件大小，通过文件系统 stat 获取
    // 同时对标记为 unversioned 的目录递归展开内部文件
    let mut expanded = Vec::new();
    for item in items.iter_mut() {
        let full = base.join(&item.path);
        // 目录和已删除的文件无法 stat，保持 size=None
        if let Ok(meta) = std::fs::metadata(&full) {
            if meta.is_file() {
                item.size = Some(meta.len());
            }
        }
        if item.status == "unversioned" && full.is_dir() {
            expand_unversioned_dir(&full, base, &mut expanded);
        }
    }
    items.append(&mut expanded);
    // 应用全局忽略规则过滤
    if let Some(patterns) = &params.ignore_patterns {
        if !patterns.is_empty() {
            let ignore_list: Vec<&str> = patterns.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            if !ignore_list.is_empty() {
                items.retain(|item| {
                    let filename = Path::new(&item.path)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    !ignore_list.iter().any(|p| matches_glob(p, filename))
                });
            }
        }
    }
    Ok(items)
}

/// 简单的 glob 匹配（支持 * 匹配任意字符序列，? 匹配单个字符）
fn matches_glob(pattern: &str, s: &str) -> bool {
    if pattern.is_empty() {
        return s.is_empty();
    }
    let pat_bytes = pattern.as_bytes();
    let s_bytes = s.as_bytes();
    let (mut pi, mut si) = (0, 0);
    let (mut star_pi, mut star_si): (Option<usize>, Option<usize>) = (None, None);
    while si < s_bytes.len() {
        if pi < pat_bytes.len() && (pat_bytes[pi] == b'?' || pat_bytes[pi] == s_bytes[si]) {
            pi += 1;
            si += 1;
        } else if pi < pat_bytes.len() && pat_bytes[pi] == b'*' {
            star_pi = Some(pi);
            star_si = Some(si);
            pi += 1;
        } else if let (Some(sp), Some(ss)) = (star_pi, star_si) {
            pi = sp + 1;
            star_si = Some(ss + 1);
            si = ss + 1;
        } else {
            return false;
        }
    }
    while pi < pat_bytes.len() && pat_bytes[pi] == b'*' {
        pi += 1;
    }
    pi == pat_bytes.len()
}

/// 递归展开未版本控制的目录，将其内部所有文件添加为 unversioned 条目
fn expand_unversioned_dir(dir: &Path, base: &Path, result: &mut Vec<FileItem>) {
    use walkdir::WalkDir;
    for entry in WalkDir::new(dir).into_iter().filter_entry(|e| {
        // 跳过 .svn 目录
        e.file_name().to_str().map(|s| s != ".svn").unwrap_or(true)
    }) {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    let relative = path.strip_prefix(base).unwrap_or(path);
                    let size = std::fs::metadata(path).ok().map(|m| m.len());
                    result.push(FileItem {
                        path: relative.to_string_lossy().to_string(),
                        status: "unversioned".to_string(),
                        size,
                        ..Default::default()
                    });
                }
            }
            Err(e) => {
                log::warn!("expand_unversioned_dir: 无法访问 {:?}: {}", dir, e);
            }
        }
    }
}

/// 获取工作副本信息
#[tauri::command]
pub async fn get_info(params: InfoParams) -> Result<RepoInfo, AppError> {
    svn::executor::validate_path(&params.path)?;
    let xml = svn::executor::run_svn(&["info", "--xml"], &params.path, None).await?;
    svn::parser::parse_info(&xml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_params_deserialize_valid() {
        let json = r#"{"path": "/tmp/test"}"#;
        let params: StatusParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/test");
    }

    #[test]
    fn test_status_params_deserialize_empty() {
        let json = r#"{"path": ""}"#;
        let params: StatusParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "");
    }

    #[test]
    fn test_info_params_deserialize_valid() {
        let json = r#"{"path": "/tmp/test"}"#;
        let params: InfoParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "/tmp/test");
    }

    #[test]
    fn test_info_params_deserialize_empty() {
        let json = r#"{"path": ""}"#;
        let params: InfoParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.path, "");
    }
}
