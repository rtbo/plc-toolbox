use std::mem;

use crate::connection;
use crate::msg::uacp;

include!(concat!(env!("OUT_DIR"), "/status_code.rs"));

impl StatusCode {
    pub fn is_good(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0
    }

    pub fn is_uncertain(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0x4000_0000
    }

    pub fn is_bad(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0x8000_0000
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, reason: {}", self.name(), self.explanation())
    }
}

impl std::error::Error for StatusCode {}

impl From<uacp::ErrorCode> for StatusCode {
    fn from(value: uacp::ErrorCode) -> Self {
        // SAFETY: All ErrorCode variants are valid status codes
        unsafe { mem::transmute(value as u32) }
    }
}

impl From<connection::Error> for StatusCode {
    fn from(value: connection::Error) -> Self {
        match value {
            connection::Error::Io(_) => StatusCode::BadCommunicationError,
            connection::Error::Uacp(err) => StatusCode::from(err.code),
            connection::Error::UnsupportedProtocol(_) => StatusCode::BadProtocolVersionUnsupported,
        }
    }
}
