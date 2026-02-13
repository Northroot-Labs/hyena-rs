#!/usr/bin/env bash
# Build release binary and produce checkpoint-named artifacts + manifest.
# See repos/docs/internal/ci/VERSIONING_STANDARD.md. Run from hyena-rs root.
set -e
REPO_ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$REPO_ROOT"

BUILD_DATE="${BUILD_DATE:-$(date -u +%Y%m%d)}"
export BUILD_DATE
COMMIT_SHA=$(git rev-parse HEAD)
SHORT_SHA=$(git rev-parse --short=7 HEAD)
CHECKPOINT_ID="cp-${BUILD_DATE}-${SHORT_SHA}"
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARTIFACT="hyena-${CHECKPOINT_ID}-${ARCH}-${OS}"
RELEASE_DIR="${REPO_ROOT}/release"
mkdir -p "$RELEASE_DIR"

cargo build --release
cp "target/release/hyena" "$RELEASE_DIR/$ARTIFACT"
(cd "$RELEASE_DIR" && shasum -a 256 "$ARTIFACT" > checksums.txt)

# Release manifest (machine-checkable)
TIMESTAMP_UTC=$(date -u +%Y-%m-%dT%H:%M:%SZ)
SHA256=$(shasum -a 256 "$RELEASE_DIR/$ARTIFACT" | awk '{print $1}')
cat > "$RELEASE_DIR/release-manifest.json" << EOF
{
  "checkpoint_id": "$CHECKPOINT_ID",
  "commit_sha": "$COMMIT_SHA",
  "timestamp_utc": "$TIMESTAMP_UTC",
  "artifacts": [
    { "name": "$ARTIFACT", "sha256": "$SHA256" }
  ]
}
EOF

echo "checkpoint_id=$CHECKPOINT_ID"
echo "artifact=$ARTIFACT"
echo "manifest=$RELEASE_DIR/release-manifest.json"
echo "checksums=$RELEASE_DIR/checksums.txt"
