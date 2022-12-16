.PHONY: fmt-check fmt clippy benchmarks

fmt-check:
	cargo +nightly fmt --all -- --check

fmt:
	cargo +nightly fmt

clippy:
	cargo +nightly clippy --all-features --tests --

benchmarks:
    cargo check --features=runtime-benchmarks --release