SHELL := /bin/bash

.PHONY: fmt fmt-check lint test smoke drift-check build-release hooks-install

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --locked

smoke:
	cargo run --quiet -p krx-cli -- --output json schema show krx_dd_trd >/dev/null
	cargo run --quiet -p krx-cli -- --output json call krx_dd_trd --date 20200414 --sample --dry-run >/dev/null

drift-check:
	./scripts/check-catalog-drift.sh

build-release:
	cargo build --locked -p krx-cli --bin krx --release

hooks-install:
	git config core.hooksPath .githooks
