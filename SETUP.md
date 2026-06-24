# QPADL Setup

## Prerequisites

| Requirement | Minimum Version | Notes |
|---|---|---|
| Rust | 1.85+ | `edition = "2024"` in Cargo.toml |
| liboqs | 0.13.0 | Must be compiled with `-DOQS_ENABLE_SIG_MAYO=Yes` |
| CMake | 3.16+ | For building liboqs |
| Ninja | 1.10+ | Optional but recommended for liboqs build |
| GCC / Clang | 7.1+ | liboqs requires C11 atomics (GCC 7.1+) |

## Quick Start

### 1. Build liboqs

```bash
bash build_liboqs.sh
```

This installs liboqs to `/tmp/liboqs_install/` by default. Override with:

```bash
export LIBOQS_INSTALL_DIR=/path/to/custom/liboqs
bash build_liboqs.sh  # note: edit build_liboqs.sh INSTALL_DIR to match
```

### 2. Build kylos-qpadl

```bash
export LIBOQS_INSTALL_DIR=/tmp/liboqs_install
bash build.sh
```

Or with custom LLVM for bindgen:

```bash
export LIBCLANG_PATH=/path/to/llvm/lib
export BINDGEN_EXTRA_CLANG_ARGS="-I/path/to/gcc/include -I/usr/include"
bash build.sh
```

### 3. Run tests

```bash
cargo test --release
```

### 4. Start the JSON-RPC crypto server

```bash
cargo run --release --bin kylos-crypto-server
```

The server reads JSON-RPC requests from stdin and writes responses to stdout. Example:

```bash
echo '{"id":1,"method":"status","params":{}}' | cargo run --release --bin kylos-crypto-server
```

## Sovereign Mirror Integration

The `server/` directory (in the Sovereign Mirror project, not this repo) spawns `kylos-crypto-server` as a subprocess.

### Required Environment Variables

Set these before starting the Express server:

| Variable | Required | Default | Description |
|---|---|---|---|
| `CRYPTO_SERVER_BIN` | No | `wsl.exe` | Binary to spawn (set to `./kylos-crypto-server` on Linux) |
| `CRYPTO_SERVER_ARGS` | No | _(see below)_ | JSON array of arguments, e.g. `'["--flag"]` |

**Default args** (if `CRYPTO_SERVER_ARGS` unset):
- WSL dev: `['/home/PROJECT_DIR/target/release/kylos-crypto-server']`

**Example Linux setup:**

```bash
export CRYPTO_SERVER_BIN=./target/release/kylos-crypto-server
export CRYPTO_SERVER_ARGS='[]'
```

### Crypto API Endpoints

| Method | Path | Purpose |
|---|---|---|
| `GET` | `/api/crypto/status` | List enabled algorithms |
| `POST` | `/api/crypto/keypair` | Generate keypair for algorithm |
| `POST` | `/api/crypto/sign` | Sign a message |
| `POST` | `/api/crypto/verify` | Verify a signature |

All I/O uses base64 encoding. See `src/services/apiService.ts` for the client.

## Cross-Compilation

```bash
# Requires cross toolchains:
#   sudo apt install gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu
bash scripts/cross_compile.sh native
```

CI uses native cross-compilation on Ubuntu 24.04 runners (see `.github/workflows/ci.yml`).

## Security Notes

- Test vector key files under `test_vectors/` are throwaway CI fixtures, not production credentials
- The JSON-RPC crypto server enforces input size limits (2 MB messages, 128 KB keys/signatures)
- All crypto operations use liboqs constant-time implementations where available
- The `cryptoPending` request map is capped at 256 entries with 30-second timeout
