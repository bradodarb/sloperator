#!/usr/bin/env bash
# Print the body of the CHANGELOG.md section for one version (the lines under "## <version>", up to the next "## ").
# Usage: packaging/changelog-section.sh <version>
# Fails when the section is missing or empty, so a release never ships without notes.
set -euo pipefail

version=$1
notes=$(awk -v v="$version" '
  /^## / { if (found) exit; if ($2 == v) { found = 1; next } }
  found { print }
' CHANGELOG.md)
if [ -z "$(echo "$notes" | tr -d '[:space:]')" ]; then
  echo "changelog-section.sh: no CHANGELOG.md section for $version" >&2
  exit 1
fi
echo "$notes"
