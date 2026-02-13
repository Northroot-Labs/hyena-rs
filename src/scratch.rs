//! Scratch log: append and read `.hyena/agent/scratch.ndjson` (one JSON object per line).

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const SCRATCH_REL: &str = ".hyena/agent/scratch.ndjson";

/// Path to scratch file under repo root.
pub fn scratch_path(root: &Path) -> std::path::PathBuf {
    root.join(SCRATCH_REL)
}

/// One line in scratch.ndjson.
#[derive(Debug, Serialize)]
pub struct ScratchEntry {
    pub ts: String,
    pub actor: String,
    pub kind: String,
    pub text: String,
}

/// Append one entry to scratch.ndjson. Creates parent dirs if needed.
pub fn append_scratch(root: &Path, actor: &str, kind: &str, text: &str) -> Result<()> {
    let path = scratch_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let ts = Utc::now().to_rfc3339();
    let entry = ScratchEntry {
        ts,
        actor: actor.to_string(),
        kind: kind.to_string(),
        text: text.to_string(),
    };
    let line = serde_json::to_string(&entry).context("serialize scratch entry")?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(f, "{}", line).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Read scratch lines, optionally limited to `max`. Returns concatenated output (each line is a JSON object).
pub fn read_scratch(root: &Path, max: Option<usize>) -> Result<String> {
    let path = scratch_path(root);
    if !path.is_file() {
        return Ok(String::new());
    }
    let f = std::fs::File::open(&path).with_context(|| format!("read {}", path.display()))?;
    let reader = BufReader::new(f);
    let mut lines: Vec<String> = reader
        .lines()
        .map_while(Result::ok)
        .filter(|s| !s.trim().is_empty())
        .collect();
    if let Some(n) = max {
        lines.truncate(n);
    }
    Ok(lines.join("\n") + if lines.is_empty() { "" } else { "\n" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn append_and_read_roundtrip() {
        let root = std::env::temp_dir().join("hyena_scratch_roundtrip");
        fs::create_dir_all(&root).unwrap();
        let path = scratch_path(&root);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(root.join(".hyena"));

        append_scratch(&root, "human", "note", "hello world").unwrap();
        append_scratch(&root, "agent", "thought", "second line").unwrap();

        let out = read_scratch(&root, None).unwrap();
        assert!(out.contains("hello world"));
        assert!(out.contains("second line"));
        assert!(out.contains("\"actor\":\"human\""));
        assert!(out.contains("\"actor\":\"agent\""));

        let limited = read_scratch(&root, Some(1)).unwrap();
        let line_count = limited.lines().filter(|s| !s.is_empty()).count();
        assert_eq!(line_count, 1);

        fs::remove_file(&path).ok();
        fs::remove_dir_all(root.join(".hyena")).ok();
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_scratch_missing_returns_empty() {
        let root = std::env::temp_dir().join("hyena_scratch_missing");
        fs::create_dir_all(&root).unwrap();
        let out = read_scratch(&root, None).unwrap();
        assert!(out.is_empty());
        fs::remove_dir(&root).ok();
    }

    #[test]
    fn scratch_entry_has_ts_and_kind() {
        let root = std::env::temp_dir().join("hyena_scratch_ts");
        fs::create_dir_all(&root).unwrap();
        append_scratch(&root, "agent", "thought", "x").unwrap();
        let out = read_scratch(&root, Some(1)).unwrap();
        assert!(out.contains("\"ts\":"));
        assert!(out.contains("\"kind\":\"thought\""));
        fs::remove_file(scratch_path(&root)).ok();
        fs::remove_dir_all(root.join(".hyena")).ok();
        fs::remove_dir_all(&root).ok();
    }
}
