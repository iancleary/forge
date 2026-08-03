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

expected_targets=(
  aarch64-apple-darwin
  x86_64-unknown-linux-gnu
  x86_64-pc-windows-msvc
)

for metadata_file in "${metadata_files[@]}"; do
  target_name="$(jq -r '.target // empty' "$metadata_file")"
  asset_name="$(jq -r '.name // empty' "$metadata_file")"
  archive_name="$(jq -r '.archive // empty' "$metadata_file")"
  sha256="$(jq -r '.sha256 // empty' "$metadata_file")"
  size_bytes="$(jq -r '.size_bytes // 0' "$metadata_file")"
  [[ -n "$target_name" ]] || die "artifact metadata is missing target: $metadata_file"
  [[ -n "$asset_name" ]] || die "artifact metadata is missing name: $metadata_file"
  [[ "$sha256" =~ ^[0-9a-fA-F]{64}$ ]] || die "artifact metadata has invalid sha256: $metadata_file"
  [[ "$size_bytes" =~ ^[1-9][0-9]*$ ]] || die "artifact metadata has invalid size_bytes: $metadata_file"
  case "$target_name" in
    aarch64-apple-darwin|x86_64-unknown-linux-gnu)
      [[ "$archive_name" == "tar.gz" ]] || die "$target_name must use tar.gz"
      expected_name="forge-${version}-${target_name}.tar.gz"
      ;;
    x86_64-pc-windows-msvc)
      [[ "$archive_name" == "zip" ]] || die "$target_name must use zip"
      expected_name="forge-${version}-${target_name}.zip"
      ;;
    *) die "unexpected artifact target in metadata: $target_name" ;;
  esac
  [[ "$asset_name" == "$expected_name" ]] || die "unexpected artifact name: $asset_name"
  artifact_path="$artifacts_dir/$asset_name"
  [[ -f "$artifact_path" ]] || die "missing artifact for metadata: $asset_name"
  actual_sha256="$(sha256_file "$artifact_path")"
  actual_sha256="$(printf '%s' "$actual_sha256" | tr '[:upper:]' '[:lower:]')"
  expected_sha256="$(printf '%s' "$sha256" | tr '[:upper:]' '[:lower:]')"
  [[ "$actual_sha256" == "$expected_sha256" ]] ||
    die "artifact checksum does not match metadata: $asset_name"
  actual_size_bytes="$(wc -c < "$artifact_path" | tr -d '[:space:]')"
  [[ "$actual_size_bytes" == "$size_bytes" ]] ||
    die "artifact size does not match metadata: $asset_name"
done

[[ "${#metadata_files[@]}" -eq "${#expected_targets[@]}" ]] ||
  die "expected exactly ${#expected_targets[@]} artifact metadata files"

for target_name in "${expected_targets[@]}"; do
  count=0
  for metadata_file in "${metadata_files[@]}"; do
    if [[ "$(jq -r '.target' "$metadata_file")" == "$target_name" ]]; then
      count=$((count + 1))
    fi
  done
  [[ "$count" -eq 1 ]] || die "expected exactly one artifact metadata file for $target_name"
done

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
for target_name in "${expected_targets[@]}"; do
  metadata_file=""
  for candidate in "${metadata_files[@]}"; do
    if [[ "$(jq -r '.target' "$candidate")" == "$target_name" ]]; then
      metadata_file="$candidate"
      break
    fi
  done
  asset_name="$(jq -r '.name' "$metadata_file")"
  sha256="$(jq -r '.sha256' "$metadata_file")"
  printf '%s  %s\n' "$sha256" "$asset_name" >> "$sha256sums_path"
done

rm -f "${metadata_files[@]}"

printf '%s\n' "$manifest_path"
