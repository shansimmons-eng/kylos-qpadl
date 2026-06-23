# QPADL — Quantum-Protected Algorithmic Defense Layer

## Architecture Rationale for Kylos Arc Sovereign Mirror

### Overview

QPADL is the post-quantum signature architecture for the Kylos Arc Sovereign
Mirror BFT mesh. It enforces **defense-in-depth through mathematical
diversity**: three cryptographically independent signature families, each
backed by a provably distinct hardness assumption, such that any single
mathematical breakthrough compromises at most one layer.

```
                 ┌─────────────────────────────────────┐
                 │         Sovereign Mirror BFT         │
                 │      (Agile-Unanimity Consensus)     │
                 └──────────────┬──────────────────────┘
                                │
                 ┌──────────────┴──────────────┐
                 │      QPADL Verification      │
                 │  (All three layers required) │
                 ├──────────────┬──────────────┤
                 │   Speed      │   Primary     │
                 │   MAYO-1/3/5 │   ML-DSA-65   │
                 │   (MQ)       │   (Lattice)   │
                 │   NP-Hard    │   Module-LWE  │
                 │   NIST On-   │   FIPS 204    │
                 │   Ramp       │               │
                 ├──────────────┴──────────────┤
                 │         Anchor Layer         │
                 │   SPHINCS+-SHA2-256f-simple  │
                 │   (Hash, SHA-256 only)       │
                 │   FIPS 205 / No structures   │
                 └─────────────────────────────┘
```

### Why Three Families?

The Bitcoin whitepaper era assumed elliptic-curve security was monolithic.
Post-quantum cryptography is less settled — each family carries unknown
future cryptanalytic risk. The three-family guarantee ensures:

| Break                          | Surviving Layers     | Mesh Status |
|--------------------------------|----------------------|-------------|
| Quantum MQ solver found        | Lattice + Hash       | Secure      |
| Lattice quantum algorithm      | MQ + Hash            | Secure      |
| SHA-256 preimage attack        | MQ + Lattice         | Secure      |
| Practical quantum computer     | All three (by design)| Secure      |
| Side-channel in one impl       | Other two unaffected | Secure      |

### Layer Definitions

#### Speed Layer: MAYO / UOV (Multivariate Quadratic)

**Role**: Primary consensus signing. Highest throughput, lowest latency.

**Algorithm**: MAYO-1 (NIST Level I), MAYO-3 (NIST Level III), MAYO-5 (NIST Level V).

**Rationale**:
- Verification is deterministic integer arithmetic — **no floating-point,
  no FFT, no rounding ambiguity** across heterogeneous nodes
- MQ problem is NP-Complete; no quantum algorithm gives more than
  quadratic speedup
- ~57 µs verification (MAYO-1) supports tens of thousands of
  verifications per second per core
- Key generation is 104× faster than Falcon-512 (44 µs vs 4.6 ms)

**NIST Status**: Round 2 On-Ramp candidate. MAYO replaces Falcon-512 when
standardized.

#### Primary Layer: ML-DSA-65 (Lattice, Module-LWE)

**Role**: FIPS-standardized fallback. Activated when MAYO cryptanalysis
or NIST non-approval compromises the Speed Layer.

**Algorithm**: ML-DSA-65 (Dilithium-3, FIPS 204, NIST Level III).

**Rationale**:
- Verification uses integer NTT — constant-time, deterministic, no
  floating-point
- Module-LWE is the most analyzed post-quantum hardness assumption;
  NIST-standardized August 2024
- 1:1 lattice security reduction to SVP; no superpolynomial quantum
  advantage known

#### Anchor Layer: SPHINCS+/SLH-DSA (Hash-Based)

**Role**: Ultimate failsafe. Used only when both MQ and Lattice are
broken or suspect. No structured mathematics.

**Algorithm**: SPHINCS+-SHA2-256f-simple (FIPS 205, SHA-256 only).

**Rationale**:
- Security rests on SHA-256 second-preimage resistance alone — no
  lattice, no MQ, no number theory
- Stateless design prevents state-reuse attacks
- "f" (fast) variant chosen for rapid verification in emergency
  fallback scenarios
- Migration path: SHA-256 → SHA-3 → future hash, independently of
  the other two layers

### Falcon-512 Temporary Patch

Deployed as a stopgap while MAYO matures through NIST On-Ramp. Used with
these known risks documented in the threat model:

- Floating-point FFT verification is NOT deterministic across
  x86_64/aarch64/riscv64
- Constant-time audit runs in CI: 1,000 iterations per algorithm
- Migration to MAYO scheduled upon NIST On-Ramp finalization

### P-Gate Physicalization Constraint

All verification MUST be deterministic integer arithmetic to produce
identical results across heterogeneous P-Gate nodes:

| Constraint            | Requirement                   |
|-----------------------|-------------------------------|
| No floating-point     | No FFT, no x87, no NEON FP   |
| No rounding variance  | IEEE 754 traps excluded       |
| Integer-only          | NTT, MQ eval, hash traversal |
| Deterministic verify  | Same sig → same pass/fail    |

This is NOT optional. A single verification divergence in a BFT consensus
mesh causes a partition, a fork, or a liveness failure.

### Cross-Architecture Strategy

| Target       | Verification Method       | CI Status       |
|--------------|---------------------------|-----------------|
| x86_64       | Native (AMD64 AVX2)       | Full test suite |
| aarch64      | Native (NEON)             | QEMU via cross  |
| riscv64      | Native (RV64GC)           | QEMU via cross  |

Test vectors are generated on x86_64 and verified on all three
architectures. Each release includes a constant-time audit report
from all targets.

### Cryptographic Agility Roadmap

| Timeline       | Speed Layer   | Primary Layer | Anchor Layer   |
|----------------|---------------|---------------|----------------|
| Current        | MAYO-1/3/5    | ML-DSA-65     | SPHINCS+-256f  |
| NIST On-Ramp   | MAYO (final)  | ML-DSA-65     | SPHINCS+-256f  |
| Next gen       | PROV / 3rd MQ | HAWK / Mitaka | SHA-3-based    |
| Breakthrough   | Replace MQ    | Replace Lattice | Replace hash |

Each layer can be replaced independently without touching the other two.

### References

- NIST FIPS 204: Module-Lattice-Based Digital Signature Standard (ML-DSA)
- NIST FIPS 205: Stateless Hash-Based Digital Signature Standard (SLH-DSA)
- NIST FIPS 206: FN-DSA (Falcon) — documented risk, rejected for Speed Layer
- NIST On-Ramp: MAYO and UOV submissions
- Frontiers in Blockchain (2026): Falcon FFT precision analysis
- Kylos Arc QPADL Threat Model: `QPADL_THREAT_MODEL.md`
- Benchmarks: `BENCHMARKS.md`
