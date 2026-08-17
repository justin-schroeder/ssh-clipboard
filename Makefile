.PHONY: check test build run

check:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	npm test

test:
	cargo test --all-targets
	npm test

build:
	cargo build --release

run:
	cargo run --
