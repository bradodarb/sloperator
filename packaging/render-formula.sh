#!/usr/bin/env bash
# Render the Homebrew formula from packaging/sloperator.rb.tmpl to stdout.
# Usage: packaging/render-formula.sh <version> <dist-dir>
# <dist-dir> holds the .tar.gz.sha256 files that packaging/package.sh wrote for every formula target.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: packaging/render-formula.sh <version> <dist-dir>" >&2
  exit 2
fi
version=$1 dist=$2

sha() {
  local file="$dist/sloperator-${version}-$1.tar.gz.sha256"
  [ -f "$file" ] || { echo "render-formula.sh: missing $file" >&2; exit 1; }
  local hash
  hash=$(cut -d ' ' -f 1 "$file")
  [[ "$hash" =~ ^[0-9a-f]{64}$ ]] || { echo "render-formula.sh: bad sha256 in $file" >&2; exit 1; }
  echo "$hash"
}

mac_arm=$(sha aarch64-apple-darwin)
linux_arm=$(sha aarch64-unknown-linux-gnu)
linux_intel=$(sha x86_64-unknown-linux-gnu)

sed -e "s/@VERSION@/${version}/g" \
    -e "s/@SHA_AARCH64_APPLE_DARWIN@/${mac_arm}/" \
    -e "s/@SHA_AARCH64_UNKNOWN_LINUX_GNU@/${linux_arm}/" \
    -e "s/@SHA_X86_64_UNKNOWN_LINUX_GNU@/${linux_intel}/" \
    "$(dirname "$0")/sloperator.rb.tmpl"
