use core::fmt;

use crate::StatusCode;

pub enum Error {
     NotConnected,
     ConnectionRejected,
     Other(StatusCode),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotConnected => write!(f, "Not connected"),
            Error::ConnectionRejected => write!(f, "Connection rejected"),
            Error::Other(code) => {
                write!(f, "{}", code.name())
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotConnected => write!(f, "Not connected"),
            Error::ConnectionRejected => write!(f, "Connection rejected"),
            Error::Other(code) => write!(f, "Other error: {:?}", code),
        }
    }
}

impl core::error::Error for Error {}

impl From<StatusCode> for Error {
    fn from(code: StatusCode) -> Self {
        match code {
            StatusCode::BadNotConnected => Error::NotConnected,
            StatusCode::BadConnectionRejected => Error::ConnectionRejected,
            _ => Error::Other(code),
        }
    }
}
