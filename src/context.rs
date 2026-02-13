//! Nearest-notes resolution: walk up from path to find NOTES.md and return path + content.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const NOTES_MD: &str = "NOTES.md";

/// Resolve path to start from: root.join(path) if path is relative, else path.
fn start_path(root: &Path, from: Option<PathBuf>) -> PathBuf {
    match from {
        None => root.to_path_buf(),
        Some(p) if p.is_absolute() => p,
        Some(p) => root.join(p),
    }
}

/// Returns true if `current` is at or under `root` (repo boundary).
fn under_root(current: &Path, root: &Path) -> bool {
    current == root || current.starts_with(root)
}

/// Find nearest NOTES.md by walking up from `from` until repo root. Returns directory containing NOTES.md and its path.
pub fn nearest_notes_dir(root: &Path, from: Option<PathBuf>) -> Option<(PathBuf, PathBuf)> {
    let start = start_path(root, from);
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start
    };
    loop {
        if !under_root(&current, root) {
            break;
        }
        let notes = current.join(NOTES_MD);
        if notes.is_file() {
            return Some((current, notes));
        }
        let parent = current.parent()?.to_path_buf();
        if !under_root(&parent, root) {
            break;
        }
        current = parent;
    }
    None
}

/// Read NOTES.md content with optional line limit (excerpt).
pub fn read_notes_excerpt(notes_path: &Path, max_lines: Option<usize>) -> Result<String> {
    let s = std::fs::read_to_string(notes_path)
        .with_context(|| format!("read {}", notes_path.display()))?;
    let out = match max_lines {
        None => s,
        Some(n) => s.lines().take(n).collect::<Vec<_>>().join("\n"),
    };
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn nearest_notes_finds_in_same_dir() {
        let dir = std::env::temp_dir().join("hyena_ctx_same");
        fs::create_dir_all(&dir).unwrap();
        let notes = dir.join(NOTES_MD);
        fs::write(&notes, "hello").unwrap();
        let (dir_found, path_found) = nearest_notes_dir(&dir, Some(dir.clone())).unwrap();
        assert_eq!(dir_found, dir);
        assert_eq!(path_found, notes);
        fs::remove_file(&notes).ok();
        fs::remove_dir(&dir).ok();
    }

    #[test]
    fn nearest_notes_finds_parent() {
        let root = std::env::temp_dir().join("hyena_ctx_root");
        let sub = root.join("a").join("b");
        fs::create_dir_all(&sub).unwrap();
        let root_notes = root.join(NOTES_MD);
        fs::write(&root_notes, "root").unwrap();
        let (_dir, path) = nearest_notes_dir(&root, Some(sub)).unwrap();
        assert_eq!(path, root_notes);
        fs::remove_file(&root_notes).ok();
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn nearest_notes_none_when_missing() {
        let dir = std::env::temp_dir().join("hyena_ctx_none");
        fs::create_dir_all(&dir).unwrap();
        let r = nearest_notes_dir(&dir, Some(dir.clone()));
        fs::remove_dir(&dir).ok();
        assert!(r.is_none());
    }

    #[test]
    fn read_notes_excerpt_limits_lines() {
        let dir = std::env::temp_dir().join("hyena_excerpt");
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("NOTES.md");
        fs::write(&p, "a\nb\nc\nd\n").unwrap();
        let s = read_notes_excerpt(&p, Some(2)).unwrap();
        assert_eq!(s, "a\nb");
        fs::remove_file(&p).ok();
        fs::remove_dir(&dir).ok();
    }
}
