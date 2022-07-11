DEFAULT: ci
cargo := "cargo"

ci: check test fmt clippy doc audit

build:
    {{cargo}} build

check:
    {{cargo}} check

test:
    {{cargo}} test

fmt:
    {{cargo}} fmt --all -- --check

clippy:
    {{cargo}} clippy -- -D warnings

doc:
    {{cargo}} rustdoc -- -D warnings

audit:
    {{cargo}} audit
