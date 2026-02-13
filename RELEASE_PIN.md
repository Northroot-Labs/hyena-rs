# Pinned Hyena release (checkpoint-based)

Versioning is **automatic** and **machine-checkable** per [VERSIONING_STANDARD](../docs/internal/ci/VERSIONING_STANDARD.md). We pin by **checkpoint ID**, not semver.

## Pin files

| File | Purpose |
|------|--------|
| `release-pin.txt` | Single line `CHECKPOINT_ID=cp-YYYYMMDD-<short_sha>`. Defines which build to install. |
| `checksums-pin.txt` | One line per artifact: `sha256  hyena-<checkpoint_id>-<arch>-<os>`. Verification only. |

## Build release (local)

From `repos/hyena-rs`:

```bash
./scripts/build-release.sh
```

Produces `release/hyena-<checkpoint_id>-<arch>-<os>`, `release/checksums.txt`, and `release/release-manifest.json`. Checkpoint ID is derived from git and build date (no manual version bump).

## Create GitHub release

Tag = checkpoint ID. From `repos/hyena-rs` after running `build-release.sh`:

```bash
cp="$(grep checkpoint_id release/release-manifest.json | cut -d'"' -f4)"
gh release create "$cp" \
  --repo Northroot-Labs/hyena-rs \
  --title "$cp" \
  --notes "Checkpoint release. Verify with release-manifest.json or checksums-pin.txt." \
  release/hyena-"$cp"-* release/checksums.txt release/release-manifest.json
```

## Install (pinned)

**Local (no GitHub release):** From `repos/hyena-rs`, after `build-release.sh`:

```bash
./scripts/install-hyena-local.sh
```

Puts the built binary in `bin/hyena`. Cursor MCP uses `HYENA_BIN=../../bin/hyena`.

**From GitHub:** From workspace root, after a release exists with tag = checkpoint ID:

```bash
./scripts/install-hyena-release.sh
```

Reads `release-pin.txt`, downloads artifact, verifies with `checksums-pin.txt`, installs to `repos/hyena-rs/bin/hyena`.

## Updating the pin

After a new checkpoint release: set `CHECKPOINT_ID` in `release-pin.txt` and add the new artifact line(s) to `checksums-pin.txt`; commit both.
