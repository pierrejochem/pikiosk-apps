#!/bin/sh
# Builds one app for the Raspberry Pi and packs it as a .kpkg.
#
#   scripts/build-app.sh <app-dir> <sdk-dir> <out-dir>
#
# <app-dir>   apps/<name>: a Cargo crate with package/manifest.toml (+ assets)
# <sdk-dir>   the unpacked pikiosk SDK (the tarball attached to a pikiosk
#             release, after running its relocate-sdk.sh)
# <out-dir>   receives <id>-<version>.kpkg
#
# The SDK's sysroot has the libraries kiosk-kit links against (glibc, libinput,
# libudev, Mesa GBM/EGL), so one aarch64 binary runs on the Pi 3B, 4 and 5.
# Needs kpkg on PATH and the aarch64 Rust target (rustup target add
# aarch64-unknown-linux-gnu).
set -eu
app=${1:?usage: build-app.sh <app-dir> <sdk-dir> <out-dir>}
sdk=$(cd "${2:?}" && pwd)
out=${3:?}
app=$(cd "$app" && pwd)
mkdir -p "$out"
name=$(sed -n 's/^name *= *"\(.*\)"/\1/p' "$app/Cargo.toml" | head -1)

triple=aarch64-unknown-linux-gnu
cross=$(ls "$sdk"/bin/*-linux-gnu-gcc | head -1)
sysroot=$(ls -d "$sdk"/*/sysroot)
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="$cross"
export CC_aarch64_unknown_linux_gnu="$cross"
export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_SYSROOT_DIR="$sysroot"
export PKG_CONFIG_LIBDIR="$sysroot/usr/lib/pkgconfig:$sysroot/usr/share/pkgconfig"

(cd "$app" && cargo build --release --locked --target "$triple")

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp -r "$app/package/." "$stage/"
entry=$(sed -n 's/^entry *= *"\(.*\)"/\1/p' "$stage/manifest.toml")
mkdir -p "$stage/$(dirname "$entry")"
install -m 0755 "$app/target/$triple/release/$name" "$stage/$entry"
kpkg build "$stage" -o "$out/$(sed -n 's/^id *= *"\(.*\)"/\1/p' "$stage/manifest.toml")-$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$stage/manifest.toml").kpkg"
