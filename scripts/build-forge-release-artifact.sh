#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
build-forge-release-artifact.sh

Build and package the managed Forge binaries for one target triple.

Usage:
  scripts/build-forge-release-artifact.sh --version VERSION --target TARGET --output-dir DIR

Example:
  scripts/build-forge-release-artifact.sh \
    --version 20260415.0.1 \
    --target x86_64-apple-darwin \
    --output-dir dist
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

extract_bins() {
  installer="$1"
  sed -n '/^  # BEGIN FORGE_BINARIES$/,/^  # END FORGE_BINARIES$/p' "$installer" \
    | sed -e '1d' -e '$d' \
    | sed -e "/^  cat <<'EOF'$/d" -e '/^EOF$/d' \
    | sed -e 's/[[:space:]]*$//' -e '/^$/d' -e '/^#/d'
}

version=""
target=""
output_dir=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      version="${2:-}"
      shift 2
      ;;
    --target)
      target="${2:-}"
      shift 2
      ;;
    --output-dir)
      output_dir="${2:-}"
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
[[ -n "$target" ]] || die "--target is required"
[[ -n "$output_dir" ]] || die "--output-dir is required"

need_cmd cargo
need_cmd tar

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(cd "$script_dir/.." && pwd)"
installer="$root/scripts/install-forge-release.sh"
[[ -f "$installer" ]] || die "missing release installer: $installer"

mkdir -p "$output_dir"

current_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' "$root/crates/forge/Cargo.toml" | head -n 1)"
[[ "$current_version" == "$version" ]] || die "crate version $current_version does not match --version $version"

bins=()
while IFS= read -r bin; do
  [[ -n "$bin" ]] || continue
  bins+=("$bin")
done < <(extract_bins "$installer")

[[ "${#bins[@]}" -gt 0 ]] || die "no managed binaries found in $installer"

cargo_args=(build --manifest-path "$root/Cargo.toml" --release --locked --target "$target")
for bin in "${bins[@]}"; do
  cargo_args+=(-p "$bin" --bin "$bin")
done

cargo "${cargo_args[@]}"

target_dir="$root/target/$target/release"
stage_dir="$(mktemp -d)"
cleanup() {
  rm -rf "$stage_dir"
}
trap cleanup EXIT

for bin in "${bins[@]}"; do
  src="$target_dir/$bin"
  [[ -f "$src" ]] || die "missing built binary: $src"
  cp "$src" "$stage_dir/$bin"
done

asset_name="forge-${version}-${target}.tar.gz"
asset_path="$output_dir/$asset_name"
tar -C "$stage_dir" -czf "$asset_path" "${bins[@]}"

sha256="$(sha256_file "$asset_path")"
size_bytes="$(wc -c < "$asset_path" | tr -d '[:space:]')"

printf '%s  %s\n' "$sha256" "$asset_name" > "$output_dir/$asset_name.sha256"
cat > "$output_dir/$asset_name.metadata.json" <<EOF
{
  "target": "$target",
  "name": "$asset_name",
  "sha256": "$sha256",
  "size_bytes": $size_bytes
}
EOF

printf '%s\n' "$asset_path"
