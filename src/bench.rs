use crate::sig;
use std::time::Instant;

pub const ITERATIONS: u64 = 10_000;
pub const WARMUP: u64 = 500;

pub struct BenchResult {
    pub level: u8,
    pub operation: &'static str,
    pub message_label: &'static str,
    pub message_size: usize,
    pub total_ns: u128,
    pub avg_ns: f64,
    pub ops_per_sec: f64,
    pub sample_count: u64,
}

pub fn bench_keygen(level: u8) -> Result<Vec<BenchResult>, sig::Error> {
    let sig = super::create_sig(level)?;

    for _ in 0..WARMUP {
        let _ = sig.keypair()?;
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.keypair()?;
    }
    let elapsed = start.elapsed().as_nanos();

    Ok(vec![BenchResult {
        level,
        operation: "keygen",
        message_label: "N/A",
        message_size: 0,
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    }])
}

pub fn bench_sign(
    sig: &sig::Sig,
    msg: &[u8],
    label: &'static str,
    sk: &[u8],
) -> Result<BenchResult, sig::Error> {
    for _ in 0..WARMUP {
        let _ = sig.sign(msg, sk)?;
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.sign(msg, sk)?;
    }
    let elapsed = start.elapsed().as_nanos();

    Ok(BenchResult {
        level: 0,
        operation: "sign",
        message_label: label,
        message_size: msg.len(),
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    })
}

pub fn bench_verify(
    sig: &sig::Sig,
    msg: &[u8],
    label: &'static str,
    signature: &[u8],
    pk: &[u8],
) -> Result<BenchResult, sig::Error> {
    for _ in 0..WARMUP {
        let _ = sig.verify(msg, signature, pk);
    }

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.verify(msg, signature, pk)?;
    }
    let elapsed = start.elapsed().as_nanos();

    Ok(BenchResult {
        level: 0,
        operation: "verify",
        message_label: label,
        message_size: msg.len(),
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    })
}

pub fn run_full_bench() -> Result<Vec<BenchResult>, sig::Error> {
    let mut results = Vec::new();
    let messages: &[(&str, &[u8])] = &[
        ("empty", b""),
        ("small", b"Hello Kylos Arc"),
        ("medium", &[0xABu8; 1024]),
        ("large", &[0xCDu8; 65_536]),
    ];

    for level in super::supported_levels() {
        let sig = super::create_sig(level)?;

        results.extend(bench_keygen(level)?);

        let (pk, sk) = sig.keypair()?;

        for (label, msg) in messages {
            let bench = bench_sign(&sig, msg, label, &sk)?;
            results.push(BenchResult {
                level,
                ..bench
            });

            let sig_bytes = sig.sign(msg, &sk)?;
            let bench = bench_verify(&sig, msg, label, &sig_bytes, &pk)?;
            results.push(BenchResult {
                level,
                ..bench
            });
        }
    }

    Ok(results)
}

pub fn run_falcon_bench(alg_name: &'static str) -> Result<Vec<BenchResult>, sig::Error> {
    let mut results = Vec::new();
    let messages: &[(&str, &[u8])] = &[
        ("empty", b""),
        ("small", b"Hello Kylos Arc"),
        ("medium", &[0xABu8; 1024]),
        ("large", &[0xCDu8; 65_536]),
    ];

    let sig = super::create_sig_by_name(alg_name)?;
    let level_code: u8 = if alg_name.contains("1024") { 5 } else { 1 };

    results.push(BenchResult {
        level: level_code,
        operation: "keygen",
        message_label: alg_name,
        message_size: 0,
        total_ns: 0,
        avg_ns: 0.0,
        ops_per_sec: 0.0,
        sample_count: 0,
    });

    for _ in 0..WARMUP {
        let _ = sig.keypair()?;
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.keypair()?;
    }
    let elapsed = start.elapsed().as_nanos();
    results.push(BenchResult {
        level: level_code,
        operation: "keygen",
        message_label: alg_name,
        message_size: 0,
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    });

    let (pk, sk) = sig.keypair()?;

    for (label, msg) in messages {
        let bench = bench_sign(&sig, msg, label, &sk)?;
        results.push(BenchResult {
            level: level_code,
            ..bench
        });

        let sig_bytes = sig.sign(msg, &sk)?;
        let bench = bench_verify(&sig, msg, label, &sig_bytes, &pk)?;
        results.push(BenchResult {
            level: level_code,
            ..bench
        });
    }

    Ok(results)
}

pub fn run_ml_dsa_65_bench() -> Result<Vec<BenchResult>, sig::Error> {
    let mut results = Vec::new();
    let messages: &[(&str, &[u8])] = &[
        ("empty", b""),
        ("small", b"Hello Kylos Arc"),
        ("medium", &[0xABu8; 1024]),
        ("large", &[0xCDu8; 65_536]),
    ];

    let sig = super::create_sig_by_name(super::ML_DSA_65)?;

    results.push(BenchResult {
        level: 3,
        operation: "keygen",
        message_label: "ML-DSA-65",
        message_size: 0,
        total_ns: 0,
        avg_ns: 0.0,
        ops_per_sec: 0.0,
        sample_count: 0,
    });

    for _ in 0..WARMUP {
        let _ = sig.keypair()?;
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.keypair()?;
    }
    let elapsed = start.elapsed().as_nanos();
    results.push(BenchResult {
        level: 3,
        operation: "keygen",
        message_label: "ML-DSA-65",
        message_size: 0,
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    });

    let (pk, sk) = sig.keypair()?;

    for (msg_label, msg) in messages {
        let bench = bench_sign(&sig, msg, msg_label, &sk)?;
        results.push(BenchResult {
            level: 3,
            ..bench
        });

        let sig_bytes = sig.sign(msg, &sk)?;
        let bench = bench_verify(&sig, msg, msg_label, &sig_bytes, &pk)?;
        results.push(BenchResult {
            level: 3,
            ..bench
        });
    }

    Ok(results)
}

pub fn run_sphincs_256f_bench() -> Result<Vec<BenchResult>, sig::Error> {
    let mut results = Vec::new();
    let messages: &[(&str, &[u8])] = &[
        ("empty", b""),
        ("small", b"Hello Kylos Arc"),
        ("medium", &[0xABu8; 1024]),
        ("large", &[0xCDu8; 65_536]),
    ];

    let sig = super::create_sig_by_name(super::SPHINCS_256F)?;

    results.push(BenchResult {
        level: 5,
        operation: "keygen",
        message_label: "SPHINCS+-SHA2-256f-simple",
        message_size: 0,
        total_ns: 0,
        avg_ns: 0.0,
        ops_per_sec: 0.0,
        sample_count: 0,
    });

    for _ in 0..WARMUP {
        let _ = sig.keypair()?;
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = sig.keypair()?;
    }
    let elapsed = start.elapsed().as_nanos();
    results.push(BenchResult {
        level: 5,
        operation: "keygen",
        message_label: "SPHINCS+-SHA2-256f-simple",
        message_size: 0,
        total_ns: elapsed,
        avg_ns: elapsed as f64 / ITERATIONS as f64,
        ops_per_sec: 1_000_000_000.0 / (elapsed as f64 / ITERATIONS as f64),
        sample_count: ITERATIONS,
    });

    let (pk, sk) = sig.keypair()?;

    for (msg_label, msg) in messages {
        let bench = bench_sign(&sig, msg, msg_label, &sk)?;
        results.push(BenchResult {
            level: 5,
            ..bench
        });

        let sig_bytes = sig.sign(msg, &sk)?;
        let bench = bench_verify(&sig, msg, msg_label, &sig_bytes, &pk)?;
        results.push(BenchResult {
            level: 5,
            ..bench
        });
    }

    Ok(results)
}

pub fn results_to_csv(results: &[BenchResult]) -> String {
    let mut csv =
        String::from("level,operation,message_label,message_size,total_ns,avg_ns,ops_per_sec\n");
    for r in results {
        csv.push_str(&format!(
            "{},{},{},{},{},{:.1},{:.1}\n",
            r.level,
            r.operation,
            r.message_label,
            r.message_size,
            r.total_ns,
            r.avg_ns,
            r.ops_per_sec
        ));
    }
    csv
}
