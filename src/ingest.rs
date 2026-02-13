//! Ingest: discover raw inputs, chunk markdown, append to .notes/notes.ndjson.
//! Contract: HYENA_RS_TASKS 4.x, hyena-policy-spec extraction.chunking.

use crate::{policy, raw};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

const DERIVED_LOG: &str = ".notes/notes.ndjson";

/// One atom emitted to notes.ndjson.
#[derive(Debug, Serialize)]
pub struct NoteEntry {
    pub ts: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    pub source: String,
    pub text: String,
    pub provenance: Provenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct Provenance {
    pub source_file: String,
    pub line_start: u32,
    pub line_end: u32,
}

/// Chunk from markdown: one atom (bullet, paragraph, heading, or code block).
#[derive(Debug)]
struct Chunk {
    line_start: u32,
    line_end: u32,
    kind: &'static str,
    text: String,
}

/// Chunk markdown per policy rules: top-level bullet, paragraph, heading, code block.
fn chunk_markdown(content: &str) -> Vec<Chunk> {
    let mut out = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    let n = lines.len();

    while i < n {
        let line = lines[i];
        let line_num = (i + 1) as u32;

        // Code fence: take until next ```
        if line.starts_with("```") {
            let start = line_num;
            let mut block = line.to_string();
            i += 1;
            while i < n && !lines[i].starts_with("```") {
                block.push('\n');
                block.push_str(lines[i]);
                i += 1;
            }
            if i < n {
                block.push('\n');
                block.push_str(lines[i]);
                i += 1;
            }
            out.push(Chunk {
                line_start: start,
                line_end: (i) as u32,
                kind: "code_block",
                text: block.trim().to_string(),
            });
            continue;
        }

        // Heading
        if let Some(rest) = line.strip_prefix('#') {
            let _level = 1 + rest.chars().take_while(|c| *c == '#').count();
            let rest = rest.trim_start_matches('#').trim();
            out.push(Chunk {
                line_start: line_num,
                line_end: line_num,
                kind: "heading",
                text: rest.to_string(),
            });
            i += 1;
            continue;
        }

        // Top-level bullet (- or *)
        let trimmed = line.trim_start();
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            out.push(Chunk {
                line_start: line_num,
                line_end: line_num,
                kind: "bullet",
                text: trimmed[2..].trim().to_string(),
            });
            i += 1;
            continue;
        }

        // Paragraph: run of non-empty lines until blank or special
        if !line.trim().is_empty() {
            let start = line_num;
            let mut para = Vec::new();
            para.push(line.trim());
            i += 1;
            while i < n {
                let l = lines[i];
                if l.trim().is_empty() {
                    i += 1;
                    break;
                }
                if l.starts_with("```")
                    || l.starts_with('#')
                    || l.trim_start().starts_with("- ")
                    || l.trim_start().starts_with("* ")
                {
                    break;
                }
                para.push(l.trim());
                i += 1;
            }
            let text = para.join(" ").trim().to_string();
            if !text.is_empty() {
                out.push(Chunk {
                    line_start: start,
                    line_end: start + para.len() as u32 - 1,
                    kind: "paragraph",
                    text,
                });
            }
            continue;
        }

        i += 1;
    }

    out
}

fn path_relative_to_root(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|_| path.display().to_string())
}

/// Run ingest: discover raw files, chunk each, append to .notes/notes.ndjson.
pub fn run_ingest(root: &Path, policy_path: &Path, scope: Option<&PathBuf>) -> Result<usize> {
    let policy = policy::load(policy_path)?;
    let patterns: Vec<String> = policy
        .filesystem
        .as_ref()
        .and_then(|fs| fs.raw_inputs.as_ref())
        .and_then(|ri| ri.patterns.as_ref())
        .cloned()
        .unwrap_or_else(|| {
            raw::DEFAULT_RAW_PATTERNS
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        });

    let paths = raw::discover_raw_files(root, scope, &patterns)?;
    let derived_path = root.join(DERIVED_LOG);
    if let Some(parent) = derived_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&derived_path)
        .with_context(|| format!("open {}", derived_path.display()))?;

    let mut count = 0usize;
    for path in &paths {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let source_rel = path_relative_to_root(path, root);
        let scope_str: String = path
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .unwrap_or_else(|| ".".to_string());

        for chunk in chunk_markdown(&content) {
            if chunk.text.is_empty() {
                continue;
            }
            let ts = Utc::now().to_rfc3339();
            let entry = NoteEntry {
                ts,
                kind: chunk.kind.to_string(),
                scope: Some(scope_str.clone()),
                source: source_rel.clone(),
                text: chunk.text.clone(),
                provenance: Provenance {
                    source_file: source_rel.clone(),
                    line_start: chunk.line_start,
                    line_end: chunk.line_end,
                },
                author: Some("human".to_string()),
                confidence: Some(0.5),
            };
            let line = serde_json::to_string(&entry).context("serialize note entry")?;
            writeln!(file, "{}", line)
                .with_context(|| format!("append {}", derived_path.display()))?;
            count += 1;
        }
    }

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_bullets_and_paragraph() {
        let md = r#"# Title

- first bullet
- second bullet

Some paragraph here.
More of it.

- another
"#;
        let chunks = chunk_markdown(md);
        assert!(chunks.len() >= 4);
        let kinds: Vec<&str> = chunks.iter().map(|c| c.kind).collect();
        assert!(kinds.contains(&"heading"));
        assert!(kinds.contains(&"bullet"));
        assert!(kinds.contains(&"paragraph"));
        let bullet_texts: Vec<&str> = chunks
            .iter()
            .filter(|c| c.kind == "bullet")
            .map(|c| c.text.as_str())
            .collect();
        assert!(bullet_texts.contains(&"first bullet"));
        assert!(bullet_texts.contains(&"second bullet"));
    }

    #[test]
    fn chunk_code_block() {
        let md = r#"Before
```rust
fn main() {}
```
After
"#;
        let chunks = chunk_markdown(md);
        let code: Vec<_> = chunks.iter().filter(|c| c.kind == "code_block").collect();
        assert_eq!(code.len(), 1);
        assert!(code[0].text.contains("fn main()"));
    }
}
