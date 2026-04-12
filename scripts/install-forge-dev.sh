#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
install-forge-dev.sh

Dev-only convenience wrapper.

Usage:
  scripts/install-forge-dev.sh local [--path PATH] [--no-force]
  scripts/install-forge-dev.sh repo  [--repo OWNER/REPO] [--ref REF] [--dir PATH] [--no-force]

Examples:
  scripts/install-forge-dev.sh local
  scripts/install-forge-dev.sh repo --ref main
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 2
fi

mode="$1"
shift 1

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

case "$mode" in
  local)
    exec "$script_dir/install-forge-dev-local.sh" "$@"
    ;;
  repo)
    exec "$script_dir/install-forge-dev-repo.sh" "$@"
    ;;
  -h|--help|help)
    usage
    exit 0
    ;;
  *)
    echo "error: unknown mode: $mode" >&2
    usage >&2
    exit 2
    ;;
esac

