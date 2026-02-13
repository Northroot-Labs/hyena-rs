# Pinned Hyena release (for MCP and scripts)

Use this version and checksum to install the Hyena CLI reproducibly.

| Field | Value |
|-------|--------|
| **Version** | 0.2.0 |
| **Tag** | v0.2.0 |
| **Release URL** | https://github.com/Northroot-Labs/hyena-rs/releases/tag/v0.2.0 |

## Checksums (darwin/linux)

After the GitHub release is created, download the artifact for your platform and verify:

```text
# arm64 macOS (Apple Silicon)
<checksum>  hyena-0.2.0-arm64-darwin
```

Current build (arm64-darwin):

```text
8a53490b30a0176d3d3db390520a1fd05d23978b6b0121f27fee677df2f46316  hyena-0.2.0-arm64-darwin
```

## Create GitHub release (one-time after tag is pushed)

From `repos/hyena-rs` with `release/` containing the built artifact and `release/checksums.txt`:

```bash
gh release create v0.2.0 \
  --repo Northroot-Labs/hyena-rs \
  --title "v0.2.0" \
  --notes "Delta ingest (--only paths), MCP server. See RELEASE_PIN.md for checksums." \
  release/hyena-0.2.0-*-* release/checksums.txt
```

Replace `release/hyena-0.2.0-*-*` with the actual artifact path(s) if needed (e.g. `release/hyena-0.2.0-arm64-darwin`).

## Install (pinned)

From workspace root:

```bash
./scripts/install-hyena-release.sh
```

Installs to `repos/hyena-rs/bin/hyena` and verifies against `checksums-pin.txt`. MCP uses `HYENA_BIN=../../bin/hyena` (from `mcp_server` cwd).
