use crate::sig;
use std::path::Path;

static TEST_MESSAGES: &[(&str, &[u8])] = &[
    ("empty", b""),
    ("short", b"Hello, Kylos Arc QPADL MAYO test vector"),
    (
        "medium",
        b"Post-quantum signature verification must be deterministic integer arithmetic only. No floating-point, no FFT. This is the Speed Layer of the Sovereign Mirror BFT mesh.",
    ),
];

pub const MAX_MESSAGE_SIZE: usize = 1_048_576;

#[derive(Clone)]
pub struct MayoKeypair {
    pub level: u8,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

#[derive(Clone)]
pub struct MayoSignature {
    pub level: u8,
    pub message: Vec<u8>,
    pub message_label: String,
    pub signature: Vec<u8>,
}

#[derive(Clone)]
pub struct TestVectorSet {
    pub keypair: MayoKeypair,
    pub signatures: Vec<MayoSignature>,
}

impl TestVectorSet {
    pub fn generate_one(level: u8) -> Result<Self, sig::Error> {
        let sig = super::create_sig(level)?;
        let (pk, sk) = sig.keypair()?;

        let mut signatures = Vec::new();
        for (label, msg) in TEST_MESSAGES {
            let signature = sig.sign(msg, &sk)?;
            signatures.push(MayoSignature {
                level,
                message: msg.to_vec(),
                message_label: label.to_string(),
                signature,
            });
        }

        Ok(TestVectorSet {
            keypair: MayoKeypair {
                level,
                public_key: pk,
                secret_key: sk,
            },
            signatures,
        })
    }

    pub fn verify_one(&self) -> Result<(), String> {
        let sig = super::create_sig(self.keypair.level)
            .map_err(|e| format!("Failed to create Sig: {e}"))?;

        for ts in &self.signatures {
            sig.verify(&ts.message, &ts.signature, &self.keypair.public_key)
                .map_err(|e| format!("Verify failed for '{}': {e}", ts.message_label))?;

            let re_signed = sig
                .sign(&ts.message, &self.keypair.secret_key)
                .map_err(|e| format!("Re-sign failed for '{}': {e}", ts.message_label))?;
            sig.verify(&ts.message, &re_signed, &self.keypair.public_key)
                .map_err(|e| format!("Re-verify failed for '{}': {e}", ts.message_label))?;
        }
        Ok(())
    }

    pub fn verify_malformed_rejected(&self) -> Result<(), String> {
        let sig = super::create_sig(self.keypair.level)
            .map_err(|e| format!("Failed to create Sig: {e}"))?;

        for ts in &self.signatures {
            let mut bad_sig = ts.signature.clone();
            if bad_sig.is_empty() {
                continue;
            }
            bad_sig[0] ^= 0xFF;
            if sig.verify(&ts.message, &bad_sig, &self.keypair.public_key).is_ok() {
                return Err(format!(
                    "Malformed signature accepted for '{}'",
                    ts.message_label
                ));
            }
        }
        Ok(())
    }
}

pub fn generate_all() -> Result<Vec<TestVectorSet>, sig::Error> {
    let levels = super::supported_levels();
    if levels.is_empty() {
        return Err(sig::UNKNOWN_ALG);
    }
    levels.into_iter().map(TestVectorSet::generate_one).collect()
}

pub fn verify_all(sets: &[TestVectorSet]) -> Result<(), String> {
    for set in sets {
        let label = format!("MAYO-{}", set.keypair.level);
        set.verify_one()
            .map_err(|e| format!("{label}: {e}"))?;
        set.verify_malformed_rejected()
            .map_err(|e| format!("{label}: {e}"))?;
        let sig = super::create_sig(set.keypair.level).unwrap();

        let large_msg = vec![0xABu8; MAX_MESSAGE_SIZE];
        let large_sig = sig
            .sign(&large_msg, &set.keypair.secret_key)
            .map_err(|e| format!("{label}: large msg sign failed: {e}"))?;
        sig.verify(&large_msg, &large_sig, &set.keypair.public_key)
            .map_err(|e| format!("{label}: large msg verify failed: {e}"))?;

        let msg_1b = b"x";
        let sig_1b = sig
            .sign(msg_1b, &set.keypair.secret_key)
            .map_err(|e| format!("{label}: 1-byte sign failed: {e}"))?;
        sig.verify(msg_1b, &sig_1b, &set.keypair.public_key)
            .map_err(|e| format!("{label}: 1-byte verify failed: {e}"))?;
    }
    Ok(())
}

pub fn serialize(sets: &[TestVectorSet], dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("mkdir: {e}"))?;
    for set in sets {
        let base = dir.join(format!("mayo{}", set.keypair.level));
        std::fs::write(base.with_extension("pk"), &set.keypair.public_key)
            .map_err(|e| format!("write pk: {e}"))?;
        std::fs::write(base.with_extension("sk"), &set.keypair.secret_key)
            .map_err(|e| format!("write sk: {e}"))?;

        for ts in &set.signatures {
            let name = format!("mayo{}_{}.sig", set.keypair.level, ts.message_label);
            std::fs::write(dir.join(&name), &ts.signature)
                .map_err(|e| format!("write sig: {e}"))?;
        }
    }
    Ok(())
}

pub fn deserialize(dir: &Path) -> Result<Vec<TestVectorSet>, String> {
    let mut sets = Vec::new();
    for level in [1u8, 3, 5] {
        let pk_path = dir.join(format!("mayo{level}.pk"));
        let sk_path = dir.join(format!("mayo{level}.sk"));
        if !pk_path.exists() || !sk_path.exists() {
            continue;
        }
        let public_key = std::fs::read(&pk_path).map_err(|e| format!("read pk: {e}"))?;
        let secret_key = std::fs::read(&sk_path).map_err(|e| format!("read sk: {e}"))?;

        let mut signatures = Vec::new();
        for (label, msg) in TEST_MESSAGES {
            let sig_path = dir.join(format!("mayo{level}_{label}.sig"));
            if sig_path.exists() {
                let signature = std::fs::read(&sig_path).map_err(|e| format!("read sig: {e}"))?;
                signatures.push(MayoSignature {
                    level,
                    message: msg.to_vec(),
                    message_label: label.to_string(),
                    signature,
                });
            }
        }

        sets.push(TestVectorSet {
            keypair: MayoKeypair {
                level,
                public_key,
                secret_key,
            },
            signatures,
        });
    }
    Ok(sets)
}

// --- Falcon-512 temporary patch test vectors ---

static FALCON_MESSAGES: &[(&str, &[u8])] = &[
    ("empty", b""),
    (
        "short",
        b"Falcon-512 temporary patch for Kylos Arc QPADL",
    ),
    (
        "medium",
        b"Floating-point FFT verification risk documented for heterogeneous BFT consensus. Temporary deployment while MAYO matures through NIST on-ramp.",
    ),
];

pub struct FalconKeypair {
    pub algorithm: &'static str,
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

pub struct FalconSignature {
    pub algorithm: &'static str,
    pub message: Vec<u8>,
    pub message_label: String,
    pub signature: Vec<u8>,
}

pub struct FalconTestVectorSet {
    pub keypair: FalconKeypair,
    pub signatures: Vec<FalconSignature>,
}

impl FalconTestVectorSet {
    pub fn generate(alg_name: &'static str) -> Result<Self, sig::Error> {
        if !super::algo_is_enabled(alg_name) {
            return Err(sig::UNKNOWN_ALG);
        }
        let sig = super::create_sig_by_name(alg_name)?;
        let (pk, sk) = sig.keypair()?;

        let mut signatures = Vec::new();
        for (label, msg) in FALCON_MESSAGES {
            let signature = sig.sign(msg, &sk)?;
            signatures.push(FalconSignature {
                algorithm: alg_name,
                message: msg.to_vec(),
                message_label: label.to_string(),
                signature,
            });
        }

        Ok(FalconTestVectorSet {
            keypair: FalconKeypair {
                algorithm: alg_name,
                public_key: pk,
                secret_key: sk,
            },
            signatures,
        })
    }

    pub fn verify_one(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(self.keypair.algorithm)
            .map_err(|e| format!("Failed to create {}: {e}", self.keypair.algorithm))?;

        for ts in &self.signatures {
            sig.verify(&ts.message, &ts.signature, &self.keypair.public_key)
                .map_err(|e| {
                    format!("{} verify '{}': {e}", self.keypair.algorithm, ts.message_label)
                })?;
        }
        Ok(())
    }

    pub fn verify_malformed_rejected(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(self.keypair.algorithm)
            .map_err(|e| format!("Failed to create {}: {e}", self.keypair.algorithm))?;

        for ts in &self.signatures {
            let mut bad_sig = ts.signature.clone();
            if bad_sig.is_empty() {
                continue;
            }
            bad_sig[0] ^= 0xFF;
            if sig.verify(&ts.message, &bad_sig, &self.keypair.public_key).is_ok() {
                return Err(format!(
                    "{} accepted malformed sig for '{}'",
                    self.keypair.algorithm, ts.message_label
                ));
            }
        }
        Ok(())
    }

    pub fn large_msg_test(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(self.keypair.algorithm)
            .map_err(|e| format!("Failed to create {}: {e}", self.keypair.algorithm))?;
        let large_msg = vec![0xABu8; MAX_MESSAGE_SIZE];
        let large_sig = sig
            .sign(&large_msg, &self.keypair.secret_key)
            .map_err(|e| format!("{} large msg sign: {e}", self.keypair.algorithm))?;
        sig.verify(&large_msg, &large_sig, &self.keypair.public_key)
            .map_err(|e| format!("{} large msg verify: {e}", self.keypair.algorithm))
    }
}

pub fn generate_falcon_vectors() -> Result<Vec<FalconTestVectorSet>, sig::Error> {
    let algos: &[&str] = &[
        super::FALCON_512,
        super::FALCON_PADDED_512,
    ];
    let mut sets = Vec::new();
    for alg in algos {
        if super::algo_is_enabled(alg) {
            sets.push(FalconTestVectorSet::generate(alg)?);
        }
    }
    Ok(sets)
}

pub fn verify_falcon_vectors(sets: &[FalconTestVectorSet]) -> Result<(), String> {
    for set in sets {
        set.verify_one().map_err(|e| format!("{}: {e}", set.keypair.algorithm))?;
        set.verify_malformed_rejected()
            .map_err(|e| format!("{}: {e}", set.keypair.algorithm))?;
        set.large_msg_test()
            .map_err(|e| format!("{}: {e}", set.keypair.algorithm))?;
    }
    Ok(())
}

// --- ML-DSA-65 (Primary Layer, Lattice) ---

static ML_DSA_MESSAGES: &[(&str, &[u8])] = &[
    ("empty", b""),
    (
        "short",
        b"ML-DSA-65 Primary Layer for Kylos Arc QPADL",
    ),
    (
        "medium",
        b"Lattice-based Module-LWE signature. Integer NTT, constant-time. FIPS 204 standardized. Fallback when MQ is compromised.",
    ),
];

pub struct MlDsaKeypair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

pub struct MlDsaSignature {
    pub message: Vec<u8>,
    pub message_label: String,
    pub signature: Vec<u8>,
}

pub struct MlDsaTestVectorSet {
    pub keypair: MlDsaKeypair,
    pub signatures: Vec<MlDsaSignature>,
}

impl MlDsaTestVectorSet {
    pub fn generate() -> Result<Self, sig::Error> {
        if !super::algo_is_enabled(super::ML_DSA_65) {
            return Err(sig::UNKNOWN_ALG);
        }
        let sig = super::create_sig_by_name(super::ML_DSA_65)?;
        let (pk, sk) = sig.keypair()?;

        let mut signatures = Vec::new();
        for (label, msg) in ML_DSA_MESSAGES {
            let signature = sig.sign(msg, &sk)?;
            signatures.push(MlDsaSignature {
                message: msg.to_vec(),
                message_label: label.to_string(),
                signature,
            });
        }

        Ok(MlDsaTestVectorSet {
            keypair: MlDsaKeypair {
                public_key: pk,
                secret_key: sk,
            },
            signatures,
        })
    }

    pub fn verify_one(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::ML_DSA_65)
            .map_err(|e| format!("Failed to create ML-DSA-65: {e}"))?;

        for ts in &self.signatures {
            sig.verify(&ts.message, &ts.signature, &self.keypair.public_key)
                .map_err(|e| format!("ML-DSA-65 verify '{}': {e}", ts.message_label))?;
        }
        Ok(())
    }

    pub fn verify_malformed_rejected(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::ML_DSA_65)
            .map_err(|e| format!("Failed to create ML-DSA-65: {e}"))?;

        for ts in &self.signatures {
            let mut bad_sig = ts.signature.clone();
            if bad_sig.is_empty() {
                continue;
            }
            bad_sig[0] ^= 0xFF;
            if sig.verify(&ts.message, &bad_sig, &self.keypair.public_key).is_ok() {
                return Err(format!("ML-DSA-65 accepted malformed sig for '{}'", ts.message_label));
            }
        }
        Ok(())
    }

    pub fn large_msg_test(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::ML_DSA_65)
            .map_err(|e| format!("Failed to create ML-DSA-65: {e}"))?;
        let large_msg = vec![0xABu8; MAX_MESSAGE_SIZE];
        let large_sig = sig
            .sign(&large_msg, &self.keypair.secret_key)
            .map_err(|e| format!("ML-DSA-65 large msg sign: {e}"))?;
        sig.verify(&large_msg, &large_sig, &self.keypair.public_key)
            .map_err(|e| format!("ML-DSA-65 large msg verify: {e}"))
    }
}

pub fn generate_ml_dsa_vectors() -> Result<MlDsaTestVectorSet, sig::Error> {
    MlDsaTestVectorSet::generate()
}

pub fn verify_ml_dsa_vectors(set: &MlDsaTestVectorSet) -> Result<(), String> {
    set.verify_one().map_err(|e| format!("ML-DSA-65: {e}"))?;
    set.verify_malformed_rejected()
        .map_err(|e| format!("ML-DSA-65: {e}"))?;
    set.large_msg_test()
        .map_err(|e| format!("ML-DSA-65: {e}"))
}

// --- SPHINCS+ (Anchor Layer, Hash-Based) ---

static SPHINCS_MESSAGES: &[(&str, &[u8])] = &[
    ("empty", b""),
    (
        "short",
        b"SPHINCS+ Anchor Layer for Kylos Arc QPADL",
    ),
    (
        "medium",
        b"Hash-based post-quantum signature. No algebraic structure. SHA-256 only. Stateless failsafe when both MQ and Lattice are broken.",
    ),
];

pub struct SphincsKeypair {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
}

pub struct SphincsSignature {
    pub message: Vec<u8>,
    pub message_label: String,
    pub signature: Vec<u8>,
}

pub struct SphincsTestVectorSet {
    pub keypair: SphincsKeypair,
    pub signatures: Vec<SphincsSignature>,
}

impl SphincsTestVectorSet {
    pub fn generate() -> Result<Self, sig::Error> {
        if !super::algo_is_enabled(super::SPHINCS_256F) {
            return Err(sig::UNKNOWN_ALG);
        }
        let sig = super::create_sig_by_name(super::SPHINCS_256F)?;
        let (pk, sk) = sig.keypair()?;

        let mut signatures = Vec::new();
        for (label, msg) in SPHINCS_MESSAGES {
            let signature = sig.sign(msg, &sk)?;
            signatures.push(SphincsSignature {
                message: msg.to_vec(),
                message_label: label.to_string(),
                signature,
            });
        }

        Ok(SphincsTestVectorSet {
            keypair: SphincsKeypair {
                public_key: pk,
                secret_key: sk,
            },
            signatures,
        })
    }

    pub fn verify_one(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::SPHINCS_256F)
            .map_err(|e| format!("Failed to create SPHINCS+: {e}"))?;

        for ts in &self.signatures {
            sig.verify(&ts.message, &ts.signature, &self.keypair.public_key)
                .map_err(|e| format!("SPHINCS+ verify '{}': {e}", ts.message_label))?;
        }
        Ok(())
    }

    pub fn verify_malformed_rejected(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::SPHINCS_256F)
            .map_err(|e| format!("Failed to create SPHINCS+: {e}"))?;

        for ts in &self.signatures {
            let mut bad_sig = ts.signature.clone();
            if bad_sig.is_empty() {
                continue;
            }
            bad_sig[0] ^= 0xFF;
            if sig.verify(&ts.message, &bad_sig, &self.keypair.public_key).is_ok() {
                return Err(format!("SPHINCS+ accepted malformed sig for '{}'", ts.message_label));
            }
        }
        Ok(())
    }

    pub fn large_msg_test(&self) -> Result<(), String> {
        let sig = super::create_sig_by_name(super::SPHINCS_256F)
            .map_err(|e| format!("Failed to create SPHINCS+: {e}"))?;
        let large_msg = vec![0xABu8; MAX_MESSAGE_SIZE];
        let large_sig = sig
            .sign(&large_msg, &self.keypair.secret_key)
            .map_err(|e| format!("SPHINCS+ large msg sign: {e}"))?;
        sig.verify(&large_msg, &large_sig, &self.keypair.public_key)
            .map_err(|e| format!("SPHINCS+ large msg verify: {e}"))
    }
}

pub fn generate_sphincs_vectors() -> Result<SphincsTestVectorSet, sig::Error> {
    SphincsTestVectorSet::generate()
}

pub fn verify_sphincs_vectors(set: &SphincsTestVectorSet) -> Result<(), String> {
    set.verify_one().map_err(|e| format!("SPHINCS+: {e}"))?;
    set.verify_malformed_rejected()
        .map_err(|e| format!("SPHINCS+: {e}"))?;
    set.large_msg_test()
        .map_err(|e| format!("SPHINCS+: {e}"))
}
