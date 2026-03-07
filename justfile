[group('dev')]
setup:
  cargo install cargo-nextest --locked
  rustup component add llvm-tools-preview
  cargo install cargo-llvm-cov

[group('test')]
local-test: setup release
  CARGO_TEST=1 cargo watch -x "nextest run --workspace"

[group('dev')]
local-web:
  cargo run --bin web

[group('dev')]
release:
  cargo build --release

[group('test')]
test-suite: setup
  cargo nextest run --workspace

[group('test')]
test-fmt:
  cargo fmt --check

[group('test')]
test-lint:
  cargo clippy --workspace -- -D warnings

[group('test')]
test-coverage: setup
  cargo llvm-cov nextest --workspace --tests --html --open

[group('test')]
test: test-fmt test-lint test-suite test-coverage
