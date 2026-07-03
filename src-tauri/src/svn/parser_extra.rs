use crate::models::error::AppError;
use quick_xml::de::from_str;

/// 解析 `svn log --xml --verbose` 输出
pub fn parse_log(xml: &str) -> Result<Vec<LogEntry>, AppError> {
    #[derive(Debug, serde::Deserialize)]
    struct LogOutput {
        #[serde(rename = "logentry", default)]
        logentry: Vec<RawLogEntry>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct RawLogEntry {
        #[serde(rename = "@revision")]
        revision: u64,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        date: Option<String>,
        #[serde(default)]
        msg: Option<String>,
        #[serde(default)]
        paths: Option<RawPaths>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct RawPaths {
        #[serde(rename = "path", default)]
        path: Vec<RawPath>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct RawPath {
        #[serde(rename = "@action")]
        action: String,
        #[serde(rename = "$value")]
        value: String,
    }

    let output: LogOutput = from_str(xml)
        .map_err(|e| AppError::ParseFailed(format!("log XML 解析失败: {}", e)))?;

    Ok(output.logentry.into_iter().map(|entry| LogEntry {
        revision: entry.revision,
        author: entry.author,
        date: entry.date,
        msg: entry.msg,
        paths: entry.paths.map(|p| {
            p.path.into_iter().map(|rp| LogPathEntry {
                action: rp.action,
                path: rp.value,
            }).collect()
        }),
    }).collect())
}

/// 解析 `svn blame --xml` 输出
pub fn parse_blame(xml: &str) -> Result<Vec<BlameLine>, AppError> {
    #[derive(Debug, serde::Deserialize)]
    struct BlameOutput {
        target: BlameTarget,
    }
    #[derive(Debug, serde::Deserialize)]
    struct BlameTarget {
        #[serde(rename = "entry", default)]
        entry: Vec<BlameEntry>,
    }
    #[derive(Debug, serde::Deserialize)]
    struct BlameEntry {
        #[serde(rename = "@number")]
        number: u64,
        #[serde(default)]
        commit: BlameCommit,
        /// 捕获 `<entry number="N">行内容</entry>` 中的文本内容（即代码行原文）。
        ///
        /// 边界情况说明：
        /// 1. $value 在 quick_xml 中用于捕获元素的文本内容，但不同 svn 版本的
        ///    --xml 输出格式可能略有差异。部分 svn 版本（如 1.8.x 以下）在 entry
        ///    元素内可能没有直接的文本内容，导致 $value 解析为空字符串。
        /// 2. 空文件场景下，blame 输出可能没有任何 entry 元素（entry 列表为空），
        ///    此时 content 字段不会被触发解析。
        /// 3. 对于内容为空的代码行（如空行），svn blame 输出的 `<entry>` 文本内容
        ///    即为空字符串，属于正常行为。
        ///
        /// 处理策略：通过 #[serde(default)] 确保当 XML 中无可解析文本内容时，
        /// content 字段默认为空字符串 ""，避免反序列化失败。编码时不需要对空字符串
        /// 做特殊处理（空行本身是合法的 blame 结果）。
        #[serde(rename = "$value", default)]
        content: String,
    }
    #[derive(Debug, Default, serde::Deserialize)]
    struct BlameCommit {
        #[serde(rename = "@revision")]
        revision: u64,
        #[serde(default)]
        author: Option<String>,
        #[serde(default)]
        date: Option<String>,
    }

    let output: BlameOutput = from_str(xml)
        .map_err(|e| AppError::ParseFailed(format!("blame XML 解析失败: {}", e)))?;

    Ok(output.target.entry.into_iter().map(|entry| BlameLine {
        revision: entry.commit.revision,
        author: entry.commit.author.unwrap_or_default(),
        date: entry.commit.date.unwrap_or_default(),
        line_number: entry.number,
        content: entry.content,
    }).collect())
}

/// 解析 `svn diff` 输出（纯文本 unified diff）——原样返回
pub fn parse_diff(diff_text: &str) -> Result<String, AppError> {
    Ok(diff_text.to_string())
}

/// 解析 `svn mergeinfo` 输出（纯文本每行一个版本号）
pub fn parse_mergeinfo(xml: &str) -> Result<Vec<u64>, AppError> {
    Ok(xml.lines().filter_map(|line| line.trim().parse::<u64>().ok()).collect())
}

// ── 以下为骨架阶段已定义的结构体 ──

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub revision: u64,
    pub author: Option<String>,
    pub date: Option<String>,
    pub msg: Option<String>,
    pub paths: Option<Vec<LogPathEntry>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogPathEntry {
    pub action: String,
    #[serde(rename = "$value")]
    pub path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BlameLine {
    pub revision: u64,
    pub author: String,
    pub date: String,
    pub line_number: u64,
    pub content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_log ─────────────────────────────────

    #[test]
    fn test_parse_log_single() {
        let xml = r#"<?xml version="1.0"?>
<log>
<logentry revision="10">
<author>bob</author>
<date>2024-02-01T12:00:00.000000Z</date>
<msg>bug fix</msg>
</logentry>
</log>"#;
        let entries = parse_log(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].revision, 10);
        assert_eq!(entries[0].author.as_deref(), Some("bob"));
        assert_eq!(entries[0].msg.as_deref(), Some("bug fix"));
    }

    #[test]
    fn test_parse_log_multi_with_paths() {
        let xml = r#"<?xml version="1.0"?>
<log>
<logentry revision="20">
<author>alice</author>
<date>2024-03-01T00:00:00.000000Z</date>
<msg>refactored</msg>
<paths>
<path action="M">trunk/src/main.rs</path>
<path action="A">trunk/src/lib.rs</path>
</paths>
</logentry>
<logentry revision="19">
<author>bob</author>
<date>2024-02-28T00:00:00.000000Z</date>
<msg>wip</msg>
</logentry>
</log>"#;
        let entries = parse_log(xml).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].revision, 20);
        let paths = entries[0].paths.as_ref().unwrap();
        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].action, "M");
        assert_eq!(paths[0].path, "trunk/src/main.rs");
        assert_eq!(paths[1].path, "trunk/src/lib.rs");
    }

    #[test]
    fn test_parse_log_empty() {
        let xml = r#"<?xml version="1.0"?>
<log>
</log>"#;
        let entries = parse_log(xml).unwrap();
        assert!(entries.is_empty());
    }

    // ── parse_diff ─────────────────────────────────

    #[test]
    fn test_parse_diff_standard() {
        let diff = "--- a/file\n+++ b/file\n@@ -1 +1 @@\n-old\n+new\n";
        assert_eq!(parse_diff(diff).unwrap(), diff);
    }

    #[test]
    fn test_parse_diff_binary() {
        let diff = "Binary files /dev/null and b/file differ\n";
        assert_eq!(parse_diff(diff).unwrap(), diff);
    }

    // ── parse_blame ─────────────────────────────────

    #[test]
    fn test_parse_blame_standard() {
        let xml = r#"<?xml version="1.0"?>
<blame>
<target>
<entry number="1">
<commit revision="10"><author>alice</author><date>2024-01-01</date></commit>line1 content
</entry>
<entry number="2">
<commit revision="11"><author>bob</author><date>2024-01-02</date></commit>line2 content
</entry>
</target>
</blame>"#;
        let lines = parse_blame(xml).unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].revision, 10);
        assert_eq!(lines[0].author, "alice");
        assert_eq!(lines[0].line_number, 1);
        assert_eq!(lines[0].content.trim(), "line1 content");
        assert_eq!(lines[1].line_number, 2);
    }

    #[test]
    fn test_parse_blame_empty_file() {
        let xml = r#"<?xml version="1.0"?>
<blame>
<target>
</target>
</blame>"#;
        let lines = parse_blame(xml).unwrap();
        assert!(lines.is_empty());
    }

    // ── parse_mergeinfo ─────────────────────────────

    #[test]
    fn test_parse_mergeinfo_with_values() {
        let text = "10\n20\n30\n";
        let versions = parse_mergeinfo(text).unwrap();
        assert_eq!(versions, vec![10, 20, 30]);
    }

    #[test]
    fn test_parse_mergeinfo_empty() {
        let versions = parse_mergeinfo("").unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn test_parse_mergeinfo_ignores_invalid() {
        let text = "10\nnot_a_number\n20\n";
        let versions = parse_mergeinfo(text).unwrap();
        assert_eq!(versions, vec![10, 20]);
    }
}
