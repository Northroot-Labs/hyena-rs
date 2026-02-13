#!/usr/bin/env bash
# Snapshot current main as tag baseline-day1. No editor — uses -F message file.
# Works in Helix, Cursor, or raw terminal.
set -e
cd "$(dirname "$0")/.."
git checkout main
git pull --ff-only
msg="scripts/tag-baseline-day1.txt"
[[ -f "$msg" ]] || { echo "Missing $msg"; exit 1; }
# Avoid opening editor when tag.gpgsign=true (uses -F and disables sign for this tag)
git -c tag.gpgsign=false tag -a baseline-day1 -F "$msg"
echo "Tag baseline-day1 created. Push with: git push origin baseline-day1"
