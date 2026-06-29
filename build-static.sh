#!/usr/bin/env bash
# Build a fully static (musl) browser-history binary that runs on any
# x86_64 Linux distro, independent of glibc / NixOS store paths.
#
# Output: target/x86_64-unknown-linux-musl/release/browser-history
#
# On NixOS this uses `nix-shell` to provide a rustup toolchain and a
# musl-targeting GCC. On other distros, ensure you have:
#   - rustup with the x86_64-unknown-linux-musl target installed
#   - a musl C compiler (e.g. musl-gcc / musl-tools)
# and run the `cargo build` line directly.
set -euo pipefail

TARGET=x86_64-unknown-linux-musl

build() {
  local MUSLCC
  MUSLCC=$(command -v x86_64-unknown-linux-musl-cc \
                    x86_64-unknown-linux-musl-gcc \
                    musl-gcc 2>/dev/null | head -1)
  if [ -z "${MUSLCC:-}" ]; then
    echo "error: no musl C compiler found on PATH" >&2
    exit 1
  fi
  echo "Using musl CC: $MUSLCC"

  rustup target add "$TARGET" >/dev/null 2>&1 || true

  CC_x86_64_unknown_linux_musl="$MUSLCC" \
  CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER="$MUSLCC" \
  RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target "$TARGET"

  local BIN="target/$TARGET/release/browser-history"
  strip "$BIN" 2>/dev/null || true
  echo
  echo "Built static binary: $BIN"
  file "$BIN"
}

if [ -e /etc/NIXOS ] && [ "${IN_BUILD_SHELL:-}" != "1" ]; then
  # Re-enter inside a nix-shell that provides the toolchain.
  export RUSTUP_HOME="$PWD/.rustup"
  export CARGO_HOME="$PWD/.cargo"
  export PATH="$CARGO_HOME/bin:$PATH"
  exec nix-shell -p rustup pkgsCross.musl64.buildPackages.gcc --run "
    set -e
    export RUSTUP_HOME='$RUSTUP_HOME'
    export CARGO_HOME='$CARGO_HOME'
    export PATH='$CARGO_HOME/bin':\$PATH
    rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
    IN_BUILD_SHELL=1 bash '$0'
  "
else
  build
fi
