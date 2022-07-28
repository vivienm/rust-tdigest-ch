DEFAULT: check test fmt clippy doc deny

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

deny:
    cargo deny --all-features check
