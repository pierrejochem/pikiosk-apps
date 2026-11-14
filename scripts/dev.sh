#!/bin/sh
# Runs an app in a desktop window (no DRM/KMS, no pikiosk device needed).
#
#   scripts/dev.sh <app> [cargo run args...]
set -eu
app=${1:?usage: dev.sh <app> [cargo run args...]}
shift
cd "$(dirname "$0")/../apps/$app"
exec cargo run --features desktop "$@"
