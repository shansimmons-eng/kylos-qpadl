# Kylos Arc QPADL — MAYO Benchmarks

## Overview

Benchmarks for MAYO (Multivariate Quadratic) signatures in the Kylos Arc QPADL
Speed Layer. All verification uses deterministic integer arithmetic only — no
floating-point, no FFT.

## Methodology

- **Precision**: `std::time::Instant` with nanosecond resolution
- **Sample size**: 10,000 iterations per measurement
- **Warm-up**: 500 iterations before measurement
- **Compiler**: rustc 1.94+ with `--release` (LTO enabled)
- **Library**: liboqs 0.13.0 (raw FFI, no wrapper crate)
- **Host**: x86_64, AMD EPYC / Intel Xeon (CI), QEMU for aarch64/riscv64

## Test Vector Generation

Test vectors use fixed messages across all security levels:

| Label   | Size     | Content |
|---------|----------|---------|
| empty   | 0 B      | No bytes |
| short   | 51 B     | "Hello, Kylos Arc QPADL MAYO test vector" |
| medium  | 145 B    | Speed Layer description paragraph |
| large   | 1 MiB    | `0xAB` repeated |
| 1-byte  | 1 B      | `0x78` ('x') |

Keypairs are generated once (system RNG) and committed as test vectors so that
all architectures verify the same signatures.

## Edge Cases Covered

- Zero-length message sign + verify
- 1-byte message sign + verify
- 1 MiB message sign + verify
- Malformed signature (corrupted first byte) — MUST be rejected
- Deterministic verification: signing the same message twice produces a
  different signature (non-deterministic signing), but both verify identically

## Cross-Architecture Targets

| Target                       | Emulation        |
|------------------------------|------------------|
| `x86_64-unknown-linux-gnu`   | Native (CI)      |
| `aarch64-unknown-linux-gnu`  | QEMU via `cross` |
| `riscv64gc-unknown-linux-gnu`| QEMU via `cross` |

## Running Benchmarks

```bash
# Custom harness (nanosecond precision, 10k iterations, CSV output)
cargo run --release --bin kylos-bench

# Criterion benchmarks (statistical, for development)
cargo bench

# Test vectors
cargo run --release --bin kylos-test-vectors generate
cargo run --release --bin kylos-test-vectors verify

# Full test suite
cargo test --release
```

## Results (x86_64 Native)

Benchmarked on x86_64 (local workstation). All times in nanoseconds, 10,000 iterations per measurement.

### MAYO-1 (NIST Level I)

| Operation | Message    | Avg Time (ns) | Ops/sec   |
|-----------|-----------|---------------|-----------|
| keygen    | N/A       | 44,304        | 22,571    |
| sign      | empty     | 132,543       | 7,545     |
| verify    | empty     | 57,662        | 17,343    |
| sign      | 15 B      | 136,319       | 7,336     |
| verify    | 15 B      | 52,909        | 18,901    |
| sign      | 1,024 B   | 133,282       | 7,503     |
| verify    | 1,024 B   | 56,377        | 17,738    |
| sign      | 65,536 B  | 262,908       | 3,804     |
| verify    | 65,536 B  | 180,609       | 5,537     |

### MAYO-3 (NIST Level III)

| Operation | Message    | Avg Time (ns) | Ops/sec   |
|-----------|-----------|---------------|-----------|
| keygen    | N/A       | 103,066       | 9,703     |
| sign      | empty     | 306,533       | 3,262     |
| verify    | empty     | 124,291       | 8,046     |
| sign      | 15 B      | 308,126       | 3,245     |
| verify    | 15 B      | 124,149       | 8,055     |
| sign      | 1,024 B   | 310,454       | 3,221     |
| verify    | 1,024 B   | 127,951       | 7,816     |
| sign      | 65,536 B  | 442,240       | 2,261     |
| verify    | 65,536 B  | 251,164       | 3,982     |

### MAYO-5 (NIST Level V)

| Operation | Message    | Avg Time (ns) | Ops/sec   |
|-----------|-----------|---------------|-----------|
| keygen    | N/A       | 233,456       | 4,284     |
| sign      | empty     | 621,540       | 1,609     |
| verify    | empty     | 268,125       | 3,730     |
| sign      | 15 B      | 636,627       | 1,571     |
| verify    | 15 B      | 268,368       | 3,726     |
| sign      | 1,024 B   | 631,549       | 1,583     |
| verify    | 1,024 B   | 261,120       | 3,830     |
| sign      | 65,536 B  | 751,586       | 1,331     |
| verify    | 65,536 B  | 388,222       | 2,576     |

### Key Takeaways

- **MAYO-1 verify**: ~57 µs → **17,000+ verifications/sec** — well within Speed Layer target
- **MAYO-3 verify**: ~127 µs → **7,800+ verifications/sec**
- **MAYO-5 verify**: ~270 µs → **3,700+ verifications/sec**
- Signing is 2-3x slower than verification across all levels (expected for UOV-based schemes)
- Message size has minimal impact on sign/verify time except at large sizes (65 KB+ streaming overhead)
- Key generation is ~40% of sign time, suitable for ephemeral keys if needed

## ML-DSA-65 (Primary Layer, Lattice, FIPS 204)

### Performance (x86_64 — estimates based on liboqs upstream)

| Algorithm           | Operation | Avg Time  | Ops/sec   |
|---------------------|-----------|-----------|-----------|
| ML-DSA-65           | keygen    | ~150 µs   | ~6,600    |
| ML-DSA-65           | sign      | ~230 µs   | ~4,300    |
| ML-DSA-65           | verify    | ~80 µs    | ~12,500   |

*Exact numbers pending local benchmark run (`cargo run --release --bin kylos-bench ml-dsa`).*

### Key Sizes (FIPS 204)

| Parameter | Size     |
|-----------|----------|
| Public key | 1,952 B |
| Secret key | 4,000 B |
| Signature  | 3,309 B |

## SPHINCS+-SHA2-256f-simple (Anchor Layer, Hash, FIPS 205)

### Performance (x86_64 — estimates based on liboqs upstream)

| Algorithm                  | Operation | Avg Time   | Ops/sec  |
|----------------------------|-----------|------------|----------|
| SPHINCS+-SHA2-256f-simple  | keygen    | ~500 µs    | ~2,000   |
| SPHINCS+-SHA2-256f-simple  | sign      | ~5 ms      | ~200     |
| SPHINCS+-SHA2-256f-simple  | verify    | ~1.5 ms    | ~660     |

*Exact numbers pending local benchmark run (`cargo run --release --bin kylos-bench sphincs`).*

### Key Sizes (FIPS 205)

| Parameter | Size        |
|-----------|-------------|
| Public key | 64 B        |
| Secret key | 128 B       |
| Signature  | ~50 KB      |

## Falcon-512 Temporary Patch

### Performance (x86_64)

| Algorithm           | Operation | Avg Time  | Ops/sec   |
|---------------------|-----------|-----------|-----------|
| Falcon-512          | keygen    | 4,561 µs  | 219       |
| Falcon-512          | sign      | 150 µs    | 6,645     |
| Falcon-512          | verify    | **27 µs** | **36,204**|
| Falcon-padded-512   | keygen    | 4,575 µs  | 219       |
| Falcon-padded-512   | sign      | 154 µs    | 6,476     |
| Falcon-padded-512   | verify    | **26 µs** | **37,893**|

### Comparison: Falcon-512 vs MAYO-1

| Metric          | Falcon-512  | MAYO-1      | Winner    |
|-----------------|-------------|-------------|-----------|
| Verify latency  | 27 µs       | 57 µs       | Falcon 2.1x |
| Sign latency    | 150 µs      | 133 µs      | MAYO 1.1x |
| Keygen latency  | 4,561 µs    | 44 µs       | MAYO 104x |
| Signature size  | 752 B       | ~7.5 KB     | Falcon 10x |
| Public key size | 897 B       | 1,420 B     | Falcon 1.6x |

### ⚠️ RISK: Floating-Point FFT Verification

Falcon uses floating-point FFT (Fast Fourier Transform) for both signing and
verification. This creates a **determinism risk** across heterogeneous nodes:

- **x86_64**: Uses 80-bit extended-precision x87 or AVX-512 FP
- **aarch64**: Uses IEEE 754 double-precision NEON
- **riscv64**: Uses IEEE 754 double-precision (may lack FMA)

These different FP implementations can produce **different rounding results**
for the same signature, potentially causing verification to pass on one
architecture and fail on another — a **consensus fork** in a BFT mesh.

liboqs implements Falcon with software emulation of fixed-point arithmetic to
mitigate this, but the mitigation is not formally verified for cross-arch
determinism.

### Status: Temporary Patch

Falcon-512 is deployed as a **short-term patch** while MAYO matures through
the NIST On-Ramp process. MAYO's integer-only UOV verification eliminates the
floating-point determinism risk entirely.

**Migration path:**
1. ✅ MAYO test vectors + CI pipeline deployed
2. 🔄 Falcon-512 temporary patch (current)
3. ❌ Replace Falcon-512 with MAYO once NIST standardizes
4. ❌ Purge floating-point crypto from the Speed Layer

## Three-Layer Comparison

| Metric            | MAYO-1 (Speed) | ML-DSA-65 (Primary) | SPHINCS+-256f (Anchor) |
|-------------------|----------------|---------------------|------------------------|
| Hardness          | MQ (NP-Comp)   | Module-LWE          | SHA-256 preimage       |
| Verify latency    | ~57 µs         | ~80 µs              | ~1.5 ms                |
| Sign latency      | ~133 µs        | ~230 µs             | ~5 ms                  |
| Keygen            | ~44 µs         | ~150 µs             | ~500 µs                |
| Signature size    | ~7.5 KB        | 3,309 B             | ~50 KB                 |
| Public key size   | 1,420 B        | 1,952 B             | 64 B                   |
| Standardization   | NIST On-Ramp   | FIPS 204            | FIPS 205               |
| Arithmetic        | Integer only   | Integer NTT         | Hash chain             |
| Floating-point    | Never          | Never               | Never                  |

All three layers satisfy the P-Gate constraint: deterministic integer
verification with zero floating-point operations.

## Constant-Time Audit

A deterministic verification audit runs 1,000 consecutive verifications of the
same signature and measures:
- **Pass/fail consistency**: 100/100/100% across all algorithms tested
- **Timing variance**: reported as stddev (high variance on general-purpose OS
  is expected; true constant-time analysis requires locked-frequency hardware)

See `constant_time_audit.csv` for full results.

## Architecture Decisions

See `SPEC.md` in the project root for the full QPADL architecture rationale.

### Speed Layer: MAYO / UOV
- **Security**: Multivariate Quadratic (NP-Hard)
- **NIST Status**: Round 2 on-ramp candidate
- **Verification**: Deterministic integer arithmetic
- **Est. sig size**: ~7.5 KB (MAYO-1)

### Primary Layer: ML-DSA-65 (Dilithium-3)
- **Security**: Module-LWE (Lattice, FIPS 204)
- **Verification**: Integer NTT, constant-time
- **Standardized**: August 2024
- **Key sizes**: pk=1,952B, sk=4,000B, sig=3,309B

### Anchor Layer: SPHINCS+-SHA2-256f-simple
- **Security**: SHA-256 second-preimage (no structured math)
- **Verification**: Hash chain traversal, constant-time
- **Standardized**: FIPS 205 (SLH-DSA)
- **Key sizes**: pk=64B, sk=128B, sig=~50KB

### Why not Falcon / FN-DSA?
Falcon requires floating-point FFT verification with strict rounding, making it
incompatible with permissionless heterogeneous BFT consensus. MAYO's integer
verification eliminates this failure mode entirely.
