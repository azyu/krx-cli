# AGENTS.md

> Note: `CLAUDE.md` is a symlink to this file.

## Project Structure

- `crates/cli/src/main.rs`: binary entrypoint
- `crates/cli/src/app.rs`: command dispatch and output mode resolution
- `crates/cli/src/cli.rs`: `clap` models for subcommands and flags
- `crates/cli/src/output.rs`: JSON/text output helpers
- `crates/core/src/catalog.rs`: built-in KRX API registry and schema views
- `crates/core/src/client.rs`: parameter parsing, validation, request planning, HTTP execution
- `crates/core/src/config.rs`: config path resolution and persisted auth key management
- `crates/core/src/error.rs`: typed user-facing errors
- `crates/core/src/runtime.rs`: clap-free reusable runtime surface
- `docs/reference.md`: researched KRX API inventory and endpoint notes
- `docs/references.md`: project decisions, article takeaways, open questions
- `CONTEXT.md`: agent-facing usage and safety rules

## Build & Development

- Install/build: `cargo build -p krx-cli --bin krx`
- Format: `cargo fmt --all`
- Full tests: `cargo test`
- Focused test example: `cargo test -p krx-core client::tests::parse_params_rejects_invalid_date`
- List APIs: `cargo run -p krx-cli -- schema list`
- Show schema: `cargo run -p krx-cli -- --output json schema show krx_dd_trd`
- Dry-run call: `cargo run -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample --dry-run`
- Sample live call: `cargo run -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample`
- Config path: `cargo run -p krx-cli -- config path`
- Save auth key: `cargo run -p krx-cli -- config set-auth-key YOUR_ISSUED_KEY`
- Show config: `cargo run -p krx-cli -- --output json config show`

## Code Standards

### Do

- Keep the CLI read-only unless the user explicitly asks for write operations.
- Add API metadata in `crates/core/src/catalog.rs` and update `docs/reference.md` in the same change.
- Route user input through `crates/core/src/client.rs` validation instead of parsing flags ad hoc in command handlers.
- Keep all persisted configuration under `~/.config/krx/config.json` via `crates/core/src/config.rs`.
- Prefer structured JSON output for anything an agent or script will consume.
- Use `KrxCliError` for user-facing failures so error messages stay consistent.
- Keep new modules small and single-purpose; this repo does not need framework-style layering.

### Don't

- Do not hardcode real KRX API keys in code, docs, tests, or examples.
- Do not write config files anywhere except `~/.config/krx` unless the user asks for a migration.
- Do not assemble KRX URLs outside `crates/core/src/catalog.rs` and `crates/core/src/client.rs`.
- Do not bypass `--dry-run` when adding future mutating commands.
- Do not add one-off flags that duplicate nested payload structure when `--params` can cover it.
- Do not silently accept unknown query fields; reject them.

## After Code Changes

1. Run `cargo fmt --all`
2. Run a targeted test if you changed validation or request planning
3. Run `cargo test`
4. Smoke test the CLI:
   `cargo run -p krx-cli -- --output json schema show krx_dd_trd`
5. Smoke test request planning:
   `cargo run -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample --dry-run`
6. If networking code changed, verify one sample call:
   `cargo run -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample`

## Definition of Done

A task is done only when all of the following are true:

- The requested change is complete and stays within the requested scope.
- Required code and docs are updated together when the public CLI behavior or runtime catalog changes.
- The verification steps in `After Code Changes` have been completed for the affected surface.
- Any relevant targeted test added for the change passes.

## Testing

- Keep unit tests next to the code they verify.
- Prioritize validation tests for bad dates, unknown parameters, unsafe characters, and auth resolution.
- Prefer deterministic tests that do not depend on live KRX availability.
- Use sample endpoint smoke tests only as secondary verification.

## Commit & PR

- Use conventional prefixes: `feat:`, `fix:`, `docs:`, `chore:`
- Mention affected API IDs in the commit body when changing catalog coverage.
- Keep docs and code together in the same PR when the public KRX schema changes.
- Once the Definition of Done is satisfied, make sure the work is on a branch based on `main`.
- Commit in logical steps instead of one large snapshot commit.
- Push that branch to the remote.
- Open a PR for review.

## Secrets & Environment

- Real API access uses `KRX_API_KEY` or `--auth-key`.
- Persistent config lives at `~/.config/krx/config.json` on every OS; Windows resolves `~` to the user home directory.
- Never commit `.env` files or issued keys.
- The public sample key is only for `--sample` flows; do not treat it as production access.

## Known Gotchas

- Real endpoint URLs currently use `/svc/apis/{path}/{apiId}` without `.json` or `.xml`.
- Sample endpoint URLs require the format suffix.
- The currently documented public KRX APIs all expose `basDd`, but this may change upstream.
- `docs/reference.md` is the detailed reference; `crates/core/src/catalog.rs` only keeps the runtime subset needed by the CLI.
- Auth resolution order is `--auth-key` > `KRX_API_KEY` > `~/.config/krx/config.json`.

## Multi-Agent Coordination

Before starting any task:

1. Read `.context/TASKS.md`
2. Read `.context/STEERING.md`
3. Update `.context/TASKS.md` to `[~]` with your agent name
4. Mark it `[x]` when complete

### File Purposes

| File | Purpose |
|------|---------|
| `.context/TASKS.md` | Shared task tracker |
| `.context/STEERING.md` | Current direction, priorities, and constraints |
