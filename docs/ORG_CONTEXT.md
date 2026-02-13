# Org context for hyena-rs

Single reference for Northroot-Labs org standards and canon. All links point to **Northroot-Labs/docs** (source of truth). Do not duplicate long docs here—link.

## Hyena (convention and CLI)

| Resource | Purpose |
|----------|---------|
| [Hyena README](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/README.md) | Convention: when to use, repo layout, canonical docs table |
| [HYENA_DOCTRINE.md](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_DOCTRINE.md) | Behavioral doctrine: raw immutable, append-only, provenance, uncertainty |
| [hyena-policy-spec.yaml](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/hyena-policy-spec.yaml) | Full policy spec (filesystem, modes, invariants) |
| [HYENA_CLI_SPEC.md](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_CLI_SPEC.md) | CLI contract this repo implements |
| [HYENA_RS_TASKS.md](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/HYENA_RS_TASKS.md) | Task list (v1–v4); canonical in docs |

## Agentic and model choice

| Resource | Purpose |
|----------|---------|
| [AGENTIC_MODEL_AND_BOUNDS.md](https://github.com/Northroot-Labs/docs/blob/main/internal/agent/AGENTIC_MODEL_AND_BOUNDS.md) | Model by use case, when, capabilities, budget, time limits, runtime gates |
| [Model-choice proposal](https://github.com/Northroot-Labs/docs/blob/main/internal/research/model-choice-hyena/artifacts/model-choice-proposal.md) | GCP/OpenAI/Anthropic catalog, use-case × model, budget/token defaults |
| [matrix.yaml](https://github.com/Northroot-Labs/docs/blob/main/internal/research/model-choice-hyena/artifacts/matrix.yaml) | Machine-readable suitability and budget_defaults |

## Org standards

| Resource | Purpose |
|----------|---------|
| [DATE_STANDARD.md](https://github.com/Northroot-Labs/docs/blob/main/internal/ci/DATE_STANDARD.md) | Dates never agent-generated; use `@runtime` or script/git at runtime |
| [TESTING_STANDARD.md](https://github.com/Northroot-Labs/docs/blob/main/internal/ci/TESTING_STANDARD.md) | Tests required; CI runs tests; runtime gates |
| [SECRETS_AND_IDENTITY_STANDARD.md](https://github.com/Northroot-Labs/docs/blob/main/internal/security/SECRETS_AND_IDENTITY_STANDARD.md) | Secrets, identity, tokens for tooling |

## Research (optional)

| Resource | Purpose |
|----------|---------|
| [Research README](https://github.com/Northroot-Labs/docs/blob/main/internal/research/README.md) | Versioned research specs, DAG workload, validator |

## Scope

When working on **making hyena useful**: this repo (hyena-rs) is the sole implementation scope. Org canon lives in docs; link, don’t copy. When context is crystal clean, workspace can be reduced to org-level important repos only (e.g. docs + hyena-rs).
