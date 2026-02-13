//! Load and validate .agent/POLICY.yaml (CLI-compatible subset).

#![allow(dead_code)] // fields used by serde deserialize; used as we add write/ingest

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

const POLICY_NAME: &str = "hyena";

#[derive(Debug, Deserialize)]
pub struct Policy {
    pub policy: PolicyMeta,
    #[serde(default)]
    pub actors: Option<Actors>,
    #[serde(default)]
    pub filesystem: Option<Filesystem>,
}

#[derive(Debug, Deserialize)]
pub struct PolicyMeta {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Actors {
    #[serde(default)]
    pub human: Option<ActorPerms>,
    #[serde(default)]
    pub agent: Option<ActorPerms>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ActorPerms {
    #[serde(rename = "can_write_raw_inputs", default)]
    pub can_write_raw_inputs: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct Filesystem {
    #[serde(default)]
    pub raw_inputs: Option<PathPerms>,
    #[serde(default)]
    pub agent_scratch: Option<PathPerms>,
    #[serde(default)]
    pub derived_logs: Option<PathPerms>,
}

#[derive(Debug, Default, Deserialize)]
pub struct PathPerms {
    #[serde(default)]
    pub patterns: Option<Vec<String>>,
    #[serde(default)]
    pub roots: Option<Vec<String>>,
    #[serde(default)]
    pub permissions: Option<serde_yaml::Value>,
}

/// Load policy from path and validate policy.name == "hyena".
pub fn load(path: &Path) -> Result<Policy> {
    let s = std::fs::read_to_string(path)
        .with_context(|| format!("read policy: {}", path.display()))?;
    let p: Policy = serde_yaml::from_str(&s).context("parse POLICY.yaml")?;
    if p.policy.name != POLICY_NAME {
        anyhow::bail!(
            "POLICY.yaml policy.name must be 'hyena', got '{}'",
            p.policy.name
        );
    }
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_hyena_name() {
        let yaml = r#"
policy:
  name: hyena
  version: "1.0"
actors:
  human:
    can_write_raw_inputs: true
  agent:
    can_write_raw_inputs: false
"#;
        let p: Policy = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(p.policy.name, "hyena");
        assert!(
            p.actors
                .as_ref()
                .unwrap()
                .human
                .as_ref()
                .unwrap()
                .can_write_raw_inputs
        );
    }

    #[test]
    fn load_rejects_non_hyena() {
        let yaml = "policy:\n  name: other\n";
        let p: Policy = serde_yaml::from_str(yaml).unwrap();
        assert_ne!(p.policy.name, POLICY_NAME);
        // load() does the check; we test load from temp file
        let dir = std::env::temp_dir().join("hyena_policy_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("POLICY.yaml");
        std::fs::write(&path, yaml).unwrap();
        let r = load(&path);
        std::fs::remove_file(&path).ok();
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("must be 'hyena'"));
    }

    #[test]
    fn load_accepts_hyena_file() {
        let yaml = "policy:\n  name: hyena\n";
        let dir = std::env::temp_dir().join("hyena_policy_accept");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("POLICY.yaml");
        std::fs::write(&path, yaml).unwrap();
        let p = load(&path).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!(p.policy.name, "hyena");
    }
}
