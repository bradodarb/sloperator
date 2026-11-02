#!/usr/bin/env bash
# Write <file>.sha256 next to <file>, in the "<hash>  <name>" form that `shasum -a 256 -c` and `sha256sum -c` read.
# Usage: packaging/sha256.sh <file>
set -euo pipefail

file=$1
dir=$(dirname "$file") base=$(basename "$file")
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$dir" && sha256sum "$base" > "$base.sha256")
else
  (cd "$dir" && shasum -a 256 "$base" > "$base.sha256")
fi
