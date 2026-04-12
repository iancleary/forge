#!/bin/sh

set -eu

REPO_URL="https://github.com/iancleary/forge"
REF="${FORGE_TAG:-}"
INSTALL_CODEX=1

usage() {
  cat <<'EOF'
Install Forge CLIs from a tagged release.

Usage:
  install-forge-release.sh --tag <release-tag> [--skip-codex]

Examples:
  install-forge-release.sh --tag 2026.411.2
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --tag)
      if [ "$#" -lt 2 ]; then
        echo "missing value for --tag" >&2
        exit 1
      fi
      REF="$2"
      shift 2
      ;;
    --skip-codex)
      INSTALL_CODEX=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [ -z "$REF" ]; then
  echo "--tag <release-tag> is required" >&2
  usage >&2
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required. Install Rust via https://rustup.rs first." >&2
  exit 1
fi

echo "Installing Forge CLIs from ${REPO_URL} @ ${REF}"

cargo install --git "$REPO_URL" --tag "$REF" --locked --force forge
cargo install --git "$REPO_URL" --tag "$REF" --locked --force codex-threads
cargo install --git "$REPO_URL" --tag "$REF" --locked --force linear
cargo install --git "$REPO_URL" --tag "$REF" --locked --force slack-cli

export PATH="$HOME/.cargo/bin:$PATH"

echo "Installing Forge-managed skills into ~/.agents/skills"
forge skills install --all --target user

if [ "$INSTALL_CODEX" -eq 1 ]; then
  echo "Installing Forge-managed Codex user config into ~/.codex"
  forge codex install
fi

echo "Forge install complete."
