//! Integration tests: run hyena CLI in a temp dir and assert output.
//! Requires policy and raw files to be present under --root.

use std::path::PathBuf;
use std::process::Command;

fn hyena() -> Command {
    let root = project_root();
    let exe = std::env::var("CARGO_BIN_EXE_hyena")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Try release first (CI uses --release), then debug
            let release_path = root.join("target/release/hyena");
            if release_path.exists() {
                release_path
            } else {
                root.join("target/debug/hyena")
            }
        });
    let mut c = Command::new(&exe);
    c.current_dir(&root);
    c
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Temp dir inside project so spawned process can read it (e.g. under sandbox).
fn test_root(name: &str) -> PathBuf {
    let root = project_root().join("target").join("it").join(name);
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn write_scratch_then_read_scratch_roundtrip() {
    let root = test_root("scratch");
    let _guard = RemoveOnDrop(root.clone());

    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();

    let root_str = root.to_string_lossy().into_owned();
    let out = hyena()
        .args([
            "--root",
            &root_str,
            "write",
            "scratch",
            "integration test line",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = hyena()
        .args(["--root", &root_str, "read", "scratch"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "read scratch: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("integration test line"),
        "stdout: {:?}",
        stdout
    );
    assert!(stdout.contains("\"actor\":\"human\""));
}

#[test]
fn read_raw_finds_notes_md() {
    let root = test_root("raw");
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(
        root.join(".agent/POLICY.yaml"),
        r#"policy:
  name: hyena
filesystem:
  raw_inputs:
    patterns:
      - "**/NOTES.md"
"#,
    )
    .unwrap();
    std::fs::write(root.join("NOTES.md"), "root notes content\n").unwrap();
    std::fs::write(root.join("sub/NOTES.md"), "sub notes content\n").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    let root_str = root.to_string_lossy().into_owned();

    let out = hyena()
        .args(["--root", &root_str, "read", "raw"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root notes content"));
    assert!(stdout.contains("sub notes content"));
    assert!(stdout.contains("---"));
}

#[test]
fn search_derived_log() {
    let root = test_root("search");
    std::fs::create_dir_all(root.join(".notes")).unwrap();
    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
    std::fs::write(
        root.join(".notes/notes.ndjson"),
        r#"{"ts":"2025-01-01","text":"needle in hay"}
{"ts":"2025-01-02","text":"no match"}
"#,
    )
    .unwrap();
    let _guard = RemoveOnDrop(root.clone());
    let root_str = root.to_string_lossy().into_owned();

    let out = hyena()
        .args(["--root", &root_str, "search", "needle"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("needle in hay"));
    assert!(!stdout.contains("no match"));
}

#[test]
fn read_context_finds_nearest_notes() {
    let root = test_root("context");
    std::fs::create_dir_all(root.join("a/b")).unwrap();
    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
    std::fs::write(root.join("a/NOTES.md"), "nearest notes\n").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    let root_str = root.to_string_lossy().into_owned();
    let path_arg = root.join("a/b").to_string_lossy().into_owned();

    let out = hyena()
        .args(["--root", &root_str, "read", "context", "--path", &path_arg])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("NOTES.md"));
    assert!(stdout.contains("nearest notes"));
}

/// Guard that removes the directory when dropped (end of test).
struct RemoveOnDrop(std::path::PathBuf);
impl Drop for RemoveOnDrop {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
