#!/bin/sh

set -eu

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <version>" >&2
  echo "expected version format: YYYYMMDD.0.N" >&2
  exit 1
fi

version="$1"
case "$version" in
  [0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9].0.[0-9]*)
    ;;
  *)
    echo "error: version must match YYYYMMDD.0.N, got '$version'" >&2
    exit 1
    ;;
esac

root="$(cd "$(dirname "$0")/.." && pwd)"
updated=0

for manifest in "$root"/crates/*/Cargo.toml; do
  [ -f "$manifest" ] || continue

  tmp="$(mktemp)"
  awk -v version="$version" '
    BEGIN { in_package = 0; version_set = 0 }
    /^\[package\]$/ {
      in_package = 1
      version_set = 0
      print
      next
    }
    /^\[[^][]+\]$/ {
      in_package = 0
    }
    {
      if (in_package && $0 ~ /^version[[:space:]]*=[[:space:]]*"/ && version_set == 0) {
        print "version = \"" version "\""
        version_set = 1
        next
      }
      print
    }
  ' "$manifest" > "$tmp" && mv "$tmp" "$manifest"

  updated=1
done

if [ "$updated" -eq 0 ]; then
  echo "warning: no Cargo.toml files updated" >&2
  exit 1
fi

echo "updated crate versions to $version"
