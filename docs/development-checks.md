# Development Checks

This repository keeps local checks and CI checks aligned through a small set of repo-local entrypoints.

## Standard entrypoints

```bash
make fmt
make fmt-check
make lint
make test
make hooks-install
```

- `make fmt`: runs `cargo fmt --all`
- `make fmt-check`: runs `cargo fmt --all --check`
- `make lint`: runs `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `make test`: runs `cargo test --workspace --locked`
- `make hooks-install`: sets `core.hooksPath` to `.githooks`

The pinned Rust toolchain lives in `rust-toolchain.toml`. It fixes the Rust version and ensures `clippy` and `rustfmt` are part of the expected toolchain.

## Local hooks

- `.githooks/pre-commit`: runs `make fmt-check` and `make lint`
- `.githooks/pre-push`: runs `make test`

Install the hooks once per clone:

```bash
make hooks-install
```

## CI contract

`ci.yml` and `release-build.yml` both use the same repo-local commands for format, lint, and test checks. Repository-specific smoke tests and release build verification stay as separate steps so coverage does not shrink when the shared targets stay minimal.

## Copy Checklist

1. Copy `rust-toolchain.toml` and pin the exact Rust channel plus required components.
2. Copy `Makefile` and keep `fmt`, `fmt-check`, `lint`, `test`, and `hooks-install` as the standard entrypoints.
3. Copy `.githooks/pre-commit` and `.githooks/pre-push`.
4. Update CI to run `make fmt-check`, `make lint`, and `make test`.
5. Keep repo-specific smoke tests or build checks as separate CI steps when they do not belong in the shared entrypoints.
6. Update contributor-facing docs so local setup, hooks, and CI all point at the same commands.
