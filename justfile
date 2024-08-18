ci: fmt clippy test doc audit typos

fmt:
  cargo fmt --check

clippy *args="":
  cargo clippy --all-targets --all-features {{args}}

test *args="":
  cargo test --all-features {{args}}

doc *args="":
  cargo doc --no-deps --all-features {{args}}

audit:
  cargo audit

typos:
  typos
