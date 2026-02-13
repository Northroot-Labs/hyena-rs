# hyena-rs repo notes

Context-local notes for this repo. Hyena convention: raw human input; agents read-only here.

- **Knowledge transfer:** Org canon and standards are linked from [docs/ORG_CONTEXT.md](docs/ORG_CONTEXT.md). No duplicate of long docs—link to Northroot-Labs/docs.
- **Focus:** Make hyena useful; implement and test CLI per HYENA_RS_TASKS (read raw, ingest, write scratch, search, human append-raw). Dogfood this repo (NOTES.md, .agent/, ingest, read context).
- **Current state:** read context + policy load implemented; other commands stubbed. CI: fmt, clippy, test on main/PR.
