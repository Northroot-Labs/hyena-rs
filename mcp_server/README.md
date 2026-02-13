# Hyena MCP Server

Local MCP server that exposes Hyena as an **agent substrate**: read context, derived log, scratch, search, and write scratch. Use from Cursor or any MCP client so agents can feed on Hyena context and persist findings to scratch.

## Setup

1. **Build the Hyena CLI** (parent repo): `cargo build --release` in `repos/hyena-rs`. Ensure `hyena` is on your `PATH` or set `HYENA_BIN` to the binary path.
2. **Install and run the MCP server** from this directory:

   ```bash
   uv sync
   uv run python -m hyena_mcp
   ```

   Or with explicit root and binary:

   ```bash
   HYENA_ROOT=/path/to/workspace HYENA_BIN=/path/to/hyena uv run python -m hyena_mcp
   ```

## Cursor configuration

**Project wiring (Northroot-Labs workspace):** Open the repo from the org workspace root; `.cursor/mcp.json` already points the Hyena MCP server at `repos/docs` (HYENA_ROOT) and the debug binary. Run `cargo build` in `repos/hyena-rs` and `uv sync` in `repos/hyena-rs/mcp_server` once, then use the "hyena" MCP server in Cursor.

**Manual config** (e.g. `~/.cursor/mcp.json` or another project):

```json
{
  "mcpServers": {
    "hyena": {
      "command": "uv",
      "args": ["run", "python", "-m", "hyena_mcp"],
      "cwd": "/path/to/hyena-rs/mcp_server",
      "env": {
        "HYENA_ROOT": "/path/to/workspace-with-agent-and-notes",
        "HYENA_BIN": "/path/to/hyena"
      }
    }
  }
}
```

If `HYENA_ROOT` is unset, the server uses the process current working directory.

## Tools (agent callable)

| Tool | Description |
|------|-------------|
| `hyena_read_context` | Nearest NOTES.md path and excerpt (optional path, max_lines). |
| `hyena_read_derived` | Entries from `.notes/notes.ndjson` (optional scope_contains, max). |
| `hyena_read_raw` | All raw input content in scope (optional scope). |
| `hyena_read_scratch` | Scratch log entries (optional max). |
| `hyena_search` | Grep-derived log (and optionally scratch) for query. |
| `hyena_write_scratch` | Append a line to scratch (text, optional kind). |
| `hyena_ingest` | Run ingest (optional semantic_dedupe, optional only_paths for delta). |

## Resources (load into context)

| URI | Description |
|-----|-------------|
| `hyena://context` | Same as read context. |
| `hyena://derived` | Same as read derived (recent entries). |
| `hyena://scratch` | Same as read scratch. |

## Automated test (MCP connection)

From `mcp_server` dir (after `cargo build` in hyena-rs and `uv sync` here):

```bash
uv run python test_mcp_connection.py
```

Exits 0 if the server starts, accepts a client connection, and returns expected tools and resources. On failure it prints findings (e.g. binary not found, connection error).

## Environment

- **HYENA_ROOT** — Workspace root (directory with `.agent/POLICY.yaml`). Default: process cwd.
- **HYENA_BIN** — Path to `hyena` binary. Default: `hyena` (must be on PATH).
