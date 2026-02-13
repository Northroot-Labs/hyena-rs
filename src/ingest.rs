//! Ingest: discover raw inputs, chunk markdown, append to .notes/notes.ndjson.
//! Contract: HYENA_RS_TASKS 4.x, hyena-policy-spec extraction.chunking.

use crate::{policy, raw};
use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

const DERIVED_LOG: &str = ".notes/notes.ndjson";

/// Provenance key for dedupe: same (source, line_start, line_end) = same atom.
fn provenance_key(source: &str, line_start: u32, line_end: u32) -> (String, u32, u32) {
    (source.to_string(), line_start, line_end)
}

fn semantic_key(source: &str, text: &str) -> (String, String) {
    let normalized = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    (source.to_string(), normalized)
}

/// Load existing dedupe keys from derived log.
fn load_existing_keys(
    derived_path: &Path,
    include_semantic: bool,
) -> Result<(HashSet<(String, u32, u32)>, HashSet<(String, String)>)> {
    let mut set = HashSet::new();
    let mut semantic_set = HashSet::new();
    let path = if derived_path.is_file() {
        derived_path
            .canonicalize()
            .unwrap_or_else(|_| derived_path.to_path_buf())
    } else {
        return Ok((set, semantic_set));
    };
    #[derive(Deserialize)]
    struct Line {
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        text: Option<String>,
        #[serde(default)]
        provenance: Option<Provenance>,
    }
    let f =
        std::fs::File::open(&path).with_context(|| format!("read existing {}", path.display()))?;
    for line in BufReader::new(f).lines() {
        let line = line.context("read line")?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(l) = serde_json::from_str::<Line>(trimmed) {
            if let Some(p) = l.provenance {
                set.insert(provenance_key(&p.source_file, p.line_start, p.line_end));
                if include_semantic {
                    let source = l.source.unwrap_or(p.source_file);
                    let text = l.text.unwrap_or_default();
                    semantic_set.insert(semantic_key(&source, &text));
                }
            } else if include_semantic {
                let source = l.source.unwrap_or_else(|| "?".to_string());
                let text = l.text.unwrap_or_default();
                semantic_set.insert(semantic_key(&source, &text));
            }
        }
    }
    Ok((set, semantic_set))
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Treat as markdown for chunking if path has .md or .markdown extension.
fn is_markdown_path(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
        .unwrap_or(false)
}

/// Chunk plain text / unknown format: one atom per non-empty line. Preserves provenance.
fn chunk_plain(content: &str) -> Vec<Chunk> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_num = (i + 1) as u32;
        out.push(Chunk {
            line_start: line_num,
            line_end: line_num,
            kind: "line",
            text: trimmed.to_string(),
        });
    }
    out
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

/// Normalize path to forward-slash relative form for comparison.
fn normalize_relative(path: &Path, root: &Path) -> String {
    let rel = path
        .strip_prefix(root)
        .map(|p| {
            p.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_else(|_| path_relative_to_root(path, root));
    rel
}

/// Run ingest: discover raw files, chunk each, append to .notes/notes.ndjson.
/// If only_paths is Some, only raw files whose path (relative to root) is in the set are processed (delta-aware).
pub fn run_ingest(
    root: &Path,
    policy_path: &Path,
    scope: Option<&PathBuf>,
    semantic_dedupe: bool,
    only_paths: Option<&[PathBuf]>,
) -> Result<usize> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
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

    let mut paths = raw::discover_raw_files(&root, scope, &patterns)?;
    if let Some(only) = only_paths {
        // Treat Some(&[]) the same as None: do not filter paths if the allow list is empty.
        if !only.is_empty() {
            let only_set: std::collections::HashSet<String> = only
                .iter()
                .map(|o| {
                    if o.is_absolute() {
                        normalize_relative(o, &root)
                    } else {
                        // Normalize relative paths by filtering out CurDir and handling ParentDir
                        o.components()
                            .filter_map(|c| match c {
                                std::path::Component::CurDir => None,
                                std::path::Component::ParentDir => Some("..".to_string()),
                                _ => Some(c.as_os_str().to_string_lossy().into_owned()),
                            })
                            .collect::<Vec<_>>()
                            .join("/")
                    }
                })
                .collect();
            paths.retain(|p| only_set.contains(&path_relative_to_root(p, &root)));
        }
    }
    let derived_path = root.join(DERIVED_LOG);
    if let Some(parent) = derived_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }

    let (mut existing, mut semantic_existing) = load_existing_keys(&derived_path, semantic_dedupe)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&derived_path)
        .with_context(|| format!("open {}", derived_path.display()))?;

    let mut count = 0usize;
    for path in &paths {
        let content =
            std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
        let source_rel = path_relative_to_root(path, &root);
        let scope_str: String = path
            .parent()
            .and_then(|p| p.strip_prefix(&root).ok())
            .map(|p| {
                let joined = p
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                if joined.is_empty() {
                    ".".to_string()
                } else {
                    joined
                }
            })
            .unwrap_or_else(|| ".".to_string());

        let chunks = if is_markdown_path(path) {
            chunk_markdown(&content)
        } else {
            chunk_plain(&content)
        };
        for chunk in chunks {
            if chunk.text.is_empty() {
                continue;
            }
            let key = provenance_key(&source_rel, chunk.line_start, chunk.line_end);
            if existing.contains(&key) {
                continue;
            }
            if semantic_dedupe {
                let s_key = semantic_key(&source_rel, &chunk.text);
                if semantic_existing.contains(&s_key) {
                    continue;
                }
                semantic_existing.insert(s_key);
            }
            existing.insert(key);

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

    #[test]
    fn ingest_dedupes_second_run() {
        let root = std::env::temp_dir().join("hyena_ingest_dedup");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".agent")).unwrap();
        std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
        std::fs::write(root.join("NOTES.md"), "# T\n\n- a\n- b\n").unwrap();
        let policy = root.join(".agent/POLICY.yaml");
        let n1 = run_ingest(&root, &policy, None, false, None).unwrap();
        assert!(n1 >= 3, "first ingest should write at least 3 atoms");
        let n2 = run_ingest(&root, &policy, None, false, None).unwrap();
        assert_eq!(n2, 0, "second ingest should append 0 (dedupe)");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ingest_semantic_dedupe_handles_line_shifts() {
        let root = std::env::temp_dir().join("hyena_ingest_semantic_dedup");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".agent")).unwrap();
        std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
        std::fs::write(root.join("NOTES.md"), "# T\n\n- keep this\n").unwrap();
        let policy = root.join(".agent/POLICY.yaml");

        let n1 = run_ingest(&root, &policy, None, true, None).unwrap();
        assert!(n1 >= 2);

        // Same semantic content shifted by one line.
        std::fs::write(root.join("NOTES.md"), "\n# T\n\n- keep this\n").unwrap();
        let n2 = run_ingest(&root, &policy, None, true, None).unwrap();
        assert_eq!(n2, 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ingest_delta_only_paths_processes_listed_files() {
        let root = std::env::temp_dir().join("hyena_ingest_delta");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".agent")).unwrap();
        std::fs::create_dir_all(root.join("a/b")).unwrap();
        std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
        std::fs::write(root.join("NOTES.md"), "# Root\n\n- r1\n").unwrap();
        std::fs::write(root.join("a/NOTES.md"), "# A\n\n- a1\n").unwrap();
        std::fs::write(root.join("a/b/NOTES.md"), "# B\n\n- b1\n").unwrap();
        let policy = root.join(".agent/POLICY.yaml");

        // Delta: only a/NOTES.md
        let only = vec![std::path::PathBuf::from("a/NOTES.md")];
        let n = run_ingest(&root, &policy, None, false, Some(&only)).unwrap();
        assert!(n >= 2, "a/NOTES.md should yield at least 2 atoms");

        // Full ingest would get root and a/b too; with only_paths we only got a/NOTES.md.
        let all = run_ingest(&root, &policy, None, false, None).unwrap();
        assert!(all >= n, "full ingest should add root and a/b atoms");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn ingest_delta_empty_only_paths_means_full_ingest() {
        let root = std::env::temp_dir().join("hyena_ingest_delta_full");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".agent")).unwrap();
        std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
        std::fs::write(root.join("NOTES.md"), "# T\n\n- x\n").unwrap();
        let policy = root.join(".agent/POLICY.yaml");
        
        // Empty only_paths should behave like a full ingest.
        let only: Vec<std::path::PathBuf> = Vec::new();
        let n = run_ingest(&root, &policy, None, false, Some(&only)).unwrap();
        assert!(n >= 2);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn chunk_plain_one_per_line() {
        let text = "first line\n\nsecond line\n  trimmed  \n";
        let chunks = chunk_plain(text);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].kind, "line");
        assert_eq!(chunks[0].text, "first line");
        assert_eq!(chunks[1].text, "second line");
        assert_eq!(chunks[2].text, "trimmed");
    }

    #[test]
    fn ingest_plain_txt_format_agnostic() {
        let root = std::env::temp_dir().join("hyena_ingest_plain");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".agent")).unwrap();
        std::fs::write(
            root.join(".agent/POLICY.yaml"),
            "policy:\n  name: hyena\nfilesystem:\n  raw_inputs:\n    patterns:\n      - '**/NOTES.md'\n      - '**/inbox/*.txt'\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("inbox")).unwrap();
        std::fs::write(root.join("inbox/scratch.txt"), "curious about downloads\nneed PR for branch X\n").unwrap();
        let policy = root.join(".agent/POLICY.yaml");
        let n = run_ingest(&root, &policy, None, false, None).unwrap();
        assert!(n >= 2, "plain .txt should yield one atom per non-empty line");
        let _ = std::fs::remove_dir_all(&root);
    }
}
