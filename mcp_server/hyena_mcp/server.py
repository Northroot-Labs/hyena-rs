"""
Hyena MCP server: exposes read context, derived, raw, scratch, search, write scratch, ingest.
Uses the hyena CLI binary; set HYENA_ROOT (workspace root) and optionally HYENA_BIN.
"""

import os
import subprocess
from pathlib import Path

from mcp.server.fastmcp import FastMCP

mcp = FastMCP("Hyena")


def _root() -> Path:
    return Path(os.environ.get("HYENA_ROOT", os.getcwd())).resolve()


def _hyena_bin() -> str:
    return os.environ.get("HYENA_BIN", "hyena")


def _run_hyena(*args: str, root: Path | None = None) -> str:
    root = root or _root()
    cmd = [_hyena_bin(), "--root", str(root), *args]
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        timeout=60,
        cwd=str(root),
    )
    out = (result.stdout or "").strip()
    err = (result.stderr or "").strip()
    if result.returncode != 0 and err:
        return f"Error (exit {result.returncode}): {err}\n{out}" if out else f"Error (exit {result.returncode}): {err}"
    return out


# --- Tools ---


@mcp.tool()
def hyena_read_context(path: str = "", max_lines: int = 50) -> str:
    """Nearest NOTES.md path and content excerpt. path: optional relative path; max_lines: excerpt length."""
    root = _root()
    args = ["read", "context", "--max-lines", str(max_lines)]
    if path:
        args.extend(["--path", path])
    return _run_hyena(*args, root=root)


@mcp.tool()
def hyena_read_derived(scope_contains: str = "", max_entries: int = 100) -> str:
    """Entries from .notes/notes.ndjson. scope_contains: filter by path substring; max_entries: limit."""
    root = _root()
    args = ["read", "derived", "--max", str(max_entries)]
    if scope_contains:
        args.extend(["--scope-contains", scope_contains])
    return _run_hyena(*args, root=root)


@mcp.tool()
def hyena_read_raw(scope: str = "") -> str:
    """All raw input content (NOTES.md, inbox, etc.) in scope. scope: optional subdir path."""
    root = _root()
    args = ["read", "raw"]
    if scope:
        args.extend(["--scope", scope])
    return _run_hyena(*args, root=root)


@mcp.tool()
def hyena_read_scratch(max_entries: int = 50) -> str:
    """Scratch log entries (.hyena/agent/scratch.ndjson)."""
    root = _root()
    return _run_hyena("read", "scratch", "--max", str(max_entries), root=root)


@mcp.tool()
def hyena_search(query: str, include_scratch: bool = False) -> str:
    """Search derived log (and optionally scratch) for query string."""
    root = _root()
    args = ["search", query]
    if include_scratch:
        args.append("--include-scratch")
    return _run_hyena(*args, root=root)


@mcp.tool()
def hyena_write_scratch(text: str, kind: str = "note") -> str:
    """Append a line to the scratch log. Use for findings, reminders, or session state."""
    root = _root()
    return _run_hyena("write", "scratch", text, "--kind", kind, root=root)


@mcp.tool()
def hyena_ingest(semantic_dedupe: bool = False, only_paths: str = "") -> str:
    """Run ingest: discover raw inputs, chunk, append to .notes/notes.ndjson. only_paths: comma-separated paths for delta (e.g. NOTES.md,inbox/scratch.txt)."""
    root = _root()
    args = ["ingest"]
    if semantic_dedupe:
        args.append("--semantic-dedupe")
    if only_paths:
        for p in only_paths.split(","):
            p = p.strip()
            if p:
                args.extend(["--only", p])
    return _run_hyena(*args, root=root)


# --- Resources (for loading into context) ---


@mcp.resource("hyena://context")
def resource_context() -> str:
    """Hyena context: nearest NOTES.md path and excerpt."""
    return hyena_read_context(max_lines=80)


@mcp.resource("hyena://derived")
def resource_derived() -> str:
    """Hyena derived log: recent atoms from .notes/notes.ndjson."""
    return hyena_read_derived(max_entries=100)


@mcp.resource("hyena://scratch")
def resource_scratch() -> str:
    """Hyena scratch: agent working memory."""
    return hyena_read_scratch(max_entries=50)


def main() -> None:
    mcp.run(transport="stdio")


if __name__ == "__main__":
    main()
