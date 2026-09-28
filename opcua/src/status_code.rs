use std::fmt;

use crate::ffi;

include!(concat!(env!("OUT_DIR"), "/status_code.rs"));

pub type Result<T> = std::result::Result<T, StatusCode>;


impl StatusCode {
    pub fn check(&self) -> Result<()> {
        match self {
            StatusCode::Good => Ok(()),
            _ => Err(*self),
        }
    }

    pub fn expect_good(&self, msg: &str) {
        self.check().expect(msg);
    }

    /// Converts a raw C `UA_StatusCode` into a Rust `StatusCode` without checking its validity.
    ///
    /// # Safety
    /// 
    /// The caller must ensure that the provided `code` is a valid `UA_StatusCode`.
    /// Typically, a status returned by the underlying C library can be safely converted using this function.
    pub unsafe fn from_raw_unchecked(code: ffi::UA_StatusCode) -> Self {
        unsafe {
            std::mem::transmute(code)
        }
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub(crate) fn check(code: ffi::UA_StatusCode) -> Result<()> {
    if code == ffi::UA_STATUSCODE_GOOD {
        Ok(())
    } else {
        Err(StatusCode::from(code))
    }
}

pub(crate) fn expect_good(code: ffi::UA_StatusCode) {
    if code != ffi::UA_STATUSCODE_GOOD {
        panic!("Expected good status code, got {:?}", StatusCode::from(code));
    }
}