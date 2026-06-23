#!/bin/bash
set -e
export LIBCLANG_PATH=/tmp/LLVM-22.1.8-Linux-X64/lib
export BINDGEN_EXTRA_CLANG_ARGS="-I/usr/lib/gcc/x86_64-linux-gnu/13/include -I/usr/include -x c"
cd /home/retroporter/cup/kylos-qpadl
cargo build --release -j4 2>&1
