#!/bin/sh
# Builds the signed TUF repository for one channel.
#
#   scripts/publish.sh <packages-dir> <old-repo-dir|-> <out-dir>
#
#   <packages-dir>  the .kpkg files to publish (the catalog is generated from them)
#   <old-repo-dir>  the live repository of this channel (with metadata/ and
#                   targets/), or - for the very first publish
#   <out-dir>       receives metadata/ and targets/, ready to serve
#
# Environment: ROOT_JSON, TARGETS_KEY, SNAPSHOT_KEY, TIMESTAMP_KEY (DER key files),
# and kpkg on PATH. Metadata versions only ever go up, which is what stops
# devices from accepting an older repository.
set -eu
pkgs=${1:?usage: publish.sh <packages-dir> <old-repo-dir|-> <out-dir>}
old=${2:?}
out=${3:?}
: "${ROOT_JSON:?}" "${TARGETS_KEY:?}" "${SNAPSHOT_KEY:?}" "${TIMESTAMP_KEY:?}"

kpkg catalog "$pkgs" -o "$pkgs/catalog.json"

keys="-k $TARGETS_KEY -k $SNAPSHOT_KEY -k $TIMESTAMP_KEY"
# the timestamp is re-signed daily, so it only needs to live a little longer
expiry="--targets-expires 'in 90 days' --snapshot-expires 'in 14 days' --timestamp-expires 'in 7 days'"

rm -rf "$out"
if [ "$old" = "-" ]; then
	eval tuftool create --root "$ROOT_JSON" $keys --add-targets "$pkgs" --outdir "$out" \
		$expiry --targets-version 1 --snapshot-version 1 --timestamp-version 1
else
	meta="$old/metadata"
	ts=$(jq .signed.version "$meta/timestamp.json")
	snap=$(jq '.signed.meta["snapshot.json"].version' "$meta/timestamp.json")
	tgt=$(jq '.signed.meta["targets.json"].version' "$meta/snapshot.json" 2>/dev/null || jq '.signed.meta["targets.json"].version' "$meta/$snap.snapshot.json")
	eval tuftool update --root "$ROOT_JSON" $keys --metadata-url "file://$(cd "$meta" && pwd)/" \
		--add-targets "$pkgs" --outdir "$out" $expiry \
		--targets-version $((tgt + 1)) --snapshot-version $((snap + 1)) --timestamp-version $((ts + 1))
fi
# tuftool links the targets to the files in <packages-dir>; a web server or a
# git branch needs the files themselves
rm -rf "$out.files"
cp -rL "$out" "$out.files"
rm -rf "$out"
mv "$out.files" "$out"
echo "published: $(jq .signed.version "$out/metadata/timestamp.json") timestamp, $(ls "$out/targets" | wc -l) target file(s)"
