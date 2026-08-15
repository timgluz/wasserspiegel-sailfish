#!/usr/bin/env bash
# Cross-compile wasserspiegel-core for aarch64 inside the Sailfish SDK
# build engine, then copy the staticlib + cxx bridge headers back to the
# host so the qmake (.pro) step can link them.
#
# Requires the engine to be set up first (see engine-setup.sh / `task engine:setup`).
set -euo pipefail

SFDK="${SFDK:-sfdk}"
RUST_DIR="$(cd "$(dirname "$0")" && pwd)"

command -v "$SFDK" >/dev/null || { echo "sfdk not found (set SFDK=... or export PATH)" >&2; exit 1; }

CONTAINER="${CONTAINER:-$(docker ps --format '{{.Names}}' 2>/dev/null | grep '^sailfish-sdk-build-engine' | head -1)}"
: "${CONTAINER:?could not find the SDK build engine container - is Docker running and the SDK built once?}"

sfdk_exec() { "$SFDK" engine exec sh -c "$1"; }

TARGET="$(sfdk_exec 'ls /srv/mer/targets 2>/dev/null | grep -- "-aarch64$" | sort -V | tail -1')"
: "${TARGET:?no aarch64 SDK target found in engine}"

# toolings dir drops the trailing "-aarch64" (e.g. SailfishOS-5.1.0.11)
TOOLING="${TARGET%-aarch64}"
CROSS="/srv/mer/toolings/$TOOLING/opt/cross/bin"
TRIPLE="aarch64-unknown-linux-gnu"
WS="/home/mersdk/ws-rust"

echo "== staging sources =="
docker exec "$CONTAINER" sh -c "rm -rf $WS && mkdir -p $WS"
tar -C "$RUST_DIR" --exclude=target --exclude=include -c . | docker cp - "$CONTAINER:$WS"
docker exec "$CONTAINER" sh -c "chown -R mersdk:mersdk $WS"

echo "== building (target $TARGET) =="
sfdk_exec "
  export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/home/mersdk/cross-tools/bin/aarch64-meego-linux-gnu-g++
  export CC_aarch64_unknown_linux_gnu=/home/mersdk/cross-tools/bin/aarch64-meego-linux-gnu-gcc
  export CXX_aarch64_unknown_linux_gnu=/home/mersdk/cross-tools/bin/aarch64-meego-linux-gnu-g++
  export AR_aarch64_unknown_linux_gnu=$CROSS/aarch64-meego-linux-gnu-ar
  cd $WS && \$HOME/.cargo/bin/cargo build --release --target $TRIPLE
"

echo "== copying artifacts back =="
mkdir -p "$RUST_DIR/target/$TRIPLE/release"
docker cp "$CONTAINER:$WS/target/$TRIPLE/release/libwasserspiegel_core.a" "$RUST_DIR/target/$TRIPLE/release/"
rm -rf "$RUST_DIR/include"
docker cp "$CONTAINER:$WS/include" "$RUST_DIR/include"

echo "== done =="
ls -la "$RUST_DIR/target/$TRIPLE/release/libwasserspiegel_core.a"
ls "$RUST_DIR/include"
