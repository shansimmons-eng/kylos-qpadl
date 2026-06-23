#!/bin/bash
set -e
LIBOQS_SRC=/home/retroporter/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/oqs-sys-0.11.0+liboqs-0.13.0/liboqs
BUILD_DIR=/tmp/liboqs_build
INSTALL_DIR=/tmp/liboqs_install

rm -rf "$BUILD_DIR" "$INSTALL_DIR"
mkdir -p "$BUILD_DIR" "$INSTALL_DIR"

cd "$BUILD_DIR"
cmake "$LIBOQS_SRC" \
  -DOQS_BUILD_ONLY_LIB=Yes \
  -DOQS_DIST_BUILD=Yes \
  -DOQS_ENABLE_SIG_MAYO=Yes \
  -DOQS_USE_OPENSSL=No \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX="$INSTALL_DIR"

cmake --build . --target install --config Release -j4

echo "=== liboqs built and installed to $INSTALL_DIR ==="
ls -la "$INSTALL_DIR/lib/" "$INSTALL_DIR/include/oqs/"
