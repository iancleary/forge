set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

root := justfile_directory()

default:
  @just --list

# Rust dev
check:
  cargo check

fmt:
  cargo fmt

fmt-check:
  cargo fmt -- --check

clippy:
  cargo clippy -- -D warnings

test:
  cargo test

ci: fmt-check clippy test

doc:
  cargo doc --no-deps

# Forge helpers
run-forge *args:
  cargo run -p forge -- {{args}}

run-linear *args:
  cargo run -p linear -- {{args}}

run-slack *args:
  cargo run -p slack-cli -- {{args}}

run-codex-threads *args:
  cargo run -p codex-threads -- {{args}}

install-dev-local:
  "{{root}}/scripts/install-forge-dev.sh" local

install-dev-repo:
  "{{root}}/scripts/install-forge-dev.sh" repo

