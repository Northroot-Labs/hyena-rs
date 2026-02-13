//! Raw inputs: discover files matching policy patterns and read their content.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Default patterns if policy has none (NOTES.md only).
pub const DEFAULT_RAW_PATTERNS: &[&str] = &["**/NOTES.md"];

/// Build a globset from pattern strings (e.g. "**/NOTES.md"). Uses forward slashes.
fn build_globset(patterns: &[String]) -> Result<globset::GlobSet> {
    let mut builder = globset::GlobSetBuilder::new();
    for p in patterns {
        builder.add(globset::Glob::new(p).with_context(|| format!("invalid pattern: {}", p))?);
    }
    builder.build().context("build glob set")
}

/// Path relative to root, normalized to forward slashes for glob matching.
fn relative_for_glob(path: &Path, root: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(|p| {
        p.components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/")
    })
}

/// Discover all files under `root` (optionally under `scope` dir) matching `patterns`.
/// Returns paths relative to root.
pub fn discover_raw_files(
    root: &Path,
    scope: Option<&PathBuf>,
    patterns: &[String],
) -> Result<Vec<PathBuf>> {
    let set = if patterns.is_empty() {
        build_globset(
            &DEFAULT_RAW_PATTERNS
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>(),
        )?
    } else {
        build_globset(patterns)?
    };

    let walk_root = scope
        .map(|s| root.join(s))
        .unwrap_or_else(|| root.to_path_buf());
    if !walk_root.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in WalkDir::new(&walk_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let abs = entry.path();
        let rel = match relative_for_glob(abs, root) {
            Some(r) => r,
            None => continue,
        };
        if set.is_match(&rel) {
            out.push(abs.to_path_buf());
        }
    }
    out.sort();
    Ok(out)
}

/// Read and return content of each path. Format: for each file, "path\n---\ncontent\n".
pub fn read_raw_content(paths: &[PathBuf]) -> Result<String> {
    let mut out = String::new();
    for p in paths {
        let content =
            std::fs::read_to_string(p).with_context(|| format!("read {}", p.display()))?;
        out.push_str(&p.display().to_string());
        out.push_str("\n---\n");
        out.push_str(&content);
        if !content.ends_with('\n') {
            out.push('\n');
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn discover_notes_md() {
        let root = std::env::temp_dir().join("hyena_raw_discover");
        fs::create_dir_all(root.join("a/b")).unwrap();
        fs::write(root.join("NOTES.md"), "r1").unwrap();
        fs::write(root.join("a/NOTES.md"), "r2").unwrap();
        fs::write(root.join("a/b/other.txt"), "x").unwrap();

        let patterns = vec!["**/NOTES.md".to_string()];
        let paths = discover_raw_files(&root, None, &patterns).unwrap();
        assert_eq!(paths.len(), 2);
        assert!(paths
            .iter()
            .any(|p| p.ends_with("NOTES.md") && p.parent().unwrap().ends_with("a")));
        assert!(paths
            .iter()
            .any(|p| p.ends_with("NOTES.md") && p.parent().unwrap() == root));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn discover_respects_scope() {
        let root = std::env::temp_dir().join("hyena_raw_scope");
        fs::create_dir_all(root.join("sub/dir")).unwrap();
        fs::write(root.join("NOTES.md"), "root").unwrap();
        fs::write(root.join("sub/NOTES.md"), "sub").unwrap();
        fs::write(root.join("sub/dir/NOTES.md"), "dir").unwrap();

        let patterns = vec!["**/NOTES.md".to_string()];
        let paths = discover_raw_files(&root, Some(&PathBuf::from("sub")), &patterns).unwrap();
        assert_eq!(paths.len(), 2); // sub and sub/dir
        assert!(paths.iter().all(|p| p.starts_with(&root.join("sub"))));

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn read_raw_content_formats_path_and_body() {
        let root = std::env::temp_dir().join("hyena_raw_content");
        fs::create_dir_all(&root).unwrap();
        let p = root.join("NOTES.md");
        fs::write(&p, "line1\nline2").unwrap();
        let paths = vec![p];
        let out = read_raw_content(&paths).unwrap();
        assert!(out.starts_with(&root.display().to_string()));
        assert!(out.contains("---"));
        assert!(out.contains("line1\nline2"));
        fs::remove_file(root.join("NOTES.md")).ok();
        fs::remove_dir(&root).ok();
    }

    #[test]
    fn default_patterns_when_empty() {
        let root = std::env::temp_dir().join("hyena_raw_default");
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("NOTES.md"), "x").unwrap();
        let paths = discover_raw_files(&root, None, &[]).unwrap();
        assert_eq!(paths.len(), 1);
        fs::remove_dir_all(&root).unwrap();
    }
}
