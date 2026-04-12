#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev.sh

Dev-only bootstrap for private development:
  - clones iancleary/forge via `gh` into /tmp
  - installs all workspace binary crates via `cargo install`

Usage:
  scripts/install-forge-dev.sh [--repo OWNER/REPO] [--ref REF] [--dir PATH]

Options:
  --repo   GitHub repo to clone (default: iancleary/forge)
  --ref    Git ref to check out (default: main)
  --dir    Clone directory (default: /tmp/forge)
  -h,--help  Show help
EOF
}

repo="iancleary/forge"
ref="main"
dir="/tmp/forge"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="${2:-}"
      shift 2
      ;;
    --ref)
      ref="${2:-}"
      shift 2
      ;;
    --dir)
      dir="${2:-}"
      shift 2
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

need_cmd gh
need_cmd git
need_cmd cargo
need_cmd python3

ensure_clone() {
  if [[ -d "$dir/.git" ]]; then
    local origin
    origin="$(git -C "$dir" remote get-url origin 2>/dev/null || true)"
    if [[ "$origin" != *"$repo"* ]]; then
      echo "error: $dir exists but origin does not look like $repo (origin=$origin)" >&2
      echo "hint: pass a different --dir or remove $dir" >&2
      exit 1
    fi
    git -C "$dir" fetch --tags origin
    return 0
  fi

  if [[ -e "$dir" ]]; then
    echo "error: $dir exists but is not a git repo" >&2
    echo "hint: pass a different --dir or remove $dir" >&2
    exit 1
  fi

  gh repo clone "$repo" "$dir"
  git -C "$dir" fetch --tags origin
}

checkout_ref() {
  # Works for branches, tags, and commit SHAs. For branches, pull the latest.
  git -C "$dir" checkout "$ref" >/dev/null 2>&1 || git -C "$dir" checkout --detach "$ref"
  local head
  head="$(git -C "$dir" rev-parse --abbrev-ref HEAD)"
  if [[ "$head" == "$ref" ]]; then
    git -C "$dir" pull --rebase origin "$ref"
  fi
}

bin_packages() {
  cargo -C "$dir" metadata --no-deps --format-version 1 \
    | python3 - <<'PY'
import json, sys
data = json.load(sys.stdin)
pkgs = []
for p in data.get("packages", []):
    has_bin = False
    for t in p.get("targets", []):
        if "bin" in t.get("kind", []):
            has_bin = True
            break
    if has_bin:
        pkgs.append(p["name"])
for name in pkgs:
    print(name)
PY
}

install_bins() {
  local pkgs
  mapfile -t pkgs < <(bin_packages)
  if [[ ${#pkgs[@]} -eq 0 ]]; then
    echo "error: no binary packages found in workspace" >&2
    exit 1
  fi

  echo "Installing ${#pkgs[@]} binaries from $repo@$ref into \$CARGO_HOME/bin:"
  printf '  - %s\n' "${pkgs[@]}"

  local pkg
  for pkg in "${pkgs[@]}"; do
    cargo install --locked --force --path "$dir" -p "$pkg"
  done
}

ensure_clone
checkout_ref
install_bins
