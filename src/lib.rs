pub mod bench;
pub mod constant_time;
pub mod ffi;
pub mod sig;
pub mod test_vectors;

static INIT: std::sync::Once = std::sync::Once::new();

pub fn init() {
    INIT.call_once(|| {
        unsafe { ffi::OQS_init(); }
    });
}

// MAYO (Speed Layer: MQ, NP-Hard)
const MAYO1: &str = "MAYO-1";
const MAYO3: &str = "MAYO-3";
const MAYO5: &str = "MAYO-5";

// Falcon-512 temporary patch (deployed while MAYO matures)
pub const FALCON_512: &str = "Falcon-512";
pub const FALCON_PADDED_512: &str = "Falcon-padded-512";
pub const FALCON_1024: &str = "Falcon-1024";

// ML-DSA-65 (Primary Layer: Lattice, Module-LWE, FIPS 204)
pub const ML_DSA_65: &str = "ML-DSA-65";

// SPHINCS+ (Anchor Layer: Hash-Based, FIPS 205 / SLH-DSA)
pub const SPHINCS_256F: &str = "SPHINCS+-SHA2-256f-simple";

pub fn algo_for_level(level: u8) -> Option<&'static str> {
    match level {
        1 => Some(MAYO1),
        3 => Some(MAYO3),
        5 => Some(MAYO5),
        _ => None,
    }
}

pub fn supported_levels() -> Vec<u8> {
    [1u8, 3, 5]
        .into_iter()
        .filter(|l| algo_for_level(*l).is_some_and(|a| sig::Sig::is_enabled(a)))
        .collect()
}

pub fn create_sig(level: u8) -> Result<sig::Sig, sig::Error> {
    let algo = algo_for_level(level).ok_or(sig::UNKNOWN_ALG)?;
    if !sig::Sig::is_enabled(algo) {
        return Err(sig::UNKNOWN_ALG);
    }
    sig::Sig::new(algo)
}

pub fn algo_is_enabled(name: &str) -> bool {
    sig::Sig::is_enabled(name)
}

pub fn create_sig_by_name(name: &str) -> Result<sig::Sig, sig::Error> {
    if !sig::Sig::is_enabled(name) {
        return Err(sig::UNKNOWN_ALG);
    }
    sig::Sig::new(name)
}
