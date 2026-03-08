[group('dev')]
local-dev: setup-watch
  CARGO_TEST=1 cargo watch -x "nextest run --workspace"

[group('dev')]
setup:
  just setup-watch
  just setup-nextest
  just setup-coverage
  just setup-audit
  just setup-toml

[group('dev')]
setup-nextest:
  cargo install cargo-nextest --locked

[group('dev')]
setup-watch:
  cargo install cargo-watch --locked

[group('dev')]
setup-coverage:
  rustup component add llvm-tools-preview
  cargo install cargo-llvm-cov

[group('dev')]
setup-audit:
  cargo install cargo-audit --locked

[group('dev')]
setup-toml:
  cargo install taplo-cli --locked

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
test-suite: setup-nextest
  cargo nextest run --workspace

[group('test')]
test-fmt:
  cargo fmt --check

[group('test')]
test-lint:
  cargo clippy --workspace -- -D warnings

[group('test')]
test-audit: setup-audit
  cargo audit

[group('test')]
test-toml: setup-toml
  taplo fmt -c .taplo.toml --check

[group('test')]
test-coverage: setup-nextest setup-coverage
  cargo llvm-cov nextest --workspace --tests --html --open

[group('test')]
test: test-fmt test-lint test-suite test-toml test-audit test-coverage

[group('test')]
ci: test-fmt test-lint test-suite test-toml test-audit
