set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

root := justfile_directory()

default:
  @just --list

# Rust dev
check:
  cargo check

cut-release *args:
  args='{{args}}'; case " $args " in *" --dry-run "*) cargo run -q -p forge -- release run $args ;; *) cargo run -q -p forge -- release run --apply $args ;; esac

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

release-installer-posix-test:
  sh "{{root}}/scripts/tests/install-forge-release.sh"

release-installer-windows-test:
  pwsh -NoProfile -File "{{root}}/scripts/tests/install-forge-release.ps1"

release-artifact-format-test:
  sh "{{root}}/scripts/tests/build-forge-release-artifact.sh"

release-process-check:
  sh "{{root}}/scripts/check-release-process.sh"

ci: fmt-check clippy test install-list-check release-process-check release-installer-posix-test

doc:
  cargo doc --no-deps

# Forge helpers
run-forge *args:
  cargo run -p forge -- {{args}}

run-linear *args:
  cargo run -p linear -- {{args}}

run-mermaid *args:
  cargo run -p mermaid -- {{args}}

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
