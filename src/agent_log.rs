//! Agent log: append and read `.hyena/agent/agent_log.ndjson` (one JSON object per line).
//! Dedicated log for agent actions/findings; same shape as scratch for consistency.

use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const AGENT_LOG_REL: &str = ".hyena/agent/agent_log.ndjson";

/// Path to agent log file under repo root.
pub fn agent_log_path(root: &Path) -> std::path::PathBuf {
    root.join(AGENT_LOG_REL)
}

/// One line in agent_log.ndjson (same shape as scratch).
#[derive(Debug, Serialize)]
pub struct AgentLogEntry {
    pub ts: String,
    pub actor: String,
    pub kind: String,
    pub text: String,
}

/// Append one entry to agent_log.ndjson. Creates parent dirs if needed.
pub fn append_agent_log(root: &Path, actor: &str, kind: &str, text: &str) -> Result<()> {
    let path = agent_log_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let ts = Utc::now().to_rfc3339();
    let entry = AgentLogEntry {
        ts,
        actor: actor.to_string(),
        kind: kind.to_string(),
        text: text.to_string(),
    };
    let line = serde_json::to_string(&entry).context("serialize agent_log entry")?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open {}", path.display()))?;
    writeln!(f, "{}", line).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Read agent log lines, optionally limited to `max`. Returns concatenated output (each line is a JSON object).
pub fn read_agent_log(root: &Path, max: Option<usize>) -> Result<String> {
    let path = agent_log_path(root);
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
        let root = std::env::temp_dir().join("hyena_agent_log_roundtrip");
        fs::create_dir_all(&root).unwrap();
        let path = agent_log_path(&root);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir_all(root.join(".hyena"));

        append_agent_log(&root, "agent", "tool_result", "read_derived ok").unwrap();
        append_agent_log(&root, "agent", "finding", "summary: 3 themes").unwrap();

        let out = read_agent_log(&root, None).unwrap();
        assert!(out.contains("read_derived ok"));
        assert!(out.contains("summary: 3 themes"));
        assert!(out.contains("\"actor\":\"agent\""));
        assert!(out.contains("\"kind\":\"tool_result\""));

        let limited = read_agent_log(&root, Some(1)).unwrap();
        let line_count = limited.lines().filter(|s| !s.is_empty()).count();
        assert_eq!(line_count, 1);

        fs::remove_file(&path).ok();
        fs::remove_dir_all(root.join(".hyena")).ok();
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn read_agent_log_missing_returns_empty() {
        let root = std::env::temp_dir().join("hyena_agent_log_missing");
        fs::create_dir_all(&root).unwrap();
        let out = read_agent_log(&root, None).unwrap();
        assert!(out.is_empty());
        fs::remove_dir(&root).ok();
    }

    #[test]
    fn agent_log_entry_has_ts_and_kind() {
        let root = std::env::temp_dir().join("hyena_agent_log_ts");
        fs::create_dir_all(&root).unwrap();
        append_agent_log(&root, "agent", "thought", "x").unwrap();
        let out = read_agent_log(&root, Some(1)).unwrap();
        assert!(out.contains("\"ts\":"));
        assert!(out.contains("\"kind\":\"thought\""));
        fs::remove_file(agent_log_path(&root)).ok();
        fs::remove_dir_all(root.join(".hyena")).ok();
        fs::remove_dir_all(&root).ok();
    }
}
