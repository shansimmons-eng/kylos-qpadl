fn main() {
    kylos_qpadl::init();

    let args: Vec<String> = std::env::args().collect();
    let target = args.get(1).map(|s| s.as_str()).unwrap_or("mayo");

    let run_mayo = target == "mayo" || target == "all";
    let run_falcon = target == "falcon" || target == "all";
    let run_ml_dsa = target == "ml-dsa" || target == "all";
    let run_sphincs = target == "sphincs" || target == "all";
    let run_audit = target == "audit" || target == "all";

    if run_mayo {
        match kylos_qpadl::bench::run_full_bench() {
            Ok(results) => {
                let csv = kylos_qpadl::bench::results_to_csv(&results);
                println!("{}", csv);

                std::fs::write("benchmark_results.csv", &csv)
                    .unwrap_or_else(|e| eprintln!("Warning: could not write CSV: {e}"));

                println!("--- MAYO Summary ---");
                for r in &results {
                    println!(
                        "MAYO-{} {} [{}] {:>8}ns avg  {:>10.0} ops/s",
                        r.level,
                        r.operation,
                        r.message_label,
                        format!("{:.1}", r.avg_ns),
                        r.ops_per_sec
                    );
                }
            }
            Err(e) => {
                eprintln!("MAYO benchmark failed: {e}");
                std::process::exit(1);
            }
        }
    }
    if run_falcon {
        for alg in &[kylos_qpadl::FALCON_512, kylos_qpadl::FALCON_PADDED_512] {
            if !kylos_qpadl::algo_is_enabled(alg) {
                eprintln!("{alg} not enabled, skipping");
                continue;
            }
            match kylos_qpadl::bench::run_falcon_bench(alg) {
                Ok(results) => {
                    let csv = kylos_qpadl::bench::results_to_csv(&results);
                    println!("{}", csv);

                    let fname = format!("benchmark_{}.csv", alg.to_lowercase().replace('-', "_"));
                    std::fs::write(&fname, &csv)
                        .unwrap_or_else(|e| eprintln!("Warning: could not write {fname}: {e}"));

                    println!("--- {alg} Summary ---");
                    for r in &results {
                        if r.sample_count == 0 {
                            continue;
                        }
                        println!(
                            "{} {} [{}] {:>8}ns avg  {:>10.0} ops/s",
                            alg,
                            r.operation,
                            r.message_label,
                            format!("{:.1}", r.avg_ns),
                            r.ops_per_sec
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{alg} benchmark failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
    if run_audit {
        let verbose = args.contains(&"--verbose".to_string());
        match kylos_qpadl::constant_time::run_audit_all(verbose) {
            Ok(reports) => {
                let mut csv = kylos_qpadl::constant_time::audit_report_header();
                for r in &reports {
                    csv.push_str(&kylos_qpadl::constant_time::audit_report_csv_row(r));
                }
                std::fs::write("constant_time_audit.csv", &csv)
                    .unwrap_or_else(|e| eprintln!("Warning: could not write CSV: {e}"));
                println!("{}", csv);
                println!("--- Constant-Time Audit Summary ---");
                for r in &reports {
                    println!("{}", kylos_qpadl::constant_time::audit_report_summary(r));
                }
            }
            Err(e) => {
                eprintln!("Audit failed: {e}");
                std::process::exit(1);
            }
        }
    }
    if run_ml_dsa {
        for alg_label in &[("ML-DSA-65", kylos_qpadl::bench::run_ml_dsa_65_bench)] {
            if !kylos_qpadl::algo_is_enabled(kylos_qpadl::ML_DSA_65) {
                eprintln!("ML-DSA-65 not enabled, skipping");
                continue;
            }
            match alg_label.1() {
                Ok(results) => {
                    let csv = kylos_qpadl::bench::results_to_csv(&results);
                    std::fs::write("benchmark_ml_dsa_65.csv", &csv)
                        .unwrap_or_else(|e| eprintln!("Warning: could not write CSV: {e}"));
                    println!("--- {} Summary ---", alg_label.0);
                    for r in &results {
                        if r.sample_count == 0 {
                            continue;
                        }
                        println!(
                            "{} {} [{}] {:>8}ns avg  {:>10.0} ops/s",
                            alg_label.0,
                            r.operation,
                            r.message_label,
                            format!("{:.1}", r.avg_ns),
                            r.ops_per_sec
                        );
                    }
                }
                Err(e) => {
                    eprintln!("{} benchmark failed: {e}", alg_label.0);
                    std::process::exit(1);
                }
            }
        }
    }
    if run_sphincs {
        if !kylos_qpadl::algo_is_enabled(kylos_qpadl::SPHINCS_256F) {
            eprintln!("SPHINCS+ not enabled, skipping");
        } else {
            match kylos_qpadl::bench::run_sphincs_256f_bench() {
                Ok(results) => {
                    let csv = kylos_qpadl::bench::results_to_csv(&results);
                    std::fs::write("benchmark_sphincs_256f.csv", &csv)
                        .unwrap_or_else(|e| eprintln!("Warning: could not write CSV: {e}"));
                    println!("--- SPHINCS+ Summary ---");
                    for r in &results {
                        if r.sample_count == 0 {
                            continue;
                        }
                        println!(
                            "{} {} [{}] {:>8}ns avg  {:>10.0} ops/s",
                            "SPHINCS+-SHA2-256f-simple",
                            r.operation,
                            r.message_label,
                            format!("{:.1}", r.avg_ns),
                            r.ops_per_sec
                        );
                    }
                }
                Err(e) => {
                    eprintln!("SPHINCS+ benchmark failed: {e}");
                    std::process::exit(1);
                }
            }
        }
    }
    if !run_mayo && !run_falcon && !run_ml_dsa && !run_sphincs && !run_audit {
        eprintln!("Usage: {} <mayo|falcon|ml-dsa|sphincs|audit|all> [--verbose]", args[0]);
        std::process::exit(1);
    }
}
