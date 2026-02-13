//! Read .notes/notes.ndjson with optional scope and max.

use anyhow::Result;
use std::path::Path;

const DERIVED_LOG: &str = ".notes/notes.ndjson";

/// Read derived log; filter by scope_contains (substring in line), limit to max lines.
pub fn read_derived(
    root: &Path,
    scope_contains: Option<&str>,
    max: Option<usize>,
) -> Result<Vec<String>> {
    let path = root.join(DERIVED_LOG);
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let content = std::fs::read_to_string(&path)?;
    let mut lines: Vec<String> = content
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    if let Some(scope) = scope_contains {
        lines.retain(|l| l.contains(scope));
    }
    if let Some(n) = max {
        lines.truncate(n);
    }
    Ok(lines)
}
