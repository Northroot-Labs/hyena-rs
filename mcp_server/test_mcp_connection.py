#!/usr/bin/env python3
"""
Automated MCP connection test. Run from mcp_server dir:
  uv run python test_mcp_connection.py
Uses HYENA_ROOT and HYENA_BIN from env, or defaults for repo layout.
Exits 0 if connection and list_tools/list_resources succeed; non-zero and prints findings on failure.
"""
import asyncio
import os
import sys
from pathlib import Path

# Repo layout: mcp_server is at repos/hyena-rs/mcp_server; docs at repos/docs; binary at repos/hyena-rs/target/debug/hyena
SCRIPT_DIR = Path(__file__).resolve().parent
HYENA_RS_ROOT = SCRIPT_DIR.parent
# From mcp_server: parent=hyena-rs, parent.parent=repos (or workspace root)
REPOS_OR_WORKSPACE = HYENA_RS_ROOT.parent
DEFAULT_HYENA_ROOT = str(REPOS_OR_WORKSPACE / "docs")
DEFAULT_HYENA_BIN = str(HYENA_RS_ROOT / "target" / "debug" / "hyena")


async def run_mcp_test() -> str | None:
    """Return None on success, or an error message string on failure."""
    try:
        from mcp import ClientSession, StdioServerParameters
        from mcp.client.stdio import stdio_client
    except ImportError as e:
        return f"MCP client import failed: {e}. Run: uv sync"

    hyena_root = os.environ.get("HYENA_ROOT", DEFAULT_HYENA_ROOT)
    hyena_bin = os.environ.get("HYENA_BIN", DEFAULT_HYENA_BIN)
    if not Path(hyena_root).exists():
        return f"HYENA_ROOT dir not found: {hyena_root}"
    if not Path(hyena_bin).exists():
        return f"HYENA_BIN binary not found: {hyena_bin}. Run: cargo build (in hyena-rs)"

    params = StdioServerParameters(
        command="uv",
        args=["run", "python", "-m", "hyena_mcp"],
        cwd=str(SCRIPT_DIR),
        env={**os.environ, "HYENA_ROOT": hyena_root, "HYENA_BIN": hyena_bin},
    )

    try:
        async with stdio_client(params) as (read, write):
            async with ClientSession(read, write) as session:
                await session.initialize()
                tools_result = await session.list_tools()
                tools = [t.name for t in tools_result.tools]
                if "hyena_read_context" not in tools:
                    return f"Expected hyena_read_context in tools. Got: {tools}"
                resources_result = await session.list_resources()
                uris = [str(r.uri) for r in resources_result.resources]
                if "hyena://context" not in uris:
                    return f"Expected hyena://context in resources. Got: {uris}"
    except Exception as e:
        return f"MCP connection or protocol error: {e}"

    return None


def main() -> int:
    err = asyncio.run(run_mcp_test())
    if err:
        print("MCP connection test FAILED:", err, file=sys.stderr)
        return 1
    print("MCP connection test OK: list_tools and list_resources succeeded.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
