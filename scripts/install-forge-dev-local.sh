#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev-local.sh

Dev-only installer for an already-present local checkout:
  - discovers all workspace binary crates via `cargo metadata`
  - installs each with `cargo install`

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
need_cmd python3

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

bin_packages() {
  cargo metadata --no-deps --format-version 1 --manifest-path "$repo_root/Cargo.toml" \
    | python3 - <<'PY'
import json, sys
data = json.load(sys.stdin)
pkgs = []
for p in data.get("packages", []):
    if any("bin" in t.get("kind", []) for t in p.get("targets", [])):
        pkgs.append(p["name"])
for name in pkgs:
    print(name)
PY
}

mapfile -t pkgs < <(bin_packages)
if [[ ${#pkgs[@]} -eq 0 ]]; then
  echo "error: no binary packages found in workspace at $repo_root" >&2
  exit 1
fi

echo "Installing ${#pkgs[@]} binaries from local checkout: $repo_root"
printf '  - %s\n' "${pkgs[@]}"

for pkg in "${pkgs[@]}"; do
  # --path is the workspace root; -p selects the package (binary crate).
  cargo install --locked $force --path "$repo_root" -p "$pkg"
done

