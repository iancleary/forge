#!/bin/sh

set -eu
umask 022

REPO_SLUG="iancleary/forge"
REPO_URL="https://github.com/${REPO_SLUG}"
REPO_API_URL="https://api.github.com/repos/${REPO_SLUG}"
RAW_REPO_URL="https://raw.githubusercontent.com/${REPO_SLUG}"
RELEASE_DOWNLOAD_URL="${REPO_URL}/releases/download"
REF="${FORGE_TAG:-}"
INSTALL_CODEX=1
VERIFY_ATTESTATION=0
BUILD_FROM_SOURCE=0

usage() {
  cat <<'EOF'
Install Forge CLIs from a published release.

The default path downloads the selected release checksum manifest and archive
over HTTPS, verifies SHA-256, validates the complete archive, and atomically
installs the binaries. It does not require Rust, Cargo, Git, or GitHub CLI.

Usage:
  install-forge-release.sh [--tag <version>] [--skip-codex]
  install-forge-release.sh --verify-attestation [--tag <version>]
  install-forge-release.sh --build-from-source [--tag <version>]

Options:
  --tag <version>          Forge CalVer release tag
  --skip-codex             Do not install the managed Codex user config
  --verify-attestation     Explicitly verify GitHub provenance with gh
  --build-from-source      Explicitly build the tagged source with Cargo
  -h, --help               Show this help
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
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

download_file() {
  url="$1"
  destination="$2"
  case "$url" in
    "${REPO_API_URL}/releases/latest"|\
    "${RAW_REPO_URL}/${REF}/scripts/install-forge-release.sh"|\
    "${RELEASE_DOWNLOAD_URL}/${REF}/forge-release-sha256sums.txt"|\
    "${RELEASE_DOWNLOAD_URL}/${REF}/forge-${REF}-"*)
      ;;
    *) die "refusing download from unexpected URL: $url" ;;
  esac
  curl \
    --fail \
    --show-error \
    --location \
    --proto '=https' \
    --tlsv1.2 \
    --retry 3 \
    --retry-all-errors \
    --output "$destination" \
    "$url"
}

resolve_latest_tag() {
  need_cmd curl
  latest_path="$(mktemp)"
  trap 'rm -f "$latest_path"' EXIT HUP INT TERM
  curl \
    --fail \
    --show-error \
    --location \
    --proto '=https' \
    --tlsv1.2 \
    --retry 3 \
    --retry-all-errors \
    --output "$latest_path" \
    "${REPO_API_URL}/releases/latest"
  tag="$(sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$latest_path" | head -n 1)"
  rm -f "$latest_path"
  trap - EXIT HUP INT TERM
  [ -n "$tag" ] || die "failed to resolve the latest Forge release tag"
  printf '%s\n' "$tag"
}

handoff_to_tagged_installer() {
  [ "${FORGE_INSTALLER_PINNED:-0}" = "1" ] && return 0
  command -v curl >/dev/null 2>&1 || return 0

  tmp_dir="$(mktemp -d)"
  installer_path="$tmp_dir/install-forge-release.sh"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
  download_file "${RAW_REPO_URL}/${REF}/scripts/install-forge-release.sh" "$installer_path" ||
    die "failed to download the installer for validated release tag $REF"
  chmod 755 "$installer_path"

  set -- --tag "$REF"
  [ "$INSTALL_CODEX" -eq 0 ] && set -- "$@" --skip-codex
  [ "$VERIFY_ATTESTATION" -eq 1 ] && set -- "$@" --verify-attestation
  [ "$BUILD_FROM_SOURCE" -eq 1 ] && set -- "$@" --build-from-source
  FORGE_INSTALLER_PINNED=1 FORGE_TAG="$REF" \
    exec "$installer_path" "$@"
}

validate_checksum_manifest() {
  manifest_path="$1"
  asset_name="$2"
  expected_sha=""
  matching_entries=0
  seen_names=""

  while IFS= read -r line || [ -n "$line" ]; do
    [ -n "$line" ] || continue
    printf '%s\n' "$line" | grep -Eq '^[0-9a-fA-F]{64}  [^[:space:]]+$' ||
      die "malformed checksum manifest entry"
    sha="${line%%  *}"
    name="${line#*  }"
    printf '%s\n' "$seen_names" | grep -Fqx "$name" &&
      die "duplicate checksum manifest entry: $name"
    seen_names="${seen_names}
${name}"
    if [ "$name" = "$asset_name" ]; then
      matching_entries=$((matching_entries + 1))
      expected_sha="$(printf '%s' "$sha" | tr '[:upper:]' '[:lower:]')"
    fi
  done < "$manifest_path"

  [ "$matching_entries" -eq 1 ] ||
    die "expected exactly one checksum entry for $asset_name"
  printf '%s\n' "$expected_sha"
}

verify_attestation() {
  archive_path="$1"
  need_cmd gh
  gh attestation verify "$archive_path" \
    --repo "$REPO_SLUG" \
    --source-ref "refs/tags/$REF" \
    --signer-workflow "$REPO_SLUG/.github/workflows/release-artifacts.yml" \
    --predicate-type 'https://slsa.dev/provenance/v1' ||
    die "explicit GitHub attestation verification failed for $archive_path"
}

check_destination() {
  destination="$1"
  if [ -L "$destination" ]; then
    die "refusing symlink destination directory: $destination"
  fi
  if [ -e "$destination" ] && [ ! -d "$destination" ]; then
    die "destination is not a directory: $destination"
  fi
  mkdir -p "$destination"

  if command -v id >/dev/null 2>&1 && command -v stat >/dev/null 2>&1; then
    uid="$(id -u 2>/dev/null || true)"
    owner="$(stat -c '%u' "$destination" 2>/dev/null || stat -f '%u' "$destination" 2>/dev/null || true)"
    if [ -n "$uid" ] && [ -n "$owner" ] && [ "$uid" != "$owner" ]; then
      die "destination is not owned by the current user: $destination"
    fi
  fi
}

validate_extracted_binary() {
  path="$1"
  [ ! -L "$path" ] || die "extracted binary is a symbolic link: $path"
  [ -f "$path" ] || die "extracted binary is not a regular file: $path"
}

atomic_install_from_dir() (
  source_dir="$1"
  destination="$(cargo_bin_dir)"
  check_destination "$destination"

  stage_dir="$(mktemp -d "$destination/.forge-stage.XXXXXX")"
  backup_dir="$(mktemp -d "$destination/.forge-backup.XXXXXX")"
  moved=""
  installed=""

  rollback() {
    status=$?
    trap - EXIT HUP INT TERM
    set +e
    rollback_failed=0
    for binary in $installed; do
      rm -f "$destination/$binary" || rollback_failed=1
    done
    for binary in $moved; do
      mv "$backup_dir/$binary" "$destination/$binary" || rollback_failed=1
    done
    rm -rf "$stage_dir" "$backup_dir" || rollback_failed=1
    if [ "$rollback_failed" -ne 0 ]; then
      echo "error: replacement failed and rollback failed; recovery files remain in $backup_dir" >&2
      exit 1
    fi
    exit "$status"
  }
  trap rollback EXIT HUP INT TERM

  for binary in $(default_binaries); do
    source_path="$source_dir/$binary"
    validate_extracted_binary "$source_path"
    cp "$source_path" "$stage_dir/$binary"
    chmod 755 "$stage_dir/$binary"
    validate_extracted_binary "$stage_dir/$binary"
  done

  replace_count=0
  for binary in $(default_binaries); do
    destination_path="$destination/$binary"
    if [ -L "$destination_path" ]; then
      die "refusing symlink binary destination: $destination_path"
    fi
    if [ -e "$destination_path" ] && [ ! -f "$destination_path" ]; then
      die "existing binary destination is not a regular file: $destination_path"
    fi
    if [ -e "$destination_path" ]; then
      mv "$destination_path" "$backup_dir/$binary"
      moved="$moved $binary"
    fi
  done

  for binary in $(default_binaries); do
    mv "$stage_dir/$binary" "$destination/$binary"
    installed="$installed $binary"
    replace_count=$((replace_count + 1))
    if [ "${FORGE_TEST_FAIL_REPLACEMENT_AFTER:-0}" -eq "$replace_count" ]; then
      die "test replacement failure"
    fi
  done

  trap - EXIT HUP INT TERM
  rm -rf "$stage_dir" "$backup_dir"
)

extract_and_install_artifact() (
  archive_path="$1"
  asset_name="$2"
  tmp_dir="$3"
  extract_dir="$tmp_dir/extract"
  mkdir -p "$extract_dir"

  listing="$(tar -tzf "$archive_path")" || die "failed to read release archive: $asset_name"
  entry_count="$(printf '%s\n' "$listing" | sed '/^$/d' | wc -l | tr -d '[:space:]')"
  [ "$entry_count" -eq 6 ] || die "release archive must contain exactly six entries"
  for entry in $listing; do
    case "$entry" in
      forge|codex-threads|linear|mermaid|slack-agent|slack-query) ;;
      /*|../*|*/../*|*/..|.|..) die "release archive contains unsafe path: $entry" ;;
      *) die "release archive contains unexpected entry: $entry" ;;
    esac
  done
  for expected in $(default_binaries); do
    count="$(printf '%s\n' "$listing" | grep -Fxc "$expected" || true)"
    [ "$count" -eq 1 ] || die "release archive is missing or duplicates binary: $expected"
  done
  tar -tvzf "$archive_path" | awk 'substr($1, 1, 1) != "-" { exit 1 }' ||
    die "release archive contains a non-regular entry"
  tar -xzf "$archive_path" -C "$extract_dir" ||
    die "failed to extract release archive: $asset_name"
  for binary in $(default_binaries); do
    validate_extracted_binary "$extract_dir/$binary"
  done
  atomic_install_from_dir "$extract_dir"
)

install_from_artifact() (
  need_cmd curl
  need_cmd tar
  target="$(detect_target)" || die "no Forge release artifact is published for this platform"
  asset_name="forge-${REF}-${target}.tar.gz"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM

  manifest_path="$tmp_dir/forge-release-sha256sums.txt"
  download_file "${RELEASE_DOWNLOAD_URL}/${REF}/forge-release-sha256sums.txt" "$manifest_path"
  expected_sha="$(validate_checksum_manifest "$manifest_path" "$asset_name")"
  archive_path="$tmp_dir/$asset_name"
  download_file "${RELEASE_DOWNLOAD_URL}/${REF}/$asset_name" "$archive_path"
  actual_sha="$(sha256_file "$archive_path")"
  actual_sha="$(printf '%s' "$actual_sha" | tr '[:upper:]' '[:lower:]')"
  [ "$actual_sha" = "$expected_sha" ] ||
    die "checksum mismatch for $asset_name: expected $expected_sha, got $actual_sha"
  [ "$VERIFY_ATTESTATION" -eq 0 ] || verify_attestation "$archive_path"
  extract_and_install_artifact "$archive_path" "$asset_name" "$tmp_dir"
)

install_from_source() (
  need_cmd cargo
  need_cmd git
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
  repo_dir="$tmp_dir/repo"
  git clone --depth 1 --branch "$REF" "$REPO_URL" "$repo_dir" >/dev/null 2>&1 ||
    die "failed to clone ${REPO_URL} at tag ${REF}"
  set -- build --release --locked
  for binary in $(default_binaries); do
    set -- "$@" -p "$binary" --bin "$binary"
  done
  (cd "$repo_dir" && cargo "$@") || die "tagged source build failed"
  build_dir="$repo_dir/target/release"
  for binary in $(default_binaries); do
    validate_extracted_binary "$build_dir/$binary"
  done
  atomic_install_from_dir "$build_dir"
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
    --verify-attestation)
      VERIFY_ATTESTATION=1
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
    *) die "unknown argument: $1" ;;
  esac
done

[ "$VERIFY_ATTESTATION" -eq 0 ] || [ "$BUILD_FROM_SOURCE" -eq 0 ] ||
  die "--verify-attestation cannot be combined with --build-from-source"

if [ -z "$REF" ]; then
  REF="$(resolve_latest_tag)"
fi
validate_release_tag
handoff_to_tagged_installer

echo "Installing Forge CLIs from ${REPO_URL} @ ${REF}"
if [ "$BUILD_FROM_SOURCE" -eq 1 ]; then
  install_from_source
else
  install_from_artifact
fi

forge_bin="$(cargo_bin_dir)/forge"
export PATH="$(cargo_bin_dir):$PATH"
echo "Installing Forge-managed skills into ~/.agents/skills"
"$forge_bin" skills install --all --target user
if [ "$INSTALL_CODEX" -eq 1 ]; then
  echo "Installing Forge-managed Codex user config into ~/.codex"
  "$forge_bin" codex install
fi
echo "Forge install complete."
