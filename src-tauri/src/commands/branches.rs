use crate::models::error::AppError;

/// 列出 SVN 仓库分支列表（svn ls ^/branches）
#[tauri::command]
pub async fn list_branches(url: String) -> Result<Vec<String>, AppError> {
    if url.is_empty() {
        return Ok(vec![]);
    }
    // 从 URL 推断分支根
    let branch_root = if url.contains("/branches/") {
        // 工作副本已位于分支下，定位到 branches/ 父目录
        url.rsplit_once("/branches/").map(|(base, _)| format!("{}/branches", base))
            .unwrap_or_else(|| format!("{}/branches", url.trim_end_matches('/')))
    } else {
        format!("{}/branches", url.trim_end_matches('/'))
    };
    let result = crate::svn::executor::run_svn(
        &["list", &branch_root],
        std::env::temp_dir().to_str().unwrap_or("."),
        None,
    ).await?;
    Ok(result.lines().filter(|l| l.ends_with('/')).map(|l| l.trim_end_matches('/').to_string()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_branches_empty_url() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(list_branches("".to_string())).unwrap();
        assert!(result.is_empty());
    }
}
