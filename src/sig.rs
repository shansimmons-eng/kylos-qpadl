use std::ffi::CString;
use crate::ffi::{self, OQS_SIG, OQS_SUCCESS};

#[derive(Debug)]
pub struct Error;

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "oqs error")
    }
}

impl std::error::Error for Error {}

pub const UNKNOWN_ALG: Error = Error;

pub struct Sig {
    inner: *mut OQS_SIG,
}

unsafe impl Send for Sig {}
unsafe impl Sync for Sig {}

impl Drop for Sig {
    fn drop(&mut self) {
        unsafe { ffi::OQS_SIG_free(self.inner) }
    }
}

impl Sig {
    pub fn new(alg: &str) -> Result<Self, Error> {
        let cname = CString::new(alg).map_err(|_| Error)?;
        let ptr = unsafe { ffi::OQS_SIG_new(cname.as_ptr()) };
        if ptr.is_null() {
            return Err(Error);
        }
        Ok(Sig { inner: ptr })
    }

    pub fn is_enabled(alg: &str) -> bool {
        let cname = CString::new(alg).ok();
        match cname {
            Some(c) => unsafe { ffi::OQS_SIG_alg_is_enabled(c.as_ptr()) != 0 },
            None => false,
        }
    }

    pub fn keypair(&self) -> Result<(Vec<u8>, Vec<u8>), Error> {
        let pk_len = unsafe { (*self.inner).length_public_key };
        let sk_len = unsafe { (*self.inner).length_secret_key };
        let mut pk = vec![0u8; pk_len];
        let mut sk = vec![0u8; sk_len];
        let rc = unsafe {
            ffi::OQS_SIG_keypair(self.inner, pk.as_mut_ptr(), sk.as_mut_ptr())
        };
        if rc != OQS_SUCCESS {
            return Err(Error);
        }
        Ok((pk, sk))
    }

    pub fn sign(&self, msg: &[u8], sk: &[u8]) -> Result<Vec<u8>, Error> {
        let sig_len = unsafe { (*self.inner).length_signature };
        let mut sig = vec![0u8; sig_len];
        let mut actual_len = sig_len;
        let rc = unsafe {
            ffi::OQS_SIG_sign(
                self.inner,
                sig.as_mut_ptr(),
                &mut actual_len,
                msg.as_ptr(),
                msg.len(),
                sk.as_ptr(),
            )
        };
        if rc != OQS_SUCCESS {
            return Err(Error);
        }
        sig.truncate(actual_len);
        Ok(sig)
    }

    pub fn verify(&self, msg: &[u8], sig: &[u8], pk: &[u8]) -> Result<(), Error> {
        let rc = unsafe {
            ffi::OQS_SIG_verify(
                self.inner,
                msg.as_ptr(),
                msg.len(),
                sig.as_ptr(),
                sig.len(),
                pk.as_ptr(),
            )
        };
        if rc != OQS_SUCCESS {
            return Err(Error);
        }
        Ok(())
    }
}


