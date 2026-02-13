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
fn write_agent_log_then_read_agent_log_roundtrip() {
    let root = test_root("agent_log");
    let _guard = RemoveOnDrop(root.clone());

    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();

    let root_str = root.to_string_lossy().into_owned();
    let out = hyena()
        .args([
            "--root",
            &root_str,
            "--actor",
            "agent",
            "write",
            "agent-log",
            "tool result: read_derived ok",
            "--kind",
            "tool_result",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = hyena()
        .args(["--root", &root_str, "read", "agent-log"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "read agent-log: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("tool result: read_derived ok"),
        "stdout: {:?}",
        stdout
    );
    assert!(stdout.contains("\"actor\":\"agent\""));
    assert!(stdout.contains("\"kind\":\"tool_result\""));
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
fn ingest_then_read_derived_and_search() {
    let root = test_root("ingest");
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
    std::fs::write(
        root.join("NOTES.md"),
        "# Repo notes\n\n- alpha bullet\n- beta bullet\n\nSome paragraph.\n",
    )
    .unwrap();
    let _guard = RemoveOnDrop(root.clone());
    let root_str = root.to_string_lossy().into_owned();

    let out = hyena()
        .args(["--root", &root_str, "ingest"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ingest failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ingested") && stdout.contains("atoms"),
        "stdout: {:?}",
        stdout
    );

    let out = hyena()
        .args(["--root", &root_str, "read", "derived"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alpha bullet"));
    assert!(stdout.contains("beta bullet"));
    assert!(stdout.contains("Some paragraph"));
    assert!(stdout.contains("Repo notes"));

    let out = hyena()
        .args(["--root", &root_str, "search", "alpha"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("alpha"));

    let out = hyena()
        .args(["--root", &root_str, "cluster"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cluster failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("clusters"));

    // Dedupe is tested in ingest::tests::ingest_dedupes_second_run (same process).
    // Here we only assert second ingest runs successfully.
    let out2 = hyena()
        .args(["--root", &root_str, "ingest"])
        .output()
        .unwrap();
    assert!(out2.status.success());
}

#[test]
fn ingest_only_paths_delta() {
    let root = test_root("ingest_only");
    std::fs::create_dir_all(root.join("sub/dir")).unwrap();
    std::fs::create_dir_all(root.join(".agent")).unwrap();
    std::fs::write(root.join(".agent/POLICY.yaml"), "policy:\n  name: hyena\n").unwrap();
    std::fs::write(root.join("NOTES.md"), "# R\n\n- r1\n").unwrap();
    std::fs::write(root.join("sub/NOTES.md"), "# S\n\n- s1\n").unwrap();
    let _guard = RemoveOnDrop(root.clone());
    let root_str = root.to_string_lossy().into_owned();

    let out = hyena()
        .args(["--root", &root_str, "ingest", "--only", "sub/NOTES.md"])
        .output()
        .unwrap();
    assert!(out.status.success(), "ingest --only failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ingested"));
    let n: usize = stdout
        .split_whitespace()
        .find_map(|w| w.parse().ok())
        .unwrap_or(0);
    assert!(n >= 2, "delta ingest should yield at least 2 atoms from sub/NOTES.md");
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
