#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev-repo.sh

Dev-only bootstrap for private development:
  - clones iancleary/forge via `gh` into /tmp
  - delegates to `install-forge-dev-local.sh` to install all workspace binary crates

Usage:
  scripts/install-forge-dev-repo.sh [--repo OWNER/REPO] [--ref REF] [--dir PATH] [--no-force]

Options:
  --repo   GitHub repo to clone (default: iancleary/forge)
  --ref    Git ref to check out (default: main)
  --dir    Clone directory (default: /tmp/forge)
  --no-force  Do not pass --force to cargo install
  -h,--help  Show help
EOF
}

repo="iancleary/forge"
ref="main"
dir="/tmp/forge"
force_flag=""

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
    --no-force)
      force_flag="--no-force"
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

need_cmd gh
need_cmd git

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
local_installer="$script_dir/install-forge-dev-local.sh"
if [[ ! -x "$local_installer" ]]; then
  echo "error: expected executable installer: $local_installer" >&2
  exit 1
fi

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

ensure_clone
checkout_ref
"$local_installer" --path "$dir" $force_flag
