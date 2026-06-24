#!/bin/bash
# Build kylos-qpadl with liboqs.
#
# Prerequisites:
#   - cmake, ninja-build
#   - libclang-dev (for bindgen)
#   - LLVM dev libraries, or set LIBCLANG_PATH
#
# Required environment variables:
#   LIBOQS_INSTALL_DIR   Path to liboqs installation (default: /tmp/liboqs_install)
#                        Must contain lib/liboqs.a and include/oqs/.
#
# Optional environment variables:
#   LIBCLANG_PATH        Path to LLVM lib directory for bindgen
#   BINDGEN_EXTRA_CLANG_ARGS  Extra args for clang used by bindgen
#
# To build liboqs first, run: bash build_liboqs.sh

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIBOQS_INSTALL_DIR="${LIBOQS_INSTALL_DIR:-/tmp/liboqs_install}"
JOBS="${JOBS:-$(nproc)}"

echo "=== Kylos Arc QPADL Build ==="
echo "Project: $PROJECT_DIR"
echo "liboqs: $LIBOQS_INSTALL_DIR"
echo "Jobs: $JOBS"
echo ""

export LIBOQS_INSTALL_DIR

cd "$PROJECT_DIR"
cargo build --release -j "$JOBS" 2>&1

echo "=== Build complete ==="
echo "Binary: $PROJECT_DIR/target/release/kylos-crypto-server"
echo "Test:   cargo test --release"
