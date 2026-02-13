# hyena-rs

Rust CLI for **Hyena**: policy-enforcing, file-first agent substrate. Implements the [Hyena CLI contract](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_CLI_SPEC.md) and loads policy from `.agent/POLICY.yaml`.

- **Org convention:** [Hyena (docs)](https://github.com/Northroot-Labs/docs/tree/main/internal/agent) — doctrine, policy spec, repo layout.
- **Task list:** [HYENA_RS_TASKS.md](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_RS_TASKS.md) (canonical in Northroot-Labs/docs).
- **Org context (knowledge transfer):** [docs/ORG_CONTEXT.md](docs/ORG_CONTEXT.md) — single place for links to DATE_STANDARD, TESTING_STANDARD, AGENTIC/model-choice, secrets, research. Scope: this repo is the sole focus for making hyena useful.

## Build / install

```bash
cargo build --release
# Binary: target/release/hyena
```

## Commands

- `read context | raw | derived | scratch | agent-log`
- `write scratch | agent-log | derived`
- `ingest` (optionally `--only path` for delta)
- `search QUERY` (optionally `--include-scratch`)
- `cluster` → `.work/clusters/`
- `human append-raw` (actor=human only)

Invocation: `--root <path>` (default: cwd), `--policy <path>` (default: `{root}/.agent/POLICY.yaml`), `--actor human|agent`.

**Examples:** [CLI_EXAMPLES.md](CLI_EXAMPLES.md) — one-liners for humans and agents.

**MCP (Cursor/agents):** [mcp_server/README.md](mcp_server/README.md). Workspace root has `.cursor/mcp.json` to wire the Hyena MCP server for local testing.

## License

MIT.
