#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
build-forge-release-manifest.sh

Build the release manifest and aggregate SHA256SUMS for previously built Forge release artifacts.

Usage:
  scripts/build-forge-release-manifest.sh --version VERSION --artifacts-dir DIR
EOF
}

die() {
  echo "error: $*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
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

version=""
artifacts_dir=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:-}"
      shift 2
      ;;
    --artifacts-dir)
      artifacts_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown arg: $1"
      ;;
  esac
done

[[ -n "$version" ]] || die "--version is required"
[[ -n "$artifacts_dir" ]] || die "--artifacts-dir is required"
[[ -d "$artifacts_dir" ]] || die "artifacts dir not found: $artifacts_dir"

need_cmd git
need_cmd rustc
need_cmd jq

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$script_dir/.." && pwd)"

metadata_files=("$artifacts_dir"/*.metadata.json)
[[ -e "${metadata_files[0]}" ]] || die "no artifact metadata files found in $artifacts_dir"

source_commit="$(git -C "$root" rev-parse HEAD)"
cargo_lock_sha256="$(sha256_file "$root/Cargo.lock")"
rust_toolchain="$(rustc --version)"

manifest_path="$artifacts_dir/forge-release-manifest.json"
sha256sums_path="$artifacts_dir/forge-release-sha256sums.txt"

jq -s \
  --arg version "$version" \
  --arg source_commit "$source_commit" \
  --arg cargo_lock_sha256 "$cargo_lock_sha256" \
  --arg rust_toolchain "$rust_toolchain" \
  '
    {
      version: $version,
      source_commit: $source_commit,
      cargo_lock_sha256: $cargo_lock_sha256,
      rust_toolchain: $rust_toolchain,
      artifacts: (sort_by(.target))
    }
  ' \
  "${metadata_files[@]}" > "$manifest_path"

: > "$sha256sums_path"
for metadata_file in "${metadata_files[@]}"; do
  asset_name="$(jq -r '.name' "$metadata_file")"
  sha256="$(jq -r '.sha256' "$metadata_file")"
  printf '%s  %s\n' "$sha256" "$asset_name" >> "$sha256sums_path"
done

rm -f "${metadata_files[@]}"

printf '%s\n' "$manifest_path"
