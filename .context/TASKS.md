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
| [x] | Add release install path setup for `~/.local/bin/krw` | Codex |
| [x] | Add `--fields` on top of `--body-only` | Codex |
| [x] | Evaluate MCP or library surface after CLI basics stabilize | Codex |
| [x] | Review current project for additional `$rust-cli` updates | Codex |
| [x] | Verify current config storage path behavior in code | Codex |
| [x] | Make README wording more polite and user-friendly | Codex |
| [x] | Check whether `~/.agents/skills/create-readme` exists and is readable | Codex |
| [x] | Commit and push `README.md` wording updates | Codex |
| [x] | Update `README.md` with current project capabilities and usage | Codex |
| [x] | Commit and push refreshed `README.md` content | Codex |
| [x] | Audit current implementation and complete the next highest-priority CLI gaps | Codex |
| [x] | Make clap parse failures follow the JSON error contract | Codex |
| [x] | Split the project into `krx-cli` and `krx-core` crates | Codex |
| [x] | Audit implemented KRX API coverage and identify missing runtime support | Codex |
| [x] | Review GitHub Actions and Homebrew release support against `/Volumes/WD_Blue_1TB/Code/Personal/bb-cli` | Codex |
| [x] | Inspect current Homebrew release/distribution readiness and note minimal GitHub Actions gaps | Codex |
| [x] | Add GitHub release workflows and Homebrew tap update support | Codex |
| [x] | Retarget release artifacts to linux amd64, macOS arm64, Windows x64, and Windows arm64 | Codex |
| [x] | Add linux arm64 release targeting and align Homebrew/docs | Codex |
| [x] | Define project DoD in `AGENTS.md` and add post-DoD branch/push/PR workflow | Codex |
| [x] | Commit and push the AGENTS.md DoD update | Codex |
| [x] | Apply the AGENTS.md DoD update to `main` and push it | Codex |

## Status Legend

- `[ ]` — Pending
- `[~]` — In progress
- `[x]` — Done
- `[!]` — Blocked (add reason in Notes below)

## Notes

- Real endpoint calls require an issued and approved KRX API key.
