# QPADL Threat Model: Post-Quantum Cryptographic Diversity for Kylos Arc Sovereign Mirror

## Overview

QPADL (Quantum-Protected Algorithmic Defense Layer) is the post-quantum
signature architecture for the Kylos Arc Sovereign Mirror BFT mesh. It
implements **defense-in-depth through mathematical diversity**: three
cryptographic families with provably distinct hardness assumptions, such
that any single mathematical breakthrough compromises at most one layer.

## The Three-Family Guarantee

```
  Speed Layer        Primary Layer        Anchor Layer
  ┌─────────────┐   ┌─────────────┐   ┌──────────────────┐
  │ MAYO / UOV  │   │ ML-DSA-65   │   │ SPHINCS+-SHA2-   │
  │ (MQ)        │   │ (Lattice)   │   │ 256f-simple      │
  │ NP-Hard     │   │ Module-LWE  │   │ (Hash, SHA-256)  │
  │ NIST On-Ramp│   │ FIPS 204    │   │ FIPS 205         │
  └─────────────┘   └─────────────┘   └──────────────────┘
        │                  │                  │
        └──────────────────┴──────────────────┘
                   All three required
              for a complete compromise
```

**Core guarantee**: An adversary must break all three distinct mathematical
hardness assumptions simultaneously to forge signatures in the Sovereign
Mirror. A breakthrough in any single family (e.g., a quantum algorithm
solving LWE) leaves the other two intact.

## Layer-by-Layer Analysis

### Layer 1: Speed Layer — MAYO / UOV (Multivariate Quadratic)

| Property          | Detail                                    |
|-------------------|-------------------------------------------|
| Algorithm         | MAYO (Multivariate Oil and Vinegar)       |
| Standardization   | NIST On-Ramp Round 2 candidate            |
| Hardness          | MQ problem (NP-Complete)                  |
| Signature size    | ~7.5 KB (MAYO-1)                          |
| Verification      | Deterministic integer arithmetic          |
| Verification time | ~57 µs (x86_64)                           |

**Hardness assumption**: Solving a system of multivariate quadratic equations
over a finite field is NP-Complete. The UOV trapdoor exploits the structure
of Oil and Vinegar variables — an attacker without the trapdoor must solve
the full MQ system.

**Known attacks**:
- **Direct algebraic attack (XL, Gröbner bases)**: Exponential in the number
  of variables; MAYO parameters chosen to exceed security margin
- **MinRank attack**: Exploits the oil-vinegar structure; MAYO uses
  parameterization that resists known MinRank techniques
- **Quantum**: Grover's algorithm provides at most quadratic speedup for
  MQ; no known quantum algorithm significantly reduces MQ complexity
- **Fault injection**: UOV signing operations can leak oil-vinegar
  separation; mitigated by MAYO's "whipping" technique

**Compromise impact**: If MQ is broken or MAYO is cryptanalyzed, the Speed
Layer falls back to ML-DSA-65 for all operations. Consensus throughput is
reduced (ML-DSA-65 verify is ~1.20 ms vs ~57 µs) but the mesh continues
to operate securely via the Primary Layer.

### Layer 2: Primary Layer — ML-DSA-65 (Dilithium-3, Lattice)

| Property          | Detail                                    |
|-------------------|-------------------------------------------|
| Algorithm         | ML-DSA-65 (Module-LWE, Dilithium-3)       |
| Standardization   | FIPS 204 (NIST, August 2024)              |
| Hardness          | Module-LWE (Average-case Lattice)         |
| Signature size    | 4.5 KB                                    |
| Verification      | Integer NTT, constant-time                |
| Verification time | ~1.20 ms (estimated, not yet benchmarked) |

**Hardness assumption**: Module Learning With Errors (MLWE). The security
reduction connects to the Shortest Vector Problem (SVP) over lattices,
which has no known quantum advantage beyond polynomial speedups.

**Known attacks**:
- **Lattice reduction (BKZ, sieving)**: Subexponential in lattice dimension;
  ML-DSA-65 parameters chosen for 128-bit post-quantum security
- **Quantum**: No superpolynomial quantum advantage for lattice problems;
  Grover-accelerated sieving provides limited speedup
- **Side-channel**: NTT implementation must be constant-time; masked
  implementations exist
- **Fault attacks**: Signing with biased noise can leak secret; mitigated
  by deterministic signing mode

**Compromise impact**: If Module-LWE is broken (e.g., a quantum algorithm
solves lattice problems efficiently), the Speed Layer (MQ) and Anchor Layer
(Hash) remain secure. The mesh continues to operate with reduced signature
throughput but full security.

### Layer 3: Anchor Layer — SPHINCS+/SLH-DSA (Hash-Based)

| Property          | Detail                                    |
|-------------------|-------------------------------------------|
| Algorithm         | SPHINCS+-SHA2-256f-simple (SLH-DSA)       |
| Standardization   | FIPS 205 (NIST, August 2024)              |
| Hardness          | SHA-256 second-preimage resistance        |
| Signature size    | ~50 KB                                    |
| Verification      | Stateless hash chain traversal            |
| Verification time | ~1.5 ms (estimated)                       |

**Hardness assumption**: Second-preimage resistance of the underlying hash
function (SHA-256 or SHAKE-256). No number-theoretic or algebraic structure
is involved — only the concrete security of the hash primitive.

**Known attacks**:
- **Hash function cryptanalysis**: SHA-256 has no known attack better than
  brute force; SHA-3 (SHAKE) has extensive security margins
- **Quantum**: Grover's algorithm provides quadratic speedup for preimage
  search; parameters doubled accordingly (SPHINCS+ 128s uses 256-bit
  hashes effectively)
- **State collision attacks**: Stateless design prevents state reuse
  attacks that plague stateful hash-based schemes (XMSS, LMS)
- **No structured math**: Hash-based signatures contain no algebraic
  structure for quantum algorithms to exploit (no SVP, no MQ, no
  elliptic curve, no RSA)

**Compromise impact**: If SHA-256 is broken (e.g., a preimage attack), the
Anchor Layer fails. However, such a break would also compromise TLS,
blockchain, and virtually all internet security. The Speed and Primary
Layers continue operating. A hash function migration (e.g., SHA-256 →
SHA-3 in a SHAKE-based SPHINCS+ variant) could restore Anchor Layer
security independently without changing the Speed or Primary Layers.

## Threat Scenarios

### Scenario A: Quantum Cryptanalytic Breakthrough (Single Family)

| Event                          | Speed | Primary | Anchor | Result                         |
|--------------------------------|-------|---------|--------|--------------------------------|
| Quantum MQ solver discovered   | BROKEN| SAFE    | SAFE   | Degraded throughput, secure   |
| Lattice quantum algorithm found| SAFE  | BROKEN  | SAFE   | Secure, MAYO absorbs load     |
| Hash preimage attack found     | SAFE  | SAFE    | BROKEN | Secure, rehash with SHA-3     |
| All three broken simultaneously| BROKEN| BROKEN  | BROKEN | Requires fundamentally new math|

### Scenario B: Implementation Vulnerability

| Event                          | Speed | Primary | Anchor | Result                         |
|--------------------------------|-------|---------|--------|--------------------------------|
| Constant-time bug in MAYO      | BROKEN| SAFE    | SAFE   | Fall back to ML-DSA-65        |
| NTT side-channel in ML-DSA-65  | SAFE  | BROKEN  | SAFE   | Speed Layer absorbs            |
| Hash DoS in SLH-DSA            | SAFE  | SAFE    | BROKEN| Two layers remain               |

### Scenario C: NIST Standardization Failure

| Event                          | Speed | Primary | Anchor | Result                         |
|--------------------------------|-------|---------|--------|--------------------------------|
| MAYO rejected by NIST          | BROKEN| SAFE    | SAFE   | Replace with alternative MQ    |
| ML-DSA-65 superseded           | SAFE  | BROKEN  | SAFE   | Replace with HAWK or similar   |
| SHA-256 weakened               | SAFE  | SAFE    | BROKEN | Migrate to SHA-3-based SLH-DSA |

## Falcon-512 Temporary Patch Risk

### Status

Falcon-512 is deployed as a **temporary hotfix** while MAYO matures through
the NIST On-Ramp. It is **not** part of the three-family guarantee.

### Identified Risks

1. **Floating-point FFT determinism** (CRITICAL)
   - Falcon verification uses FFT-based polynomial arithmetic with
     floating-point operations
   - Different architectures (x86_64 x87 80-bit vs aarch64 IEEE 754
     vs riscv64 soft-float) produce different rounding
   - Mitigation: liboqs uses software fixed-point emulation, but this
     is not formally verified for cross-arch determinism
   - Consensus fork potential: signature verifies on one node, fails on
     another in a heterogeneous BFT mesh

2. **Key generation performance** (OPERATIONAL)
   - Falcon-512 keygen: ~4.6 ms (219 ops/s) vs MAYO-1: ~44 µs (22,571 ops/s)
   - 100x slower keygen impacts ephemeral key rotation under load

3. **Constant-time risk**
   - FFT operations are notoriously difficult to make constant-time
   - liboqs Falcon implementation is not formally verified for CT
   - Timing variance observed in audit: stddev ~24 µs on mean ~27 µs
     (may include OS scheduling noise)

4. **Non-deterministic signing**
   - Falcon uses randomized Gaussian sampling during signing
   - Same message + same key produces different signatures each time
   - This is not a correctness issue but prevents deterministic signature
     verification tests across nodes

### Mitigation

- Constant-time audit runs in CI (1,000 verification iterations)
- Cross-architecture test vectors are generated on one arch and verified
  on all others in CI
- Migration to MAYO scheduled upon NIST On-Ramp finalization

## Zero-Knowledge Proof Compatibility

All three families support zero-knowledge proof systems:

| Layer    | ZK Compatibility                               |
|----------|------------------------------------------------|
| MAYO/UOV | MQ-based ZK (e.g., MQ QAP, MiMC hash for R1CS)|
| ML-DSA   | Lattice-based ZK (e.g., Lattice Bulletproofs)  |
| SLH-DSA  | Hash-based ZK (e.g., STARKs using SHA-256)     |

The Speed Layer (MAYO) is preferred for ZK circuits due to the natural
compatibility of MQ equations with R1CS/QAP arithmetization.

## P-Gate Physicalization Constraint

All verification must be deterministic integer arithmetic to guarantee
identical results across:
- x86_64 (Xeon, AMD EPYC)
- aarch64 (Graviton, Apple Silicon)
- riscv64 (SiFive, Esperanto)

This constraint eliminates:
- ❌ Floating-point FFT (Falcon, FN-DSA)
- ❌ Hardware-dependent rounding
- ❌ Non-deterministic verification

This constraint permits:
- ✅ Integer NTT (ML-DSA-65)
- ✅ MQ evaluation (MAYO/UOV)
- ✅ Hash chain traversal (SLH-DSA)

## Future-Proofing

### Cryptographic Agility

The three-layer architecture allows algorithm replacement within each
family independently:

| Family       | Current            | Potential Successor                    |
|--------------|--------------------|----------------------------------------|
| MQ           | MAYO               | UOV (if MAYO fails NIST), PROV (3rd gen) |
| Lattice      | ML-DSA-65          | HAWK (NTRU), Mitaka (Lattice+MQ hybrid)|
| Hash         | SLH-DSA            | ANY second-preimage-resistant hash     |

### Migration Triggers

1. **Cryptanalytic**: Break of any family → instant migration within that
   family only
2. **Standardization**: NIST finalizes MAYO → replace Falcon-512 patch
3. **Performance**: Better scheme within family → hot-swap; the layer
   abstraction isolates the change
4. **Quantum**: Practical quantum computer → all three families remain
   secure by design; no migration needed

## References

- NIST FIPS 204: ML-DSA (Dilithium)
- NIST FIPS 205: SLH-DSA (SPHINCS+)
- NIST FIPS 206: FN-DSA (Falcon) — rejected for Speed Layer
- NIST On-Ramp: MAYO / UOV submissions
- Kylos Arc QPADL SPEC.md: Architecture rationale
- Frontiers in Blockchain (2026): Falcon FFT precision analysis
