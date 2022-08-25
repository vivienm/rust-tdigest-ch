DEFAULT: fmt check test clippy doc deny typos

fmt:
    cargo fmt --check

build *args="":
    cargo build --all-features {{args}}

check:
    cargo hack --feature-powerset check --all-targets

test *args="":
    cargo test --all-features {{args}}

clippy *args="":
    cargo clippy {{args}}

doc *args="":
    cargo doc --no-deps --all-features {{args}}

deny:
    cargo deny --all-features check

typos:
    typos
