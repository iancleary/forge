#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev-local.sh

Dev-only installer for an already-present local checkout:
  - installs the binaries listed in scripts/install-forge-release.sh via `cargo install`

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

release_installer="$repo_root/scripts/install-forge-release.sh"
if [[ ! -f "$release_installer" ]]; then
  echo "error: missing release installer: $release_installer" >&2
  exit 1
fi

echo "Installing Forge binaries from local checkout: $repo_root"

extract_bins() {
  # Single source of truth: binaries list embedded in scripts/install-forge-release.sh.
  sed -n '/^  # BEGIN FORGE_BINARIES$/,/^  # END FORGE_BINARIES$/p' "$release_installer" \
    | sed -e '1d' -e '$d' \
    | sed -e "/^  cat <<'EOF'$/d" -e '/^EOF$/d' \
    | sed -e 's/[[:space:]]*$//' -e '/^$/d' -e '/^#/d'
}

extract_bins | while IFS= read -r bin; do
  case "$bin" in
    ""|\#*)
      continue
      ;;
  esac

  echo "  - $bin"

  crate_dir="$repo_root/crates/$bin"
  if [[ ! -f "$crate_dir/Cargo.toml" ]]; then
    echo "error: expected crate directory with Cargo.toml: $crate_dir" >&2
    exit 1
  fi

  cargo install --locked $force --path "$crate_dir"
done
