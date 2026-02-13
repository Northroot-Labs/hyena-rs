//! Line-scan search over .notes/notes.ndjson and optionally .hyena/agent/scratch.ndjson.

use anyhow::Result;
use std::path::Path;

const DERIVED_LOG: &str = ".notes/notes.ndjson";

fn scan_file(path: &Path, query: &str, out: &mut Vec<String>) -> Result<()> {
    if !path.is_file() {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains(query) {
            out.push(line.to_string());
        }
    }
    Ok(())
}

/// Search derived log (and optionally scratch) for lines containing `query`. Returns matching lines.
pub fn search(root: &Path, query: &str, include_scratch: bool) -> Result<Vec<String>> {
    let mut out = Vec::new();
    let derived = root.join(DERIVED_LOG);
    scan_file(&derived, query, &mut out)?;
    if include_scratch {
        let scratch = root.join(".hyena/agent/scratch.ndjson");
        scan_file(&scratch, query, &mut out)?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn search_matches_line() {
        let root = std::env::temp_dir().join("hyena_search_match");
        fs::create_dir_all(root.join(".notes")).unwrap();
        let log = root.join(".notes/notes.ndjson");
        fs::write(
            &log,
            r#"{"ts":"2025-01-01","text":"foo bar"}
{"ts":"2025-01-02","text":"baz qux"}
"#,
        )
        .unwrap();
        let hits = search(&root, "foo", false).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("foo bar"));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn search_include_scratch() {
        let root = std::env::temp_dir().join("hyena_search_scratch");
        fs::create_dir_all(root.join(".notes")).unwrap();
        fs::create_dir_all(root.join(".hyena/agent")).unwrap();
        fs::write(
            root.join(".notes/notes.ndjson"),
            r#"{"text":"only in derived"}
"#,
        )
        .unwrap();
        fs::write(
            root.join(".hyena/agent/scratch.ndjson"),
            r#"{"text":"only in scratch","query":"needle"}
"#,
        )
        .unwrap();
        let hits = search(&root, "needle", true).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].contains("needle"));
        let hits_no_scratch = search(&root, "needle", false).unwrap();
        assert_eq!(hits_no_scratch.len(), 0);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn search_missing_files_ok() {
        let root = std::env::temp_dir().join("hyena_search_missing");
        fs::create_dir_all(&root).unwrap();
        let hits = search(&root, "x", false).unwrap();
        assert!(hits.is_empty());
        fs::remove_dir(&root).ok();
    }
}
