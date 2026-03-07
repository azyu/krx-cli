# AGENTS.md

> Note: `CLAUDE.md` is a symlink to this file.

## Project Structure

- `src/main.rs`: binary entrypoint
- `src/lib.rs`: command dispatch and output mode resolution
- `src/cli.rs`: `clap` models for subcommands and flags
- `src/catalog.rs`: built-in KRX API registry and schema views
- `src/client.rs`: parameter parsing, validation, request planning, HTTP execution
- `src/config.rs`: config path resolution and persisted auth key management
- `src/error.rs`: typed user-facing errors
- `src/output.rs`: JSON/text output helpers
- `docs/reference.md`: researched KRX API inventory and endpoint notes
- `docs/references.md`: project decisions, article takeaways, open questions
- `CONTEXT.md`: agent-facing usage and safety rules

## Build & Development

- Install/build: `cargo build`
- Format: `cargo fmt --all`
- Full tests: `cargo test`
- Focused test example: `cargo test client::tests::parse_params_rejects_invalid_date --lib`
- List APIs: `cargo run -- schema list`
- Show schema: `cargo run -- --output json schema show krx_dd_trd`
- Dry-run call: `cargo run -- --output json call krx_dd_trd --date 20200414 --sample --dry-run`
- Sample live call: `cargo run -- --output json call krx_dd_trd --date 20200414 --sample`
- Config path: `cargo run -- config path`
- Save auth key: `cargo run -- config set-auth-key YOUR_ISSUED_KEY`
- Show config: `cargo run -- --output json config show`

## Code Standards

### Do

- Keep the CLI read-only unless the user explicitly asks for write operations.
- Add API metadata in `src/catalog.rs` and update `docs/reference.md` in the same change.
- Route user input through `src/client.rs` validation instead of parsing flags ad hoc in command handlers.
- Keep all persisted configuration under `~/.config/krx/config.json` via `src/config.rs`.
- Prefer structured JSON output for anything an agent or script will consume.
- Use `KrxCliError` for user-facing failures so error messages stay consistent.
- Keep new modules small and single-purpose; this repo does not need framework-style layering.

### Don't

- Do not hardcode real KRX API keys in code, docs, tests, or examples.
- Do not write config files anywhere except `~/.config/krx` unless the user asks for a migration.
- Do not assemble KRX URLs outside `src/catalog.rs` and `src/client.rs`.
- Do not bypass `--dry-run` when adding future mutating commands.
- Do not add one-off flags that duplicate nested payload structure when `--params` can cover it.
- Do not silently accept unknown query fields; reject them.

## After Code Changes

1. Run `cargo fmt --all`
2. Run a targeted test if you changed validation or request planning
3. Run `cargo test`
4. Smoke test the CLI:
   `cargo run -- --output json schema show krx_dd_trd`
5. Smoke test request planning:
   `cargo run -- --output json call krx_dd_trd --date 20200414 --sample --dry-run`
6. If networking code changed, verify one sample call:
   `cargo run -- --output json call krx_dd_trd --date 20200414 --sample`

## Testing

- Keep unit tests next to the code they verify.
- Prioritize validation tests for bad dates, unknown parameters, unsafe characters, and auth resolution.
- Prefer deterministic tests that do not depend on live KRX availability.
- Use sample endpoint smoke tests only as secondary verification.

## Commit & PR

- Use conventional prefixes: `feat:`, `fix:`, `docs:`, `chore:`
- Mention affected API IDs in the commit body when changing catalog coverage.
- Keep docs and code together in the same PR when the public KRX schema changes.

## Secrets & Environment

- Real API access uses `KRX_API_KEY` or `--auth-key`.
- Persistent config lives at `~/.config/krx/config.json` on every OS; Windows resolves `~` to the user home directory.
- Never commit `.env` files or issued keys.
- The public sample key is only for `--sample` flows; do not treat it as production access.

## Known Gotchas

- Real endpoint URLs currently use `/svc/apis/{path}/{apiId}` without `.json` or `.xml`.
- Sample endpoint URLs require the format suffix.
- The currently documented public KRX APIs all expose `basDd`, but this may change upstream.
- `docs/reference.md` is the detailed reference; `src/catalog.rs` only keeps the runtime subset needed by the CLI.
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
