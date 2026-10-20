#!/bin/sh
# Copies the packages of a published repository into a plain directory,
# without the hash prefix TUF puts on target file names. This is how the
# next publish keeps the apps that are already in the store, and how a
# release is promoted from beta to stable.
#
#   scripts/live-packages.sh <repo-dir> <out-dir>
set -eu
repo=${1:?usage: live-packages.sh <repo-dir> <out-dir>}
out=${2:?}
mkdir -p "$out"
for f in "$repo"/targets/*.kpkg; do
	[ -e "$f" ] || continue
	cp "$f" "$out/$(basename "$f" | sed 's/^[0-9a-f]\{64\}\.//')"
done
