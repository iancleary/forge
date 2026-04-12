#!/bin/sh

set -eu

REPO_URL="https://github.com/iancleary/forge"
REPO_API_URL="https://api.github.com/repos/iancleary/forge"
REF="${FORGE_TAG:-}"
INSTALL_CODEX=1

usage() {
  cat <<'EOF'
Install Forge CLIs from the latest published release or a specific tagged release.

Usage:
  install-forge-release.sh [--tag <release-tag>] [--skip-codex]

Examples:
  install-forge-release.sh
  install-forge-release.sh --tag 20260412.0.4
EOF
}

resolve_latest_tag() {
  if ! command -v curl >/dev/null 2>&1; then
    echo "curl is required to resolve the latest Forge release tag automatically." >&2
    exit 1
  fi

  tag=$(
    curl -fsSL "${REPO_API_URL}/releases/latest" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      head -n 1
  )

  if [ -z "$tag" ]; then
    echo "failed to resolve the latest Forge release tag" >&2
    exit 1
  fi

  printf '%s\n' "$tag"
}

default_binaries() {
  # Single source of truth for which binaries the installer manages.
  # BEGIN FORGE_BINARIES
  cat <<'EOF'
forge
codex-threads
linear
slack-cli
EOF
  # END FORGE_BINARIES
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
  REF=$(resolve_latest_tag)
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required. Install Rust via https://rustup.rs first." >&2
  exit 1
fi

echo "Installing Forge CLIs from ${REPO_URL} @ ${REF}"

default_binaries | while IFS= read -r bin; do
  [ -n "$bin" ] || continue
  cargo install --git "$REPO_URL" --tag "$REF" --locked --force "$bin"
done

export PATH="$HOME/.cargo/bin:$PATH"

echo "Installing Forge-managed skills into ~/.agents/skills"
forge skills install --all --target user

if [ "$INSTALL_CODEX" -eq 1 ]; then
  echo "Installing Forge-managed Codex user config into ~/.codex"
  forge codex install
fi

echo "Forge install complete."
