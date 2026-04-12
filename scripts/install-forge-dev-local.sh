#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev-local.sh

Dev-only installer for an already-present local checkout:
  - installs the repo's known workspace binary crates via `cargo install`

Usage:
  scripts/install-forge-dev-local.sh [--path PATH] [--no-force]

Options:
  --path      Repo root (default: `git rev-parse --show-toplevel` or current dir)
  --no-force  Do not pass --force to cargo install
  -h,--help   Show help
EOF
}

repo_root=""
force="--force"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --path)
      repo_root="${2:-}"
      shift 2
      ;;
    --no-force)
      force=""
      shift 1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: missing required command: $1" >&2
    exit 1
  fi
}

need_cmd cargo

if [[ -z "$repo_root" ]]; then
  if command -v git >/dev/null 2>&1 && git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    repo_root="$(git rev-parse --show-toplevel)"
  else
    repo_root="$(pwd)"
  fi
fi

if [[ ! -f "$repo_root/Cargo.toml" ]]; then
  echo "error: no Cargo.toml at repo root: $repo_root" >&2
  exit 1
fi

# Keep this list explicit. As repo authors, we prefer deterministic behavior over dynamic discovery.
crates=(
  "crates/forge"
  "crates/linear"
  "crates/slack-cli"
  "crates/codex-threads"
)

echo "Installing ${#crates[@]} binaries from local checkout: $repo_root"
for crate_rel in "${crates[@]}"; do
  echo "  - ${crate_rel#crates/}"
done

for crate_rel in "${crates[@]}"; do
  crate_dir="$repo_root/$crate_rel"
  if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
    echo "error: expected crate directory with Cargo.toml: $crate_dir" >&2
    exit 1
  fi
  cargo install --locked $force --path "$crate_dir"
done
