#!/usr/bin/env bash
# Package one release binary as sloperator-<version>-<target>.tar.gz plus its .sha256 file.
# Usage: packaging/package.sh <version> <target> <binary> <out-dir>
# Run from the repository root: the tarball carries LICENSE and README.md from there.
set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: packaging/package.sh <version> <target> <binary> <out-dir>" >&2
  exit 2
fi
version=$1 target=$2 binary=$3 out=$4
name="sloperator-${version}-${target}"

[ -x "$binary" ] || { echo "package.sh: no executable at $binary" >&2; exit 1; }
[ -f LICENSE ] && [ -f README.md ] || { echo "package.sh: run from the repository root" >&2; exit 1; }

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir "$stage/$name"
cp "$binary" "$stage/$name/sloperator"
cp LICENSE README.md "$stage/$name/"

mkdir -p "$out"
COPYFILE_DISABLE=1 tar -C "$stage" -czf "$out/$name.tar.gz" "$name"
packaging/sha256.sh "$out/$name.tar.gz"
echo "packaged $out/$name.tar.gz"
