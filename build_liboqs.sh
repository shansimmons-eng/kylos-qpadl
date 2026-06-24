#!/bin/bash
# Build and install liboqs from source.
#
# Required for kylos-qpadl: liboqs must be compiled with MAYO support
# (-DOQS_ENABLE_SIG_MAYO=Yes).
#
# Prerequisites:
#   cmake, ninja-build (or make), curl, tar, gcc/clang 7.1+
#
# Environment variables:
#   LIBOQS_VERSION   liboqs version (default: 0.13.0)
#   INSTALL_DIR      installation prefix (default: /tmp/liboqs_install)
#   BUILD_DIR        build directory (default: /tmp/liboqs_build)
#   JOBS             parallel build jobs (default: nproc)

set -e

LIBOQS_VERSION="${LIBOQS_VERSION:-0.13.0}"
INSTALL_DIR="${INSTALL_DIR:-/tmp/liboqs_install}"
BUILD_DIR="${BUILD_DIR:-/tmp/liboqs_build}"
JOBS="${JOBS:-$(nproc)}"

SRC_DIR="/tmp/liboqs_src_${LIBOQS_VERSION}"

echo "=== Building liboqs ${LIBOQS_VERSION} ==="
echo "Install: ${INSTALL_DIR}"
echo "Build:   ${BUILD_DIR}"
echo "Jobs:    ${JOBS}"
echo ""

# Download if not already extracted
if [ ! -d "$SRC_DIR" ]; then
  mkdir -p "$SRC_DIR"
  echo "Downloading liboqs ${LIBOQS_VERSION}..."
  curl -fsSL "https://github.com/open-quantum-safe/liboqs/archive/refs/tags/${LIBOQS_VERSION}.tar.gz" \
    | tar xz -C "$SRC_DIR" --strip-components=1
fi

rm -rf "$BUILD_DIR" "$INSTALL_DIR"
mkdir -p "$BUILD_DIR" "$INSTALL_DIR"

cd "$BUILD_DIR"
cmake "$SRC_DIR" \
  -DOQS_BUILD_ONLY_LIB=Yes \
  -DOQS_DIST_BUILD=Yes \
  -DOQS_ENABLE_SIG_MAYO=Yes \
  -DOQS_USE_OPENSSL=No \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR"

cmake --build . --target install --config Release -j "$JOBS"

echo ""
echo "=== liboqs ${LIBOQS_VERSION} installed to ${INSTALL_DIR} ==="
ls -la "$INSTALL_DIR/lib/" "$INSTALL_DIR/include/oqs/"
