DEFAULT: check test fmt clippy doc typos deny

build:
    cargo build --all-features

check:
    cargo check --all-features

test:
    cargo test --all-features

fmt:
    cargo fmt --all -- --check

clippy:
    cargo clippy -- -D warnings

doc:
    cargo rustdoc --all-features -- -D warnings

typos:
    typos

deny:
    cargo deny --all-features check
