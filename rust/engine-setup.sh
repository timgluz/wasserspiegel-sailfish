#!/usr/bin/env bash
# One-time (idempotent) setup of the Sailfish SDK build engine so it can
# cross-compile the wasserspiegel-core Rust staticlib for aarch64.
#
# What it installs/creates inside the engine:
#   1. rustup as the mersdk user (i686 host toolchain - the engine is 32-bit)
#   2. the aarch64-unknown-linux-gnu rust target
#   3. host gcc-c++ (needed by the `link-cplusplus` build script)
#   4. cross-toolchain wrappers that add the correct sysroot + `as`/`ld`
#      (the SDK's cross gcc does not carry a usable default sysroot and
#       resolves `as`/`ld` to the host 32-bit ones)
set -euo pipefail

SFDK="${SFDK:-sfdk}"

command -v "$SFDK" >/dev/null || { echo "sfdk not found (set SFDK=... or export PATH)" >&2; exit 1; }

CONTAINER="${CONTAINER:-$(docker ps --format '{{.Names}}' 2>/dev/null | grep '^sailfish-sdk-build-engine' | head -1)}"
: "${CONTAINER:?could not find the SDK build engine container - is Docker running and the SDK built once?}"

sfdk_exec() { "$SFDK" engine exec sh -c "$1"; }

echo "== detecting engine target =="
TARGET="$(sfdk_exec 'ls /srv/mer/targets 2>/dev/null | grep -- "-aarch64$" | sort -V | tail -1')"
if [ -z "$TARGET" ]; then
    echo "no aarch64 SDK target found in engine; run 'sfdk build' once first" >&2
    exit 1
fi
echo "target: $TARGET"

# toolings dir drops the trailing "-aarch64" (e.g. SailfishOS-5.1.0.11)
TOOLING="${TARGET%-aarch64}"
CROSS="/srv/mer/toolings/$TOOLING/opt/cross/bin"
SYSROOT="/srv/mer/targets/$TARGET"

echo "== 1/4 installing rustup (mersdk, i686 host) =="
if ! sfdk_exec 'test -x "$HOME/.cargo/bin/rustup"'; then
    sfdk_exec 'curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --no-modify-path'
fi
sfdk_exec '$HOME/.cargo/bin/rustup toolchain install stable-i686-unknown-linux-gnu --profile minimal 2>/dev/null || true'
sfdk_exec '$HOME/.cargo/bin/rustup default stable-i686-unknown-linux-gnu'

echo "== 2/4 adding aarch64 target =="
sfdk_exec '$HOME/.cargo/bin/rustup target add aarch64-unknown-linux-gnu'

echo "== 3/4 ensuring host g++ (link-cplusplus) =="
if ! sfdk_exec 'command -v g++ >/dev/null'; then
    "$SFDK" engine exec sudo -n zypper --non-interactive install gcc-c++
fi

echo "== 4/4 creating cross-toolchain wrappers =="
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/bin"
cat > "$TMP/bin/aarch64-meego-linux-gnu-gcc" <<EOF
#!/bin/sh
exec $CROSS/aarch64-meego-linux-gnu-gcc --sysroot=$SYSROOT -B/home/mersdk/cross-tools "\$@"
EOF
cat > "$TMP/bin/aarch64-meego-linux-gnu-g++" <<EOF
#!/bin/sh
exec $CROSS/aarch64-meego-linux-gnu-g++ --sysroot=$SYSROOT -B/home/mersdk/cross-tools "\$@"
EOF
chmod +x "$TMP/bin/"*

docker exec "$CONTAINER" sh -c '
  mkdir -p /home/mersdk/cross-tools
  ln -sf '"$CROSS"'/aarch64-meego-linux-gnu-as /home/mersdk/cross-tools/as
  ln -sf '"$CROSS"'/aarch64-meego-linux-gnu-ld /home/mersdk/cross-tools/ld
  chown -R mersdk:mersdk /home/mersdk/cross-tools
'
docker cp "$TMP/bin/." "$CONTAINER:/home/mersdk/cross-tools/bin/"
docker exec "$CONTAINER" sh -c 'chmod +x /home/mersdk/cross-tools/bin/*; chown mersdk:mersdk /home/mersdk/cross-tools/bin/*'

echo "== done: engine ready =="
