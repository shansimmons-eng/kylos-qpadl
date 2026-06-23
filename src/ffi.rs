use std::ffi::c_char;
use std::os::raw::c_int;

#[allow(non_camel_case_types)]
pub type OQS_STATUS = c_int;
pub const OQS_SUCCESS: OQS_STATUS = 0;
pub const OQS_ERROR: OQS_STATUS = -1;

type KeypairFn = unsafe extern "C" fn(*mut u8, *mut u8) -> OQS_STATUS;
type SignFn = unsafe extern "C" fn(
    *mut u8,
    *mut usize,
    *const u8,
    usize,
    *const u8,
) -> OQS_STATUS;
type SignCtxFn = unsafe extern "C" fn(
    *mut u8,
    *mut usize,
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
) -> OQS_STATUS;
type VerifyFn = unsafe extern "C" fn(
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
) -> OQS_STATUS;
type VerifyCtxFn = unsafe extern "C" fn(
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
    usize,
    *const u8,
) -> OQS_STATUS;

#[repr(C)]
pub struct OQS_SIG {
    pub method_name: *const c_char,
    pub alg_version: *const c_char,
    pub claimed_nist_level: u8,
    pub euf_cma: u8,
    pub suf_cma: u8,
    pub sig_with_ctx_support: u8,
    _pad: [u8; 4],
    pub length_public_key: usize,
    pub length_secret_key: usize,
    pub length_signature: usize,
    keypair: Option<KeypairFn>,
    sign: Option<SignFn>,
    sign_with_ctx_str: Option<SignCtxFn>,
    verify: Option<VerifyFn>,
    verify_with_ctx_str: Option<VerifyCtxFn>,
}

unsafe extern "C" {
    pub fn OQS_SIG_new(method_name: *const c_char) -> *mut OQS_SIG;
    pub fn OQS_SIG_free(sig: *mut OQS_SIG);
    pub fn OQS_SIG_keypair(
        sig: *const OQS_SIG,
        public_key: *mut u8,
        secret_key: *mut u8,
    ) -> OQS_STATUS;
    pub fn OQS_SIG_sign(
        sig: *const OQS_SIG,
        signature: *mut u8,
        signature_len: *mut usize,
        message: *const u8,
        message_len: usize,
        secret_key: *const u8,
    ) -> OQS_STATUS;
    pub fn OQS_SIG_verify(
        sig: *const OQS_SIG,
        message: *const u8,
        message_len: usize,
        signature: *const u8,
        signature_len: usize,
        public_key: *const u8,
    ) -> OQS_STATUS;
    pub fn OQS_SIG_alg_is_enabled(method_name: *const c_char) -> c_int;
    pub fn OQS_init();
}
