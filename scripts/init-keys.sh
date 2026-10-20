#!/bin/sh
# One-time key ceremony. Run it offline, on a machine you trust.
#
#   scripts/init-keys.sh <key-dir> <root.json>
#
# Creates four Ed25519 keys (root, targets, snapshot, timestamp) and a root.json
# that lists their public halves and is signed by the root key. Then:
#
#   - copy <root.json> to package/kioskd/root.json in the pikiosk repo and build
#     a new image: devices trust exactly this file;
#   - keep root.der offline (it is only needed to rotate keys);
#   - store targets.der, snapshot.der and timestamp.der as GitHub Actions
#     secrets (base64) for the publish workflow.
#
# Needs: openssl, tuftool (cargo install tuftool --locked).
set -eu
keys=${1:?usage: init-keys.sh <key-dir> <root.json>}
root=${2:?usage: init-keys.sh <key-dir> <root.json>}

mkdir -p "$keys"
chmod 700 "$keys"
for role in root targets snapshot timestamp; do
	if [ -e "$keys/$role.der" ]; then
		echo "keeping existing $keys/$role.der"
	else
		# tuftool reads an Ed25519 key as a PKCS#8 DER file
		openssl genpkey -algorithm ed25519 -outform DER -out "$keys/$role.der"
		chmod 600 "$keys/$role.der"
	fi
done

rm -f "$root"
tuftool root init "$root"
tuftool root expire "$root" 'in 1825 days'
for role in root targets snapshot timestamp; do
	tuftool root set-threshold "$root" "$role" 1
	tuftool root add-key "$root" -k "$keys/$role.der" --role "$role"
done
tuftool root sign "$root" -k "$keys/root.der"
echo "wrote $root"
