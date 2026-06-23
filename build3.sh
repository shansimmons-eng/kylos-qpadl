#!/bin/bash
set -e
export LIBCLANG_PATH=/tmp/LLVM-22.1.8-Linux-X64/lib
export BINDGEN_EXTRA_CLANG_ARGS="-I/tmp/liboqs_install/include -I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/include"
export PKG_CONFIG_PATH=/tmp/liboqs_install/lib/pkgconfig
export LIBOQS_NO_VENDOR=1
export LD_LIBRARY_PATH=/tmp/liboqs_install/lib:$LD_LIBRARY_PATH

source "$HOME/.cargo/env"
cd /home/retroporter/cup/kylos-qpadl
echo "=== Checking pkg-config ==="
pkg-config --libs --cflags liboqs 2>&1 || echo "pkg-config failed"
echo "=== Checking liboqs.a ==="
ls -la /tmp/liboqs_install/lib/liboqs.a
echo "=== Starting build ==="
cargo build --release -j4 2>&1
