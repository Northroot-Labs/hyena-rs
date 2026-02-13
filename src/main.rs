//! Hyena CLI: policy-enforcing, file-first agent substrate.
//! Contract: repos/docs/internal/agent/HYENA_CLI_SPEC.md

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "hyena", about = "Hyena: policy-enforced, file-first agent substrate")]
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
    let _policy_path = cli
        .policy
        .unwrap_or_else(|| cli.root.join(".agent/POLICY.yaml"));

    match &cli.command {
        Commands::Read { what } => match what {
            ReadKind::Context { .. } => println!("read context (stub)"),
            ReadKind::Raw { .. } => println!("read raw (stub)"),
            ReadKind::Derived { .. } => println!("read derived (stub)"),
            ReadKind::Scratch { .. } => println!("read scratch (stub)"),
        },
        Commands::Write { what } => match what {
            WriteKind::Scratch { .. } => println!("write scratch (stub)"),
            WriteKind::Derived { .. } => println!("write derived (stub)"),
        },
        Commands::Ingest => println!("ingest (stub)"),
        Commands::Search { query } => println!("search {:?} (stub)", query),
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
