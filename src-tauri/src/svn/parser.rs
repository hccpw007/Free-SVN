use crate::models::error::AppError;
use crate::models::file::FileItem;
use crate::models::repo::RepoInfo;
use quick_xml::de::from_str;

/// 解析 `svn status --xml` 输出 → 文件变更列表
pub fn parse_status(xml: &str) -> Result<Vec<FileItem>, AppError> {
    #[derive(Debug, serde::Deserialize)]
    struct StatusOutput {
        #[serde(rename = "target")]
        target: Vec<StatusTarget>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct StatusTarget {
        #[serde(rename = "entry", default)]
        entry: Vec<StatusEntry>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct StatusEntry {
        #[serde(rename = "@path")]
        path: String,
        #[serde(rename = "wc-status")]
        wc_status: WcStatus,
        #[serde(rename = "commit", default)]
        commit: Option<CommitInfo>,
        #[serde(rename = "lock", default)]
        lock: Option<LockEntry>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct WcStatus {
        #[serde(rename = "@item")]
        item: String,
        #[serde(rename = "@props", default)]
        props: Option<String>,
        #[serde(rename = "@revision", default)]
        revision: Option<String>,
        #[serde(rename = "@wc-locked", default)]
        wc_locked: Option<bool>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct CommitInfo {
        #[serde(rename = "@revision")]
        revision: String,
        #[serde(rename = "author", default)]
        author: Option<String>,
        #[serde(rename = "date", default)]
        date: Option<String>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct LockEntry {
        token: String,
        owner: String,
        #[serde(default)]
        comment: Option<String>,
        created: String,
        #[serde(default)]
        expires: Option<String>,
    }

    let output: StatusOutput = from_str(xml)
        .map_err(|e| AppError::ParseFailed(format!("status XML 解析失败: {}", e)))?;

    let mut items = Vec::new();
    for target in output.target {
        for entry in target.entry {
            items.push(FileItem {
                path: entry.path,
                status: entry.wc_status.item,
                wc_status: entry.wc_status.props,
                commit_revision: entry.wc_status.revision
                    .and_then(|r| r.parse::<u64>().ok())
                    .or_else(|| entry.commit.as_ref().and_then(|c| c.revision.parse::<u64>().ok())),
                commit_author: entry.commit.as_ref().and_then(|c| c.author.clone()),
                commit_date: entry.commit.as_ref().and_then(|c| c.date.clone()),
                is_binary: false, // 由 diff command 检测后设置
                property_changes: None,
                moved_from: None,
                moved_to: None,
                copied_from: None,
                size: None,
                lock: entry.lock.map(|l| crate::models::repo::LockInfo {
                    token: l.token,
                    owner: l.owner,
                    comment: l.comment,
                    created: l.created,
                    expires: l.expires,
                }),
                wc_locked: entry.wc_status.wc_locked,
            });
        }
    }
    Ok(items)
}

/// 解析 `svn info --xml` 输出 → RepoInfo
pub fn parse_info(xml: &str) -> Result<RepoInfo, AppError> {
    #[derive(Debug, serde::Deserialize)]
    struct InfoOutput {
        #[serde(rename = "entry")]
        entry: InfoEntry,
    }
    #[derive(Debug, serde::Deserialize)]
    struct InfoEntry {
        #[serde(rename = "@path")]
        path: String,
        #[serde(rename = "@kind")]
        kind: String,
        #[serde(rename = "@revision")]
        revision: u64,
        url: String,
        #[serde(rename = "relative-url", default)]
        relative_url: Option<String>,
        #[serde(rename = "repository")]
        repository: Repository,
        #[serde(default)]
        wc_info: Option<WcInfo>,
        #[serde(default)]
        commit: Option<CommitAuthorInfo>,
        #[serde(default)]
        lock: Option<LockEntry>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct Repository {
        root: String,
        uuid: String,
    }
    #[derive(Debug, serde::Deserialize)]
    struct WcInfo {
        #[serde(default)]
        schedule: Option<String>,
        #[serde(default)]
        depth: Option<String>,
        #[serde(default)]
        checksum: Option<String>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct CommitAuthorInfo {
        #[serde(rename = "@revision")]
        revision: u64,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        date: Option<String>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct LockEntry {
        token: String,
        owner: String,
        #[serde(default)]
        comment: Option<String>,
        created: String,
        #[serde(default)]
        expires: Option<String>,
    }

    let output: InfoOutput = from_str(xml)
        .map_err(|e| AppError::ParseFailed(format!("info XML 解析失败: {}", e)))?;

    let entry = output.entry;
    let branch_name = extract_branch_name(&entry.url);

    Ok(RepoInfo {
        path: entry.path,
        url: entry.url.clone(),
        relative_url: entry.relative_url,
        root: entry.repository.root,
        repository_uuid: entry.repository.uuid,
        revision: entry.revision,
        node_kind: entry.kind,
        last_changed_revision: entry.commit.as_ref().map(|c| c.revision).unwrap_or(0),
        last_changed_author: entry.commit.as_ref().and_then(|c| c.author.clone()).unwrap_or_default(),
        last_changed_date: entry.commit.as_ref().and_then(|c| c.date.clone()).unwrap_or_default(),
        schedule: entry.wc_info.as_ref().and_then(|w| w.schedule.clone()),
        depth: entry.wc_info.as_ref().and_then(|w| w.depth.clone()),
        checksum: entry.wc_info.as_ref().and_then(|w| w.checksum.clone()),
        lock: entry.lock.map(|l| crate::models::repo::LockInfo {
            token: l.token,
            owner: l.owner,
            comment: l.comment,
            created: l.created,
            expires: l.expires,
        }),
        branch_name,
    })
}

/// 从 SVN URL 中提取分支名
fn extract_branch_name(url: &str) -> Option<String> {
    let parts: Vec<&str> = url.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if *part == "trunk" {
            return Some(part.to_string());
        }
        if (*part == "branches" || *part == "tags") && i + 1 < parts.len() {
            return Some(format!("{}/{}", part, parts[i + 1]));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_status ─────────────────────────────────

    #[test]
    fn test_parse_status_empty() {
        let xml = r#"<?xml version="1.0"?>
<status>
<target path=".">
</target>
</status>"#;
        let result = parse_status(xml).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_status_modified() {
        let xml = r#"<?xml version="1.0"?>
<status>
<target path=".">
<entry path="src/main.rs">
<wc-status item="modified" props="none" revision="42">
</wc-status>
<commit revision="42">
<author>testuser</author>
<date>2024-01-15T10:00:00.000000Z</date>
</commit>
</entry>
<entry path="src/lib.rs">
<wc-status item="added" props="none">
</wc-status>
</entry>
</target>
</status>"#;
        let items = parse_status(xml).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].path, "src/main.rs");
        assert_eq!(items[0].status, "modified");
        assert_eq!(items[0].commit_revision, Some(42));
        assert_eq!(items[0].commit_author.as_deref(), Some("testuser"));
        assert_eq!(items[1].path, "src/lib.rs");
        assert_eq!(items[1].status, "added");
    }

    // ── parse_info ─────────────────────────────────

    #[test]
    fn test_parse_info_complete() {
        let xml = r#"<?xml version="1.0"?>
<info>
<entry path="/repo/trunk" kind="dir" revision="100">
<url>https://svn.example.com/svn/repo/trunk</url>
<relative-url>^/trunk</relative-url>
<repository>
<root>https://svn.example.com/svn/repo</root>
<uuid>abc123-def456</uuid>
</repository>
<wc-info>
<schedule>normal</schedule>
<depth>infinity</depth>
<checksum>sha1</checksum>
</wc-info>
<commit revision="99">
<author>dev1</author>
<date>2024-01-10T08:00:00.000000Z</date>
</commit>
</entry>
</info>"#;
        let info = parse_info(xml).unwrap();
        assert_eq!(info.path, "/repo/trunk");
        assert_eq!(info.url, "https://svn.example.com/svn/repo/trunk");
        assert_eq!(info.root, "https://svn.example.com/svn/repo");
        assert_eq!(info.repository_uuid, "abc123-def456");
        assert_eq!(info.revision, 100);
        assert_eq!(info.branch_name.as_deref(), Some("trunk"));
    }

    #[test]
    fn test_parse_info_no_wc_info() {
        let xml = r#"<?xml version="1.0"?>
<info>
<entry path="/repo" kind="dir" revision="50">
<url>https://svn.example.com/svn/repo</url>
<repository>
<root>https://svn.example.com/svn/repo</root>
<uuid>abc-123</uuid>
</repository>
<commit revision="49">
<author>bot</author>
<date>2024-01-01T00:00:00.000000Z</date>
</commit>
</entry>
</info>"#;
        let info = parse_info(xml).unwrap();
        assert_eq!(info.relative_url, None);
        assert_eq!(info.schedule, None);
    }

    // ── extract_branch_name ──────────────────────────

    #[test]
    fn test_extract_branch_name_trunk() {
        assert_eq!(
            extract_branch_name("https://svn.example.com/svn/repo/trunk"),
            Some("trunk".to_string())
        );
    }

    #[test]
    fn test_extract_branch_name_branches() {
        assert_eq!(
            extract_branch_name("https://svn.example.com/svn/repo/branches/feature-1"),
            Some("branches/feature-1".to_string())
        );
    }

    #[test]
    fn test_extract_branch_name_tags() {
        assert_eq!(
            extract_branch_name("https://svn.example.com/svn/repo/tags/v1.0"),
            Some("tags/v1.0".to_string())
        );
    }

    #[test]
    fn test_extract_branch_name_bare_repo() {
        assert_eq!(extract_branch_name("https://svn.example.com/svn/repo"), None);
    }
}
