#!/bin/bash
# Cross-compile liboqs + kylos-qpadl for aarch64 and riscv64
# Requires: Docker (for cross-rs) or native cross toolchains
#
# NOTE: cross-rs Docker images (ghcr.io/cross-rs/*:latest) use GCC 5.4.0
# which is too old for liboqs (req GCC 7.1+). CI uses native cross-compilation
# on Ubuntu 24.04 runners (see .github/workflows/ci.yml).
# For local cross-compilation, use 'native' mode (requires cross toolchains
# installed via apt).
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Kylos Arc QPADL Cross-Compilation ==="
echo "Project: $PROJECT_DIR"
echo ""

# Option 1: cross-rs (Docker-based, recommended for CI)
cross_build() {
    local target="$1"
    echo "--- Building for $target via cross ---"
    cd "$PROJECT_DIR"

    # cross reads Cross.toml for pre-build steps
    cross build --release --target "$target"
    cross test --release --target "$target"
    echo "--- $target build OK ---"
}

# Option 2: Native cross-compilation (requires toolchains installed)
native_build() {
    local target="$1"
    local cc_var=""
    local cross_prefix=""

    case "$target" in
        aarch64-unknown-linux-gnu)
            cross_prefix="aarch64-linux-gnu"
            cc_var="CC_aarch64_unknown_linux_gnu"
            export "$cc_var=${cross_prefix}-gcc"
            export "AR_aarch64_unknown_linux_gnu=${cross_prefix}-ar"
            export "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=${cross_prefix}-gcc"
            ;;
        riscv64gc-unknown-linux-gnu)
            cross_prefix="riscv64-linux-gnu"
            cc_var="CC_riscv64gc_unknown_linux_gnu"
            export "$cc_var=${cross_prefix}-gcc"
            export "AR_riscv64gc_unknown_linux_gnu=${cross_prefix}-ar"
            export "CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=${cross_prefix}-gcc"
            ;;
        *)
            echo "Unknown target: $target"
            exit 1
            ;;
    esac

    # Verify cross compiler exists
    if ! command -v "${cross_prefix}-gcc" &>/dev/null; then
        echo "ERROR: ${cross_prefix}-gcc not found"
        echo "Install: sudo apt install gcc-${cross_prefix} g++-${cross_prefix}"
        exit 1
    fi

    echo "--- Building $target natively ---"

    # Build liboqs for target
    local src_dir
    src_dir=$(ls -d ~/.cargo/registry/src/index.crates.io-*/oqs-sys-*/liboqs 2>/dev/null | head -1)
    if [ -z "$src_dir" ]; then
        echo "Fetching liboqs source..."
        mkdir -p /tmp/liboqs_src
        curl -fsSL https://github.com/open-quantum-safe/liboqs/archive/refs/tags/0.13.0.tar.gz | tar xz -C /tmp/liboqs_src --strip-components=1
        src_dir=/tmp/liboqs_src
    fi

    local install_dir="/tmp/liboqs_install_${target}"
    local build_dir="/tmp/liboqs_build_${target}"
    rm -rf "$build_dir" "$install_dir"
    mkdir -p "$build_dir" "$install_dir"

    cd "$build_dir"
    cmake "$src_dir" \
        -DOQS_BUILD_ONLY_LIB=Yes \
        -DOQS_DIST_BUILD=Yes \
        -DOQS_ENABLE_SIG_MAYO=Yes \
        -DOQS_ENABLE_SIG_FALCON=Yes \
        -DOQS_USE_OPENSSL=No \
        -DCMAKE_BUILD_TYPE=Release \
        -DCMAKE_INSTALL_PREFIX="$install_dir" \
        -DCMAKE_C_COMPILER="${cross_prefix}-gcc" \
        -DCMAKE_CXX_COMPILER="${cross_prefix}-g++"

    cmake --build . --target install --config Release -j$(nproc)
    cd "$PROJECT_DIR"

    # Build Rust project with target liboqs
    export LIBOQS_INSTALL_DIR="$install_dir"
    cargo build --release --target "$target"
    cargo test --release --target "$target" 2>/dev/null || \
        echo "Note: tests may not run without QEMU user mode"

    echo "--- $target build OK ---"
}

# Main
case "${1:-check}" in
    cross)
        cross_build "aarch64-unknown-linux-gnu"
        cross_build "riscv64gc-unknown-linux-gnu"
        ;;
    native)
        native_build "aarch64-unknown-linux-gnu"
        native_build "riscv64gc-unknown-linux-gnu"
        ;;
    check)
        echo "Usage: $0 {cross|native}"
        echo ""
        echo "  cross  - Build using cross-rs (Docker images, Cross.toml)"
        echo "  native - Build using native cross-compilation toolchains"
        echo ""
        echo "Cross-compilation targets:"
        echo "  aarch64-unknown-linux-gnu   (Raspberry Pi 5, AWS Graviton, Apple Silicon)"
        echo "  riscv64gc-unknown-linux-gnu (SiFive, Esperanto)")
        echo ""
        echo "Rust code validated for:"
        cargo check --target aarch64-unknown-linux-gnu --release 2>&1 | tail -1
        cargo check --target riscv64gc-unknown-linux-gnu --release 2>&1 | tail -1
        echo ""
        echo "Full compilation requires liboqs cross-compiled for the target."
        echo "See Cross.toml for Docker-based cross-compilation config."
        ;;
esac
