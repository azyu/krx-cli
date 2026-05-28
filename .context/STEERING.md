# Steering

> Project direction, priorities, and constraints.
> Read before starting any task.

## Current Priority

Stabilize the read-only CLI path first: schema discovery, request planning, validation, and sample endpoint execution. Do not add write operations or broad abstractions before the real read path is solid.

## Constraints

- Keep the user-facing surface as a single Rust binary (`krx`) while the repo is split into `krx-cli` and `krx-core`.
- The public KRX schema currently exposes `basDd`; avoid speculative generic abstractions.
- Prefer structured output that works for both humans and agents.
- Treat agent input as untrusted.
- Do not depend on browser-based auth flows.
- Persist config under `~/.config/krx` on every OS, even on Windows.

## Decisions Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-07 | Use Rust + `clap` for the initial CLI | Matches requested stack and keeps distribution simple |
| 2026-03-07 | Include `schema` and `--dry-run` in the first scaffold | Directly follows the referenced article's agent-friendly CLI guidance |
| 2026-03-07 | Keep a built-in API catalog in `krx-core` | Avoids runtime scraping and gives the CLI self-description |
| 2026-03-07 | Use sample endpoint support from day one | Enables end-to-end verification without a private issued key |
| 2026-03-07 | Standardize config storage at `~/.config/krx/config.json` | Keeps docs and automation examples identical across macOS, Linux, and Windows |
| 2026-03-07 | Split the repo into `krx-cli` and `krx-core` crates | Keeps `clap` isolated from reusable runtime logic while preserving a single user-facing command |

## Notes

- Detailed upstream API research lives in `docs/reference.md`.
- Architectural rationale and open questions live in `docs/references.md`.
