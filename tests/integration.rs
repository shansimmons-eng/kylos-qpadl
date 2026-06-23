use kylos_qpadl::test_vectors;

#[test]
fn test_supported_levels_non_empty() {
    kylos_qpadl::init();
    let levels = kylos_qpadl::supported_levels();
    assert!(!levels.is_empty(), "At least one MAYO level must be enabled");
}

#[test]
fn test_level_5_sig_creation() {
    kylos_qpadl::init();
    let levels = kylos_qpadl::supported_levels();
    for level in levels {
        let sig = kylos_qpadl::create_sig(level);
        assert!(sig.is_ok(), "MAYO-{level} should create Sig");
    }
}

#[test]
fn test_mayo_generate_and_verify_all_levels() {
    kylos_qpadl::init();
    let sets = test_vectors::generate_all().expect("Generation should succeed");
    assert_eq!(sets.len(), kylos_qpadl::supported_levels().len());
    test_vectors::verify_all(&sets).expect("Verification should pass");
}

#[test]
fn test_mayo_edge_cases() {
    kylos_qpadl::init();
    let sig = kylos_qpadl::create_sig(5).expect("MAYO-5");
    let (pk, sk) = sig.keypair().expect("keypair");

    let empty = b"";
    let empty_sig = sig.sign(empty, &sk).expect("sign empty");
    sig.verify(empty, &empty_sig, &pk).expect("verify empty");

    let one_byte = b"x";
    let one_sig = sig.sign(one_byte, &sk).expect("sign 1B");
    sig.verify(one_byte, &one_sig, &pk).expect("verify 1B");

    let large = vec![0x42u8; 1_048_576];
    let large_sig = sig.sign(&large, &sk).expect("sign 1MB");
    sig.verify(&large, &large_sig, &pk).expect("verify 1MB");
}

#[test]
fn test_mayo_malformed_signature_rejected() {
    kylos_qpadl::init();
    let sig = kylos_qpadl::create_sig(5).expect("MAYO-5");
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"test";
    let valid_sig = sig.sign(msg, &sk).expect("sign");

    let mut bad_sig = valid_sig.clone();
    bad_sig[0] ^= 0xFF;
    let result = sig.verify(msg, &bad_sig, &pk);
    assert!(result.is_err(), "Malformed signature must be rejected");
}

#[test]
fn test_mayo_serialize_roundtrip() {
    use std::path::Path;
    kylos_qpadl::init();
    let sets = test_vectors::generate_all().expect("generate");
    let dir = Path::new("/tmp/kylos_test_serialize");
    let _ = std::fs::remove_dir_all(dir);
    test_vectors::serialize(&sets, dir).expect("serialize");
    let loaded = test_vectors::deserialize(dir).expect("deserialize");
    assert_eq!(sets.len(), loaded.len());
    test_vectors::verify_all(&loaded).expect("verify after roundtrip");
    let _ = std::fs::remove_dir_all(dir);
}

// --- Falcon-512 temporary patch tests ---

fn falcon_enabled() -> bool {
    kylos_qpadl::algo_is_enabled(kylos_qpadl::FALCON_512)
}

#[test]
fn test_falcon_constant_time_audit() {
    kylos_qpadl::init();
    if !falcon_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_512).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"constant-time audit";
    let signature = sig.sign(msg, &sk).expect("sign");

    let report = kylos_qpadl::constant_time::audit_deterministic_verification(
        &sig, msg, "audit", &signature, &pk, kylos_qpadl::FALCON_512,
    )
    .expect("audit");

    assert!(report.all_passed, "Falcon-512 must pass all verification iterations");
    let cv = report.timing_stddev_ns / report.timing_mean_ns;
    if cv > 1.0 {
        eprintln!(
            "WARN: Falcon-512 timing CV={:.2} (stddev={:.0}ns, mean={:.0}ns) — \
             high variance may indicate non-constant-time behavior. \
             For definitive analysis, run on frequency-locked hardware with performance counters.",
            cv, report.timing_stddev_ns, report.timing_mean_ns,
        );
    }
}

#[test]
fn test_falcon_512_sig_creation() {
    kylos_qpadl::init();
    if !falcon_enabled() {
        eprintln!("Falcon-512 not enabled, skipping");
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_512);
    assert!(sig.is_ok(), "Falcon-512 should create Sig");
}

#[test]
fn test_falcon_padded_512_sig_creation() {
    kylos_qpadl::init();
    if !kylos_qpadl::algo_is_enabled(kylos_qpadl::FALCON_PADDED_512) {
        eprintln!("Falcon-padded-512 not enabled, skipping");
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_PADDED_512);
    assert!(sig.is_ok(), "Falcon-padded-512 should create Sig");
}

#[test]
fn test_falcon_generate_and_verify() {
    kylos_qpadl::init();
    let sets = test_vectors::generate_falcon_vectors().expect("Falcon generation");
    if sets.is_empty() {
        eprintln!("No Falcon algorithms enabled, skipping");
        return;
    }
    test_vectors::verify_falcon_vectors(&sets).expect("Falcon verification should pass");
    for set in &sets {
        assert!(!set.keypair.public_key.is_empty());
        assert!(!set.keypair.secret_key.is_empty());
        for sig in &set.signatures {
            assert!(!sig.signature.is_empty());
        }
    }
}

#[test]
fn test_falcon_512_key_sizes() {
    kylos_qpadl::init();
    if !falcon_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_512).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    assert_eq!(pk.len(), 897, "Falcon-512 pk should be 897 bytes");
    assert_eq!(sk.len(), 1281, "Falcon-512 sk should be 1281 bytes");
}

#[test]
fn test_falcon_edge_cases() {
    kylos_qpadl::init();
    if !falcon_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_512).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");

    let empty = b"";
    let empty_sig = sig.sign(empty, &sk).expect("sign empty");
    sig.verify(empty, &empty_sig, &pk).expect("verify empty");
    assert!(!empty_sig.is_empty(), "Falcon-512 signature should not be empty");

    let large = vec![0x42u8; 1_048_576];
    let large_sig = sig.sign(&large, &sk).expect("sign 1MB");
    sig.verify(&large, &large_sig, &pk).expect("verify 1MB");
}

#[test]
fn test_falcon_malformed_rejected() {
    kylos_qpadl::init();
    if !falcon_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::FALCON_512).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"falcon test";
    let valid_sig = sig.sign(msg, &sk).expect("sign");

    let mut bad_sig = valid_sig.clone();
    bad_sig[0] ^= 0xFF;
    let result = sig.verify(msg, &bad_sig, &pk);
    assert!(result.is_err(), "Malformed Falcon signature must be rejected");
}

// --- ML-DSA-65 (Primary Layer, Lattice) ---

fn ml_dsa_enabled() -> bool {
    kylos_qpadl::algo_is_enabled(kylos_qpadl::ML_DSA_65)
}

#[test]
fn test_ml_dsa_65_sig_creation() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        eprintln!("ML-DSA-65 not enabled, skipping");
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::ML_DSA_65);
    assert!(sig.is_ok(), "ML-DSA-65 should create Sig");
}

#[test]
fn test_ml_dsa_65_generate_and_verify() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        eprintln!("ML-DSA-65 not enabled, skipping");
        return;
    }
    let set = test_vectors::generate_ml_dsa_vectors().expect("ML-DSA-65 generation");
    test_vectors::verify_ml_dsa_vectors(&set).expect("ML-DSA-65 verification should pass");
    assert!(!set.keypair.public_key.is_empty());
    assert!(!set.keypair.secret_key.is_empty());
    for sig in &set.signatures {
        assert!(!sig.signature.is_empty());
    }
}

#[test]
fn test_ml_dsa_65_edge_cases() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::ML_DSA_65).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");

    let empty = b"";
    let empty_sig = sig.sign(empty, &sk).expect("sign empty");
    sig.verify(empty, &empty_sig, &pk).expect("verify empty");

    let large = vec![0x42u8; 1_048_576];
    let large_sig = sig.sign(&large, &sk).expect("sign 1MB");
    sig.verify(&large, &large_sig, &pk).expect("verify 1MB");
}

#[test]
fn test_ml_dsa_65_malformed_rejected() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::ML_DSA_65).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"ml-dsa test";
    let valid_sig = sig.sign(msg, &sk).expect("sign");

    let mut bad_sig = valid_sig.clone();
    bad_sig[0] ^= 0xFF;
    let result = sig.verify(msg, &bad_sig, &pk);
    assert!(result.is_err(), "Malformed ML-DSA-65 signature must be rejected");
}

#[test]
fn test_ml_dsa_65_key_sizes() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::ML_DSA_65).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    // ML-DSA-65: pk=1952B, sk=4000B, sig=3309B (FIPS 204)
    assert!(pk.len() >= 1900 && pk.len() <= 2000, "ML-DSA-65 pk={}", pk.len());
    assert!(sk.len() >= 3900 && sk.len() <= 4100, "ML-DSA-65 sk={}", sk.len());
}

// --- SPHINCS+ (Anchor Layer, Hash-Based) ---

fn sphincs_enabled() -> bool {
    kylos_qpadl::algo_is_enabled(kylos_qpadl::SPHINCS_256F)
}

#[test]
fn test_sphincs_sig_creation() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        eprintln!("SPHINCS+ not enabled, skipping");
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::SPHINCS_256F);
    assert!(sig.is_ok(), "SPHINCS+ should create Sig");
}

#[test]
fn test_sphincs_generate_and_verify() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        eprintln!("SPHINCS+ not enabled, skipping");
        return;
    }
    let set = test_vectors::generate_sphincs_vectors().expect("SPHINCS+ generation");
    test_vectors::verify_sphincs_vectors(&set).expect("SPHINCS+ verification should pass");
    assert!(!set.keypair.public_key.is_empty());
    assert!(!set.keypair.secret_key.is_empty());
    for sig in &set.signatures {
        assert!(!sig.signature.is_empty());
    }
}

#[test]
fn test_sphincs_edge_cases() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::SPHINCS_256F).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");

    let empty = b"";
    let empty_sig = sig.sign(empty, &sk).expect("sign empty");
    sig.verify(empty, &empty_sig, &pk).expect("verify empty");

    let large = vec![0x42u8; 1_048_576];
    let large_sig = sig.sign(&large, &sk).expect("sign 1MB");
    sig.verify(&large, &large_sig, &pk).expect("verify 1MB");
}

#[test]
fn test_sphincs_malformed_rejected() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::SPHINCS_256F).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"sphincs test";
    let valid_sig = sig.sign(msg, &sk).expect("sign");

    let mut bad_sig = valid_sig.clone();
    bad_sig[0] ^= 0xFF;
    let result = sig.verify(msg, &bad_sig, &pk);
    assert!(result.is_err(), "Malformed SPHINCS+ signature must be rejected");
}

#[test]
fn test_sphincs_key_sizes() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::SPHINCS_256F).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    // SPHINCS+-SHA2-256f-simple: pk=64B, sk=128B, sig=~50KB
    assert_eq!(pk.len(), 64, "SPHINCS+ pk should be 64 bytes");
    assert_eq!(sk.len(), 128, "SPHINCS+ sk should be 128 bytes");
}

#[test]
fn test_ml_dsa_65_constant_time_audit() {
    kylos_qpadl::init();
    if !ml_dsa_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::ML_DSA_65).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"constant-time audit";
    let signature = sig.sign(msg, &sk).expect("sign");

    let report = kylos_qpadl::constant_time::audit_deterministic_verification(
        &sig, msg, "audit", &signature, &pk, kylos_qpadl::ML_DSA_65,
    )
    .expect("audit");

    assert!(report.all_passed, "ML-DSA-65 must pass all verification iterations");
}

#[test]
fn test_sphincs_constant_time_audit() {
    kylos_qpadl::init();
    if !sphincs_enabled() {
        return;
    }
    let sig = kylos_qpadl::create_sig_by_name(kylos_qpadl::SPHINCS_256F).unwrap();
    let (pk, sk) = sig.keypair().expect("keypair");
    let msg = b"constant-time audit";
    let signature = sig.sign(msg, &sk).expect("sign");

    let report = kylos_qpadl::constant_time::audit_deterministic_verification(
        &sig, msg, "audit", &signature, &pk, kylos_qpadl::SPHINCS_256F,
    )
    .expect("audit");

    assert!(report.all_passed, "SPHINCS+ must pass all verification iterations");
}
