# Tasks

> Each agent MUST update this file after completing a task.
> Read this file before starting any work.

| Status | Task | Agent |
|--------|------|-------|
| [x] | Research public KRX Open API inventory and document it in `docs/reference.md` | Codex |
| [x] | Create initial Rust CLI scaffold with `clap`, schema commands, validation, and sample call support | Codex |
| [x] | Create project references, agent context, and repository instructions | Codex |
| [x] | Add cross-platform config storage rooted at `~/.config/krx` | Codex |
| [x] | Audit remaining implementation gaps and prepare multi-agent work split | Codex |
| [x] | Audit runtime catalog metadata gaps between `src/catalog.rs` and `docs/reference.md` | Codex |
| [x] | Expand runtime schema data beyond counts to full output field metadata | Codex |
| [x] | Add richer KRX error mapping for real endpoint failures | Codex |
| [x] | Add minimal `--body-only` response filtering support | Codex |
| [x] | Add release install path setup for `~/.local/bin/krx` | Codex |
| [ ] | Evaluate `--fields` on top of `--body-only` | — |
| [ ] | Evaluate MCP or library surface after CLI basics stabilize | — |
| [x] | Review current project for additional `$rust-cli` updates | Codex |
| [x] | Verify current config storage path behavior in code | Codex |
| [x] | Make README wording more polite and user-friendly | Codex |
| [x] | Check whether `~/.agents/skills/create-readme` exists and is readable | Codex |
| [x] | Commit and push `README.md` wording updates | Codex |
| [x] | Assess MDCSTAT024 investor net-buy source claim | Codex |
| [x] | Rename public binary from `krw` to `krx` | Codex |

## Status Legend

- `[ ]` — Pending
- `[~]` — In progress
- `[x]` — Done
- `[!]` — Blocked (add reason in Notes below)

## Notes

- Real endpoint calls require an issued and approved KRX API key.
