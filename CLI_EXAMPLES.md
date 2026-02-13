# Hyena CLI — quick examples

Run from a repo that has `.agent/POLICY.yaml` (e.g. `repos/docs`). Use `--root <dir>` if not in that repo.

## Humans

```bash
# See nearest notes and excerpt
hyena read context

# Ingest all raw inputs (NOTES.md, inbox, etc.) → .notes/notes.ndjson
hyena ingest

# Search your notes
hyena search "PR"
hyena search "scripts" --include-scratch

# Append to scratch (quick jot)
hyena write scratch "need to submit PR for branch X"

# Read what’s in scratch
hyena read scratch
```

## Agents

```bash
# Same reads; agent typically uses --actor agent (no raw write)
hyena --actor agent read context
hyena --actor agent read derived --max 50
hyena --actor agent read scratch --max 20

# Search and append findings to scratch
hyena --actor agent search "improve scripts"
hyena --actor agent write scratch "finding: see NOTES.md line 5" --kind finding

# Delta ingest (only changed paths)
hyena --actor agent ingest --only NOTES.md --only inbox/scratch.txt
```

## One-liners

```bash
hyena read context && hyena read derived --max 10
hyena ingest && hyena search "TODO"
```
