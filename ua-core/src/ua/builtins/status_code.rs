use std::mem;

use crate::msg::binary::{self, DecodingError};
use crate::msg::uacp;
use crate::{DataType, connection};

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
            connection::Error::Decoding(DecodingError::BadMessageType) => {
                StatusCode::BadTcpMessageTypeInvalid
            }
            connection::Error::Decoding(DecodingError::NotEnoughData) => {
                StatusCode::BadEncodingLimitsExceeded
            }
            connection::Error::Decoding(_) => StatusCode::BadDataEncodingInvalid,
            connection::Error::UnsupportedProtocol(_) => StatusCode::BadProtocolVersionUnsupported,
            connection::Error::MessageSize { .. } => StatusCode::BadEncodingLimitsExceeded,
        }
    }
}

impl DataType for StatusCode {
    fn type_id() -> crate::ua::NodeId {
        crate::ua::NodeId::Numeric(0, 19)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        StatusCode::Good
    }
}

impl binary::Encode for StatusCode {
    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(*self as u32)
    }
}

impl binary::Decode for StatusCode {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let code = buf.get_u32_le();
        Self::try_from(code).map_err(|_| binary::DecodingError::InvalidData)
    }
}
