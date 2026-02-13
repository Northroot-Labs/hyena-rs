# Agent rules (Hyena)

Agents operating in this repo under the Hyena convention must:

1. **Load and follow** `.agent/POLICY.yaml`. No writes to raw inputs (NOTES.md, etc.); append-only to derived logs and scratch.
2. **Follow org doctrine:** [HYENA_DOCTRINE](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_DOCTRINE.md).
3. **Org context:** See [docs/ORG_CONTEXT.md](../docs/ORG_CONTEXT.md) for all canon links (dates, testing, secrets, model choice).

Suggested edits to human notes: emit as patches under `.work/patches/`, never apply automatically.
