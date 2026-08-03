#!/bin/sh

set -eu

REPO_URL="https://github.com/iancleary/forge"
REPO_API_URL="https://api.github.com/repos/iancleary/forge"
RAW_REPO_URL="https://raw.githubusercontent.com/iancleary/forge"
RELEASE_DOWNLOAD_URL="${REPO_URL}/releases/download"
REF="${FORGE_TAG:-}"
INSTALL_CODEX=1
BUILD_FROM_SOURCE=0

usage() {
  cat <<'EOF'
Install Forge CLIs from the latest published release or a specific tagged release.

The default deployment path downloads the platform archive and published
SHA-256 manifest from the same selected GitHub release tag. It does not
require a Rust toolchain. Source builds are explicit development or recovery
operations.

Usage:
  install-forge-release.sh [--tag <release-tag>] [--skip-codex] [--build-from-source]

Examples:
  install-forge-release.sh
  install-forge-release.sh --tag 20260412.0.7
  install-forge-release.sh --tag 20260412.0.7 --build-from-source
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

resolve_latest_tag() {
  need_cmd curl

  tag=$(
    curl -fsSL "${REPO_API_URL}/releases/latest" |
      sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' |
      head -n 1
  )

  [ -n "$tag" ] || die "failed to resolve the latest Forge release tag"
  printf '%s\n' "$tag"
}

validate_release_tag() {
  printf '%s\n' "$REF" | grep -Eq '^[0-9]{8}\.0\.[0-9]+$' ||
    die "invalid Forge release tag: $REF"
}

default_binaries() {
  # BEGIN FORGE_BINARIES
  cat <<'EOF'
forge
codex-threads
linear
mermaid
slack-agent
slack-query
EOF
  # END FORGE_BINARIES
}

cargo_bin_dir() {
  if [ -n "${CARGO_HOME:-}" ]; then
    printf '%s/bin\n' "$CARGO_HOME"
    return
  fi
  [ -n "${HOME:-}" ] || die "HOME is not set"
  printf '%s/.cargo/bin\n' "$HOME"
}

detect_target() {
  os="$(uname -s 2>/dev/null || printf unknown)"
  arch="$(uname -m 2>/dev/null || printf unknown)"
  case "$os/$arch" in
    Darwin/x86_64) printf '%s\n' x86_64-apple-darwin ;;
    Darwin/arm64|Darwin/aarch64) printf '%s\n' aarch64-apple-darwin ;;
    Linux/x86_64) printf '%s\n' x86_64-unknown-linux-gnu ;;
    *) return 1 ;;
  esac
}

sha256_file() {
  file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$file" | awk '{print $1}'
    return
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$file" | awk '{print $1}'
    return
  fi
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 -r "$file" | awk '{print $1}'
    return
  fi
  die "missing SHA-256 tool (sha256sum, shasum, or openssl)"
}

artifact_install_unavailable_reason() {
  if ! command -v curl >/dev/null 2>&1; then
    printf '%s\n' "release artifact download requires curl"
    return 0
  fi
  if ! command -v tar >/dev/null 2>&1; then
    printf '%s\n' "release artifact extraction requires tar"
    return 0
  fi
  if ! detect_target >/dev/null 2>&1; then
    printf '%s\n' "no attested release artifact is published for this platform"
    return 0
  fi
  return 1
}

handoff_to_tagged_installer() {
  [ "${FORGE_INSTALLER_PINNED:-0}" = "1" ] && return 0
  command -v curl >/dev/null 2>&1 || return 0

  tmp_dir="$(mktemp -d)"
  installer_path="$tmp_dir/install-forge-release.sh"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

  curl -fsSL "${RAW_REPO_URL}/${REF}/scripts/install-forge-release.sh" -o "$installer_path" || return 0
  chmod +x "$installer_path"

  set --
  [ -n "$REF" ] && set -- "$@" --tag "$REF"
  [ "$INSTALL_CODEX" -eq 0 ] && set -- "$@" --skip-codex
  [ "$BUILD_FROM_SOURCE" -eq 1 ] && set -- "$@" --build-from-source
  FORGE_INSTALLER_PINNED=1 FORGE_TAG="$REF" \
    exec "$installer_path" "$@"
}

install_from_artifact() (
  set -eu

  command -v curl >/dev/null 2>&1 || return 2
  command -v tar >/dev/null 2>&1 || return 2

  target="$(detect_target)" || return 2
  asset_name="forge-${REF}-${target}.tar.gz"

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

  sha256sums_path="$tmp_dir/forge-release-sha256sums.txt"
  if ! curl -fsSL "${RELEASE_DOWNLOAD_URL}/${REF}/forge-release-sha256sums.txt" -o "$sha256sums_path"; then
    return 2
  fi

  expected_sha="$(grep "  ${asset_name}\$" "$sha256sums_path" | awk '{print $1}' | head -n 1)"
  [ -n "$expected_sha" ] || return 2
  printf '%s\n' "$expected_sha" | grep -Eq '^[0-9a-fA-F]{64}$' ||
    die "invalid SHA-256 entry for $asset_name"
  archive_path="$tmp_dir/$asset_name"
  curl -fsSL "${RELEASE_DOWNLOAD_URL}/${REF}/${asset_name}" -o "$archive_path" ||
    die "failed to download release artifact: $asset_name"

  actual_sha="$(sha256_file "$archive_path")"
  [ "$actual_sha" = "$expected_sha" ] ||
    die "checksum mismatch for $asset_name: expected $expected_sha, got $actual_sha"

  entries="$(tar -tzf "$archive_path")"
  for entry in $entries; do
    case "$entry" in
      forge|codex-threads|linear|mermaid|slack-agent|slack-query)
        ;;
      *)
        die "release artifact contains unexpected entry: $entry"
        ;;
    esac
  done

  extract_dir="$tmp_dir/extract"
  mkdir -p "$extract_dir"
  tar -xzf "$archive_path" -C "$extract_dir"

  cargo_bin="$(cargo_bin_dir)"
  mkdir -p "$cargo_bin"
  for bin in $(default_binaries); do
    src="$extract_dir/$bin"
    [ -f "$src" ] || die "attested release artifact is missing binary: $bin"
    cp "$src" "$cargo_bin/$bin"
    chmod 755 "$cargo_bin/$bin" 2>/dev/null || true
  done
)

install_from_source() (
  set -eu

  need_cmd cargo
  need_cmd git

  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

  repo_dir="$tmp_dir/repo"
  git clone --depth 1 --branch "$REF" "$REPO_URL" "$repo_dir" >/dev/null 2>&1 ||
    die "failed to clone ${REPO_URL} at tag ${REF}"

  set -- build --release --locked
  for bin in $(default_binaries); do
    set -- "$@" -p "$bin" --bin "$bin"
  done
  (
    cd "$repo_dir"
    cargo "$@"
  )

  cargo_bin="$(cargo_bin_dir)"
  mkdir -p "$cargo_bin"
  for bin in $(default_binaries); do
    src="$repo_dir/target/release/$bin"
    [ -f "$src" ] || die "source build did not produce binary: $bin"
    cp "$src" "$cargo_bin/$bin"
    chmod 755 "$cargo_bin/$bin" 2>/dev/null || true
  done
)

while [ "$#" -gt 0 ]; do
  case "$1" in
    --tag)
      [ "$#" -ge 2 ] || die "missing value for --tag"
      REF="$2"
      shift 2
      ;;
    --skip-codex)
      INSTALL_CODEX=0
      shift
      ;;
    --build-from-source)
      BUILD_FROM_SOURCE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown argument: $1"
      ;;
  esac
done

if [ -z "$REF" ]; then
  REF="$(resolve_latest_tag)"
fi
validate_release_tag

handoff_to_tagged_installer

echo "Installing Forge CLIs from ${REPO_URL} @ ${REF}"

if [ "$BUILD_FROM_SOURCE" -eq 0 ]; then
  artifact_reason="$(artifact_install_unavailable_reason || true)"
  if install_from_artifact; then
    echo "Installed checksum-verified release artifacts for ${REF}"
  else
    status=$?
    if [ "$status" -ne 2 ]; then
      exit "$status"
    fi
    if [ -n "$artifact_reason" ]; then
      die "${artifact_reason}; no binary installation was performed (use --build-from-source explicitly for development or recovery)"
    else
      die "checksum-verified release artifact install failed; no binary installation was performed (use --build-from-source explicitly for development or recovery)"
    fi
  fi
else
  install_from_source
fi

export PATH="$(cargo_bin_dir):$PATH"

echo "Installing Forge-managed skills into ~/.agents/skills"
forge skills install --all --target user

if [ "$INSTALL_CODEX" -eq 1 ]; then
  echo "Installing Forge-managed Codex user config into ~/.codex"
  forge codex install
fi

echo "Forge install complete."
