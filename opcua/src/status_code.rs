use std::fmt;

use crate::ffi;

include!(concat!(env!("OUT_DIR"), "/status_code.rs"));

pub type Result<T> = std::result::Result<T, StatusCode>;

pub(crate) fn check(code: ffi::UA_StatusCode) -> Result<()> {
    if code == ffi::UA_STATUSCODE_GOOD {
        Ok(())
    } else {
        Err(StatusCode::from(code))
    }
}

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
