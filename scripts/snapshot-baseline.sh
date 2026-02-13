#!/usr/bin/env bash
# Snapshot current main as tag baseline-day1. Run from repo root.
set -e
cd "$(dirname "$0")/.."
git checkout main
git pull --ff-only
git tag -a baseline-day1 -m "Baseline: CI green, policy load, read context, tests"
echo "Tag baseline-day1 created. Push with: git push origin baseline-day1"
