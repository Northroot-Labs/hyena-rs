//! Injects checkpoint ID and build version from git and date (VERSIONING_STANDARD).
//! Source: repos/docs/internal/ci/VERSIONING_STANDARD.md

use std::process::Command;

fn main() {
    let short_sha = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
            } else {
                None
            }
        });

    let date = std::env::var("BUILD_DATE").ok().or_else(|| {
        Command::new("date")
            .args(["-u", "+%Y%m%d"])
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    });

    let (checkpoint_id, build_version) = match (short_sha, date) {
        (Some(sha), Some(d)) => {
            let display_date = if d.len() == 8 {
                format!("{}.{}.{}", &d[0..4], &d[4..6], &d[6..8])
            } else {
                d.clone()
            };
            (
                format!("cp-{}-{}", d, sha),
                format!("{}-{}", display_date, sha),
            )
        }
        _ => (
            "cp-0.0.0-dev".to_string(),
            "0.0.0-dev".to_string(),
        ),
    };

    println!("cargo:rustc-env=HYENA_CHECKPOINT_ID={}", checkpoint_id);
    println!("cargo:rustc-env=HYENA_BUILD_VERSION={}", build_version);
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-env-changed=BUILD_DATE");
}
