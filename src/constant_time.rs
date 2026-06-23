use crate::sig;
use std::time::Instant;

pub const AUDIT_ITERATIONS: u32 = 1_000;

pub struct AuditReport {
    pub algorithm: &'static str,
    pub message_label: &'static str,
    pub verify_count: u32,
    pub all_passed: bool,
    pub timing_min_ns: u64,
    pub timing_max_ns: u64,
    pub timing_mean_ns: f64,
    pub timing_stddev_ns: f64,
}

pub fn audit_deterministic_verification(
    sig: &sig::Sig,
    msg: &[u8],
    label: &'static str,
    signature: &[u8],
    pk: &[u8],
    alg_name: &'static str,
) -> Result<AuditReport, sig::Error> {
    let mut timings = Vec::with_capacity(AUDIT_ITERATIONS as usize);
    let mut all_passed = true;

    for _ in 0..AUDIT_ITERATIONS {
        let start = Instant::now();
        let result = sig.verify(msg, signature, pk);
        let elapsed = start.elapsed().as_nanos() as u64;
        timings.push(elapsed);
        if result.is_err() {
            all_passed = false;
        }
    }

    let verify_count = AUDIT_ITERATIONS;
    let timing_min_ns = *timings.iter().min().unwrap_or(&0);
    let timing_max_ns = *timings.iter().max().unwrap_or(&0);
    let timing_sum: u128 = timings.iter().map(|&t| t as u128).sum();
    let timing_mean_ns = timing_sum as f64 / verify_count as f64;
    let variance: f64 = timings
        .iter()
        .map(|&t| {
            let diff = t as f64 - timing_mean_ns;
            diff * diff
        })
        .sum::<f64>()
        / verify_count as f64;
    let timing_stddev_ns = variance.sqrt();

    Ok(AuditReport {
        algorithm: alg_name,
        message_label: label,
        verify_count,
        all_passed,
        timing_min_ns,
        timing_max_ns,
        timing_mean_ns,
        timing_stddev_ns,
    })
}

pub fn audit_report_csv_row(r: &AuditReport) -> String {
    format!(
        "{},{},{},{},{},{},{:.1},{:.1}\n",
        r.algorithm,
        r.message_label,
        r.verify_count,
        r.all_passed,
        r.timing_min_ns,
        r.timing_max_ns,
        r.timing_mean_ns,
        r.timing_stddev_ns,
    )
}

pub fn audit_report_header() -> String {
    String::from(
        "algorithm,message_label,verify_count,all_passed,timing_min_ns,timing_max_ns,timing_mean_ns,timing_stddev_ns\n",
    )
}

pub fn audit_report_summary(r: &AuditReport) -> String {
    format!(
        "{} [{}]: {} passes/{}, mean={:.0}ns, stddev={:.0}ns, min={}ns, max={}ns  {}",
        r.algorithm,
        r.message_label,
        if r.all_passed { "ALL" } else { "FAIL" },
        r.verify_count,
        r.timing_mean_ns,
        r.timing_stddev_ns,
        r.timing_min_ns,
        r.timing_max_ns,
        if r.all_passed { "OK" } else { "DETERMINISM VIOLATION" },
    )
}

fn run_audit_for_alg(
    alg_name: &'static str,
    verbose: bool,
    reports: &mut Vec<AuditReport>,
) -> Result<(), sig::Error> {
    if !super::algo_is_enabled(alg_name) {
        if verbose {
            eprintln!("  {alg_name} not enabled, skipping");
        }
        return Ok(());
    }
    let sig_obj = super::create_sig_by_name(alg_name)?;
    let (pk, sk) = sig_obj.keypair()?;

    let test_cases: &[(&str, &[u8])] = &[
        ("empty", b""),
        ("short", b"Constant-time audit message for Kylos Arc"),
        ("medium", &[0xABu8; 1024]),
    ];

    for (label, msg) in test_cases {
        let signature = sig_obj.sign(msg, &sk)?;
        let report = audit_deterministic_verification(
            &sig_obj, msg, label, &signature, &pk, alg_name,
        )?;
        if verbose {
            eprintln!("    {}", audit_report_summary(&report));
        }
        reports.push(report);
    }
    Ok(())
}

pub fn run_audit_all(verbose: bool) -> Result<Vec<AuditReport>, sig::Error> {
    let mut reports = Vec::new();

    for level in super::supported_levels() {
        let alg = super::algo_for_level(level).unwrap_or("unknown");
        if verbose {
            eprintln!("  Auditing MAYO-{level}...");
        }
        let sig_obj = super::create_sig(level)?;
        let (pk, sk) = sig_obj.keypair()?;

        let test_cases: &[(&str, &[u8])] = &[
            ("empty", b""),
            ("short", b"Constant-time audit message for Kylos Arc QPADL"),
            ("medium", &[0xABu8; 1024]),
        ];

        for (label, msg) in test_cases {
            let signature = sig_obj.sign(msg, &sk)?;
            let report = audit_deterministic_verification(
                &sig_obj, msg, label, &signature, &pk, alg,
            )?;
            if verbose {
                eprintln!("    {}", audit_report_summary(&report));
            }
            reports.push(report);
        }
    }

    for alg in &[super::FALCON_512, super::FALCON_PADDED_512] {
        if verbose {
            eprintln!("  Auditing {alg}...");
        }
        run_audit_for_alg(alg, verbose, &mut reports)?;
    }

    for alg in &[super::ML_DSA_65, super::SPHINCS_256F] {
        if verbose {
            eprintln!("  Auditing {alg}...");
        }
        run_audit_for_alg(alg, verbose, &mut reports)?;
    }

    Ok(reports)
}
