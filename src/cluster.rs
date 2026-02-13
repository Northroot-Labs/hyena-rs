//! Light clustering: read .notes/notes.ndjson, group by word-overlap similarity, write .work/clusters/.

use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

const DERIVED_LOG: &str = ".notes/notes.ndjson";
const CLUSTERS_DIR: &str = ".work/clusters";

/// Minimum notes per cluster (per policy promotion.scrap_to_cluster.min_atoms).
const MIN_ATOMS: usize = 2;
/// Similarity threshold (per policy promotion.scrap_to_cluster.similarity_threshold; default 0.65).
const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.65;

#[derive(Debug, serde::Deserialize)]
struct NoteLine {
    text: Option<String>,
    source: Option<String>,
    #[serde(default)]
    provenance: Option<ProvenanceRef>,
}

#[derive(Debug, serde::Deserialize, Default)]
struct ProvenanceRef {
    source_file: Option<String>,
    line_start: Option<u32>,
    line_end: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ClusterNote {
    pub source_file: String,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct ClusterFile {
    pub notes: Vec<ClusterNote>,
}

/// Normalize markdown-ish text for tokenization: replace **, [], () with space so words are preserved.
fn normalize_for_tokens(text: &str) -> String {
    let mut out = text.to_string();
    for (from, to) in [
        ("**", " "),
        ("[", " "),
        ("]", " "),
        ("(", " "),
        (")", " "),
        (":", " "),
    ] {
        out = out.replace(from, to);
    }
    out
}

fn tokenize(text: &str) -> HashSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

/// Tokenize after normalizing markdown so "**Focus:**" and "Focus" yield the same words.
fn tokenize_normalized(text: &str) -> HashSet<String> {
    tokenize(&normalize_for_tokens(text))
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count();
    let union = a.union(b).count();
    if union == 0 {
        0.0
    } else {
        inter as f64 / union as f64
    }
}

/// Union-Find for grouping note indices.
struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
        }
    }
    fn find(&mut self, i: usize) -> usize {
        if self.parent[i] != i {
            self.parent[i] = self.find(self.parent[i]);
        }
        self.parent[i]
    }
    fn union(&mut self, i: usize, j: usize) {
        let pi = self.find(i);
        let pj = self.find(j);
        if pi != pj {
            self.parent[pi] = pj;
        }
    }
    fn groups(&mut self) -> HashMap<usize, Vec<usize>> {
        let n = self.parent.len();
        for i in 0..n {
            let _ = self.find(i);
        }
        let mut out: HashMap<usize, Vec<usize>> = HashMap::new();
        for i in 0..n {
            out.entry(self.find(i)).or_default().push(i);
        }
        out
    }
}

/// Run cluster: read derived log, group by similarity, write .work/clusters/cluster-<uuid>.yaml.
pub fn run_cluster(root: &Path, _policy_path: &Path) -> Result<usize> {
    let log_path = root.join(DERIVED_LOG);
    if !log_path.is_file() {
        return Ok(0);
    }

    let content = fs::read_to_string(&log_path)?;
    let mut notes: Vec<NoteLine> = Vec::new();
    let mut seen: HashSet<(String, u32, u32)> = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(n) = serde_json::from_str::<NoteLine>(trimmed) {
            let prov = n.provenance.as_ref();
            let source = prov
                .and_then(|p| p.source_file.clone())
                .or_else(|| n.source.clone())
                .unwrap_or_else(|| "?".to_string());
            let line_start = prov.and_then(|p| p.line_start).unwrap_or(0);
            let line_end = prov.and_then(|p| p.line_end).unwrap_or(0);
            let key = (source.clone(), line_start, line_end);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);
            notes.push(n);
        }
    }

    if notes.len() < MIN_ATOMS {
        return Ok(0);
    }

    // Build token sets per note and a reverse index from token -> note indices.
    // This optimization reduces comparisons from O(n²) to proportional to pairs sharing tokens.
    let mut word_sets: Vec<HashSet<String>> = Vec::with_capacity(notes.len());
    let mut token_index: HashMap<String, Vec<usize>> = HashMap::new();

    for (idx, n) in notes.iter().enumerate() {
        let tokens = tokenize_normalized(n.text.as_deref().unwrap_or(""));
        // Populate reverse index so we only compare notes that share at least one token.
        for token in tokens.iter() {
            token_index.entry(token.clone()).or_default().push(idx);
        }
        word_sets.push(tokens);
    }

    let mut uf = UnionFind::new(notes.len());
    let threshold = DEFAULT_SIMILARITY_THRESHOLD;

    // Track which pairs we've already compared, since notes can share multiple tokens.
    let mut compared_pairs: HashSet<(usize, usize)> = HashSet::new();

    for indices in token_index.values() {
        // For each token, consider all unique pairs of notes that contain it.
        for (pos_i, &i) in indices.iter().enumerate() {
            for &j in indices.iter().skip(pos_i + 1) {
                let pair = if i < j { (i, j) } else { (j, i) };
                if !compared_pairs.insert(pair) {
                    continue;
                }
                if jaccard(&word_sets[pair.0], &word_sets[pair.1]) >= threshold {
                    uf.union(pair.0, pair.1);
                }
            }
        }
    }

    let groups = uf.groups();
    let clusters_dir = root.join(CLUSTERS_DIR);
    fs::create_dir_all(&clusters_dir)
        .with_context(|| format!("create {}", clusters_dir.display()))?;

    let mut written = 0usize;
    for (_root_idx, indices) in groups {
        if indices.len() < MIN_ATOMS {
            continue;
        }
        let cluster_notes: Vec<ClusterNote> = indices
            .iter()
            .map(|&idx| {
                let n = &notes[idx];
                let prov = n.provenance.as_ref();
                ClusterNote {
                    source_file: prov
                        .and_then(|p| p.source_file.clone())
                        .or_else(|| n.source.clone())
                        .unwrap_or_else(|| "?".to_string()),
                    line_start: prov.and_then(|p| p.line_start).unwrap_or(0),
                    line_end: prov.and_then(|p| p.line_end).unwrap_or(0),
                    text: n.text.clone().unwrap_or_default(),
                }
            })
            .collect();

        let id = uuid_simple();
        let path = clusters_dir.join(format!("cluster-{}.yaml", id));
        let file = ClusterFile {
            notes: cluster_notes,
        };
        let yaml = serde_yaml::to_string(&file).context("serialize cluster")?;
        fs::write(&path, yaml).with_context(|| format!("write {}", path.display()))?;
        written += 1;
    }

    Ok(written)
}

fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // Use process ID and a counter to avoid collisions within the same nanosecond
    let pid = std::process::id();
    // Use a simple hash of the timestamp and PID for uniqueness (31 is a common prime multiplier)
    let hash = (t.wrapping_mul(31).wrapping_add(pid as u128)) % 0xffff_ffff_ffff_ffff;
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_and_jaccard() {
        let a = tokenize("alpha beta gamma");
        let b = tokenize("alpha beta delta");
        assert!(jaccard(&a, &b) > 0.3);
        assert!(jaccard(&a, &a) >= 0.99);
    }

    #[test]
    fn min_atoms_constant() {
        assert!(MIN_ATOMS >= 2);
    }

    #[test]
    fn normalize_strips_markdown() {
        let t = normalize_for_tokens("**Focus:** Make hyena useful.");
        assert!(t.contains("Focus"));
        assert!(t.contains("Make"));
        assert!(!t.contains('*'));
        let t2 = normalize_for_tokens("[docs](ORG_CONTEXT.md)");
        assert!(!t2.contains('['));
        assert!(t2.contains("docs"));
    }
}
