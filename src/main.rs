//! Hyena CLI: policy-enforcing, file-first agent substrate.
//! Contract: repos/docs/internal/agent/HYENA_CLI_SPEC.md

mod context;
mod policy;
mod raw;
mod scratch;
mod search;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "hyena",
    about = "Hyena: policy-enforced, file-first agent substrate"
)]
struct Cli {
    #[arg(long, default_value = ".")]
    root: std::path::PathBuf,

    #[arg(long)]
    policy: Option<std::path::PathBuf>,

    #[arg(long, default_value = "human", value_parser = ["human", "agent"])]
    actor: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read: context, raw, derived, scratch
    Read {
        #[command(subcommand)]
        what: ReadKind,
    },
    /// Write: scratch, derived (policy-checked)
    Write {
        #[command(subcommand)]
        what: WriteKind,
    },
    /// Walk NOTES.md, chunk, append events to .notes/notes.ndjson
    Ingest,
    /// Grep/scan .notes/notes.ndjson (and optionally scratch)
    Search {
        query: String,
        #[arg(long)]
        include_scratch: bool,
    },
    /// Human-only: append bullet to nearest NOTES.md
    Human {
        #[command(subcommand)]
        sub: HumanSub,
    },
}

#[derive(Subcommand)]
enum ReadKind {
    Context {
        #[arg(long)]
        path: Option<std::path::PathBuf>,
        #[arg(long)]
        max_lines: Option<usize>,
    },
    Raw {
        #[arg(long)]
        scope: Option<std::path::PathBuf>,
    },
    Derived {
        #[arg(long)]
        scope_contains: Option<String>,
        #[arg(long)]
        max: Option<usize>,
    },
    Scratch {
        #[arg(long)]
        max: Option<usize>,
    },
}

#[derive(Subcommand)]
enum WriteKind {
    Scratch {
        text: String,
        #[arg(long)]
        kind: Option<String>,
    },
    Derived {
        text: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        scope: Option<std::path::PathBuf>,
        #[arg(long)]
        source: Option<std::path::PathBuf>,
    },
}

#[derive(Subcommand)]
enum HumanSub {
    AppendRaw {
        text: String,
        #[arg(long)]
        path: Option<std::path::PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let policy_path = cli
        .policy
        .unwrap_or_else(|| cli.root.join(".agent/POLICY.yaml"));

    match &cli.command {
        Commands::Read { what } => match what {
            ReadKind::Context { path, max_lines } => {
                cmd_read_context(&cli.root, &policy_path, path.as_ref(), *max_lines)?
            }
            ReadKind::Raw { scope } => cmd_read_raw(&cli.root, &policy_path, scope.as_ref())?,
            ReadKind::Derived { .. } => println!("read derived (stub)"),
            ReadKind::Scratch { max } => cmd_read_scratch(&cli.root, *max)?,
        },
        Commands::Write { what } => match what {
            WriteKind::Scratch { text, kind } => {
                cmd_write_scratch(&cli.root, &cli.actor, text, kind.as_deref())?
            }
            WriteKind::Derived { .. } => println!("write derived (stub)"),
        },
        Commands::Ingest => println!("ingest (stub)"),
        Commands::Search {
            query,
            include_scratch,
        } => cmd_search(&cli.root, query, *include_scratch)?,
        Commands::Human { sub } => match sub {
            HumanSub::AppendRaw { .. } => {
                if cli.actor != "human" {
                    anyhow::bail!("human append-raw requires --actor human");
                }
                println!("human append-raw (stub)");
            }
        },
    }
    Ok(())
}

fn cmd_read_context(
    root: &std::path::Path,
    policy_path: &std::path::Path,
    path: Option<&PathBuf>,
    max_lines: Option<usize>,
) -> Result<()> {
    let _policy = policy::load(policy_path)?;
    let (_dir, notes_path) = context::nearest_notes_dir(root, path.cloned())
        .ok_or_else(|| anyhow::anyhow!("no NOTES.md found from path (walk up to root)"))?;
    let excerpt = context::read_notes_excerpt(&notes_path, max_lines)?;
    println!("{}", notes_path.display());
    println!("---");
    print!("{}", excerpt);
    if excerpt.ends_with('\n') {
        // already newline
    } else if !excerpt.is_empty() {
        println!();
    }
    Ok(())
}

fn cmd_read_raw(
    root: &std::path::Path,
    policy_path: &std::path::Path,
    scope: Option<&PathBuf>,
) -> Result<()> {
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
    let out = raw::read_raw_content(&paths)?;
    print!("{}", out);
    Ok(())
}

fn cmd_read_scratch(root: &std::path::Path, max: Option<usize>) -> Result<()> {
    let out = scratch::read_scratch(root, max)?;
    print!("{}", out);
    Ok(())
}

fn cmd_write_scratch(
    root: &std::path::Path,
    actor: &str,
    text: &str,
    kind: Option<&str>,
) -> Result<()> {
    scratch::append_scratch(root, actor, kind.unwrap_or("note"), text)
}

fn cmd_search(root: &std::path::Path, query: &str, include_scratch: bool) -> Result<()> {
    let lines = search::search(root, query, include_scratch)?;
    for line in &lines {
        println!("{}", line);
    }
    Ok(())
}
