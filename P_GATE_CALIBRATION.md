# P-Gate Physicalization Calibration

## Overview

P-Gate is a hardware acceleration unit for the Kylos Arc Sovereign Mirror BFT
mesh. It must deterministically verify post-quantum signatures within strict
timing windows to satisfy the Agile-Unanimity quorum requirements.

This document calibrates P-Gate timing using measured and estimated benchmark
data from the QPADL signature stack.

## Constraint

All verification is deterministic integer arithmetic — no floating-point, no
FFT, no rounding-dependent divergence across x86_64/aarch64/riscv64.

## Latency Budget per Operation

| Layer     | Algorithm              | Verify (avg) | Sign (avg) | Keygen (avg) |
|-----------|------------------------|--------------|------------|--------------|
| Speed     | MAYO-1                 | **57 µs**   | 133 µs     | 44 µs        |
| Speed     | MAYO-3                 | 127 µs      | 308 µs     | 103 µs       |
| Speed     | MAYO-5                 | 270 µs      | 636 µs     | 233 µs       |
| Primary   | ML-DSA-65              | ~80 µs*     | ~230 µs*   | ~150 µs*     |
| Anchor    | SPHINCS+-SHA2-256f     | ~1.5 ms*    | ~5 ms*     | ~500 µs*     |
| Patch     | Falcon-512             | 27 µs       | 150 µs     | 4,561 µs     |
| Patch     | Falcon-padded-512      | 26 µs       | 154 µs     | 4,575 µs     |

*Values marked with * are liboqs upstream estimates; run `cargo run --release
--bin kylos-bench <algo>` for measured local numbers.*

## BFT Consensus Round Budget

For a committee of N nodes with quorum Q = min(N, ⌈√N⌉ + 2):

| N  | Q  | Max verify (MAYO-1) | Max verify (Falcon-512) |
|----|----|---------------------|-------------------------|
| 10 |  5 | 285 µs              | 135 µs                  |
| 25 |  7 | 399 µs              | 189 µs                  |
| 50 |  9 | 513 µs              | 243 µs                  |
| 100| 12 | 684 µs              | 324 µs                  |
| 1000| 34| 1,938 µs            | 918 µs                  |

**Worst-case total verify latency** = Q × max(verify latency).

For N=1000: Q=34 → MAYO-1 verify takes ~1.9 ms total. Entire consensus round
(including network, ordering, application logic) should budget ≥10× this:
~20 ms minimum round timeout for the Speed Layer.

## Three-Layer Overhead

In normal operation only MAYO (Speed Layer) is verified. ML-DSA-65 and
SPHINCS+ are verified only when:

1. **Speed Layer compromised**: ML-DSA-65 verify (~80 µs) → total verify
   ~2.7 ms for N=1000
2. **Speed + Primary compromised**: SPHINCS+ verify (~1.5 ms) → total verify
   ~51 ms for N=1000

## P-Gate Pipeline Sizing

Assuming a single P-Gate unit processes one verify at a time:

| Scenario          | Verify latency | Sustained verifies/sec (single P-Gate) |
|-------------------|---------------|----------------------------------------|
| MAYO-1 (normal)   | 57 µs         | 17,543                                 |
| ML-DSA-65 (fallback) | 80 µs     | 12,500                                 |
| SPHINCS+ (emergency) | 1,500 µs  | 666                                    |

For a mesh processing 10,000 tx/sec with Q=34 verifications per tx:

| Scenario          | Required verifies/sec | Single P-Gate | P-Gate units needed |
|-------------------|----------------------|----------------|---------------------|
| MAYO-1 (normal)   | 340,000              | 17,543         | 20                  |
| ML-DSA-65 (fallback) | 340,000          | 12,500         | 28                  |
| SPHINCS+ (emergency) | 340,000          | 666            | 511                 |

## Determinism Guarantee

Every P-Gate unit — regardless of x86_64, aarch64, or riscv64 host — produces
**identical verify/fail results** for the same (signature, message, public key)
tuple. This is guaranteed because:

1. MAYO verification is integer MQ evaluation only
2. ML-DSA-65 verification is integer NTT only
3. SPHINCS+ verification is SHA-256 hash chain only
4. No floating-point, no FFT, no hardware-dependent rounding

## Recommended P-Gate Configuration

| Parameter              | Value               | Rationale                     |
|------------------------|---------------------|--------------------------------|
| Clock                  | ≥ 100 MHz           | 10 ns per cycle; 57 µs → 5,700 cycles for MAYO-1 verify |
| Data path              | 256-bit             | SHA-256 word size              |
| Memory                 | 64 KB SRAM          | Largest sig ~50 KB (SPHINCS+)  |
| Pipeline depth         | 5-7 stages          | MAYO MQ eval pipeline          |
| Multiplicity           | 20+ units           | 10K tx/sec × Q=34 at MAYO-1   |
| Deterministic          | Hard-wired integer  | No microcode, no FPU           |

See `SPEC.md` for architecture rationale and `QPADL_THREAT_MODEL.md` for
attack analysis.
