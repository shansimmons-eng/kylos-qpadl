use std::path::Path;

fn main() {
    kylos_qpadl::init();

    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("generate");
    let target = args.get(2).map(|s| s.as_str()).unwrap_or("mayo");

    let run_mayo = target == "mayo" || target == "all";
    let run_falcon = target == "falcon" || target == "all";
    let run_ml_dsa = target == "ml-dsa" || target == "all";
    let run_sphincs = target == "sphincs" || target == "all";

    match cmd {
        "generate" => {
            let vec_dir = Path::new("test_vectors");

            if run_mayo {
                match kylos_qpadl::test_vectors::generate_all() {
                    Ok(sets) => {
                        println!("Generated {} MAYO test vector sets", sets.len());
                        for set in &sets {
                            println!(
                                "  MAYO-{}: pk={}B sk={}B sigs={}",
                                set.keypair.level,
                                set.keypair.public_key.len(),
                                set.keypair.secret_key.len(),
                                set.signatures.len()
                            );
                        }
                        if let Err(e) = kylos_qpadl::test_vectors::serialize(&sets, vec_dir) {
                            eprintln!("Serialize error: {e}");
                            std::process::exit(1);
                        }
                        println!("MAYO test vectors written to {}/", vec_dir.display());
                    }
                    Err(e) => {
                        eprintln!("MAYO generation failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if run_falcon {
                match kylos_qpadl::test_vectors::generate_falcon_vectors() {
                    Ok(sets) => {
                        println!("Generated {} Falcon test vector sets", sets.len());
                        for set in &sets {
                            println!(
                                "  {}: pk={}B sk={}B sigs={}",
                                set.keypair.algorithm,
                                set.keypair.public_key.len(),
                                set.keypair.secret_key.len(),
                                set.signatures.len()
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Falcon generation failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if run_ml_dsa {
                match kylos_qpadl::test_vectors::generate_ml_dsa_vectors() {
                    Ok(set) => {
                        println!(
                            "ML-DSA-65: pk={}B sk={}B sigs={}",
                            set.keypair.public_key.len(),
                            set.keypair.secret_key.len(),
                            set.signatures.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ML-DSA-65 generation failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if run_sphincs {
                match kylos_qpadl::test_vectors::generate_sphincs_vectors() {
                    Ok(set) => {
                        println!(
                            "SPHINCS+: pk={}B sk={}B sigs={}",
                            set.keypair.public_key.len(),
                            set.keypair.secret_key.len(),
                            set.signatures.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("SPHINCS+ generation failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if !run_mayo && !run_falcon && !run_ml_dsa && !run_sphincs {
                eprintln!("Unknown target: {target}. Use mayo|falcon|ml-dsa|sphincs|all");
                std::process::exit(1);
            }
        }
        "verify" => {
            let vec_dir = Path::new("test_vectors");

            if run_mayo {
                match kylos_qpadl::test_vectors::deserialize(vec_dir) {
                    Ok(sets) => {
                        if sets.is_empty() {
                            eprintln!("No MAYO test vectors found in {}/", vec_dir.display());
                            std::process::exit(1);
                        }
                        for set in &sets {
                            let level = set.keypair.level;
                            if let Err(e) = set.verify_one() {
                                eprintln!("MAYO-{level} verify_one failed: {e}");
                                std::process::exit(1);
                            }
                            if let Err(e) = set.verify_malformed_rejected() {
                                eprintln!("MAYO-{level} malformed rejection failed: {e}");
                                std::process::exit(1);
                            }
                            println!("MAYO-{level}: all checks passed");
                        }
                        if let Err(e) = kylos_qpadl::test_vectors::verify_all(&sets) {
                            eprintln!("Full verify_all failed: {e}");
                            std::process::exit(1);
                        }
                        println!("All MAYO test vectors verified OK");
                    }
                    Err(e) => {
                        eprintln!("Deserialize error: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if run_falcon {
                kylos_qpadl::test_vectors::generate_falcon_vectors()
                    .map_err(|e| format!("Falcon generation: {e}"))
                    .and_then(|sets| {
                        kylos_qpadl::test_vectors::verify_falcon_vectors(&sets)
                            .map(|_| sets)
                    })
                    .map(|sets| {
                        println!("Falcon test vectors: all checks passed");
                        for set in &sets {
                            println!(
                                "  {}: pk={}B sk={}B sigs={} verified OK",
                                set.keypair.algorithm,
                                set.keypair.public_key.len(),
                                set.keypair.secret_key.len(),
                                set.signatures.len()
                            );
                        }
                    })
                    .unwrap_or_else(|e| {
                        eprintln!("Falcon verification failed: {e}");
                        std::process::exit(1);
                    });
            }
            if run_ml_dsa {
                match kylos_qpadl::test_vectors::generate_ml_dsa_vectors() {
                    Ok(set) => {
                        if let Err(e) = kylos_qpadl::test_vectors::verify_ml_dsa_vectors(&set) {
                            eprintln!("ML-DSA-65 verification failed: {e}");
                            std::process::exit(1);
                        }
                        println!(
                            "ML-DSA-65: pk={}B sk={}B sigs={} verified OK",
                            set.keypair.public_key.len(),
                            set.keypair.secret_key.len(),
                            set.signatures.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("ML-DSA-65 verification failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if run_sphincs {
                match kylos_qpadl::test_vectors::generate_sphincs_vectors() {
                    Ok(set) => {
                        if let Err(e) = kylos_qpadl::test_vectors::verify_sphincs_vectors(&set) {
                            eprintln!("SPHINCS+ verification failed: {e}");
                            std::process::exit(1);
                        }
                        println!(
                            "SPHINCS+: pk={}B sk={}B sigs={} verified OK",
                            set.keypair.public_key.len(),
                            set.keypair.secret_key.len(),
                            set.signatures.len()
                        );
                    }
                    Err(e) => {
                        eprintln!("SPHINCS+ verification failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
            if !run_mayo && !run_falcon && !run_ml_dsa && !run_sphincs {
                eprintln!("Unknown target: {target}. Use mayo|falcon|ml-dsa|sphincs|all");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Usage: {} <generate|verify> [mayo|falcon|ml-dsa|sphincs|all]", args[0]);
            std::process::exit(1);
        }
    }
}
