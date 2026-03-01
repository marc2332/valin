f:
    taplo fmt
    cargo +nightly fmt --all -- --error-on-unformatted --unstable-features

c:
    taplo check
    cargo clippy --workspace -- -D warnings

r:
    cargo run -- --performance-overlay ./Cargo.lock Cargo.toml