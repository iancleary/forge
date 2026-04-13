set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

root := justfile_directory()

default:
  @just --list

# Rust dev
check:
  cargo check

bump-version version:
  "{{root}}/scripts/bump-version.sh" "{{version}}"

fmt:
  cargo fmt

fmt-check:
  cargo fmt -- --check

clippy:
  cargo clippy -- -D warnings

test:
  cargo test

install-list-check:
  sh "{{root}}/scripts/check-forge-binaries.sh"

ci: fmt-check clippy test install-list-check

doc:
  cargo doc --no-deps

# Forge helpers
run-forge *args:
  cargo run -p forge -- {{args}}

run-linear *args:
  cargo run -p linear -- {{args}}

run-slack-query *args:
  cargo run -p slack-query -- {{args}}

run-slack-agent *args:
  cargo run -p slack-agent -- {{args}}

run-codex-threads *args:
  cargo run -p codex-threads -- {{args}}

install-dev-local:
  "{{root}}/scripts/install-forge-dev.sh" local

install-dev-repo:
  "{{root}}/scripts/install-forge-dev.sh" repo
