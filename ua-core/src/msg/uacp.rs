//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::BufMut;

use crate::msg::binary::EncodeBinary;

#[derive(Debug, Clone, Copy, ua_macros::EncodeBinary)]
pub struct Header {
    pub message_type: [u8; 3],
    pub reserved: u8,
    pub message_size: u32,
}

impl Header {
    pub fn encoded_len() -> u32 {
        3 + 1 + 4
    }
}

pub trait UacpMessage: EncodeBinary {
    fn message_type() -> [u8; 3];
}

pub struct Msg<T: UacpMessage>(pub T);

impl<T: UacpMessage> EncodeBinary for Msg<T> {
    fn encoded_len(&self) -> u32 {
        Header::encoded_len() + self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let header = Header {
            message_type: T::message_type(),
            reserved: b'F',
            message_size: self.encoded_len(),
        };
        header.encode(buf);
        self.0.encode(buf);
    }
}

#[derive(Debug, Clone, ua_macros::EncodeBinary)]
pub struct Hello {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
    pub endpoint_url: String,
}

impl UacpMessage for Hello {
    fn message_type() -> [u8; 3] {
        *b"HEL"
    }
}

#[derive(Debug, Clone, Copy, ua_macros::EncodeBinary)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ua_macros::EncodeBinary)]
#[repr(u32)]
pub enum ErrorCode {
    /// The server cannot process the request because it is too busy.
    BadTcpServerTooBusy = 0x807d0000,
    /// The type of the message specified in the header invalid.
    BadTcpMessageTypeInvalid = 0x807e0000,
    /// The SecureChannelId and/or TokenId are not currently in use.
    BadTcpSecureChannelUnknown = 0x807f0000,
    /// The size of the message chunk specified in the header is too large.
    BadTcpMessageTooLarge = 0x80800000,
    /// The operation timed out.
    BadTimeout = 0x800a0000,
    /// There are not enough resources to process the request.
    BadTcpNotEnoughResources = 0x80810000,
    /// An internal error occurred.
    BadTcpInternalError = 0x80820000,
    /// The server does not recognize the QueryString specified.
    BadTcpEndpointUrlInvalid = 0x80830000,
    /// An error occurred verifying security.
    BadSecurityChecksFailed = 0x80130000,
    /// The request could not be sent because of a network interruption.
    BadRequestInterrupted = 0x80840000,
    /// Timeout occurred while processing the request.
    BadRequestTimeout = 0x80850000,
    /// The secure channel has been closed.
    BadSecureChannelClosed = 0x80860000,
    /// The token has expired or is not recognized.
    BadSecureChannelTokenUnknown = 0x80870000,
    /// The certificate is not trusted.
    BadCertificateUntrusted = 0x801a0000,
    /// It was not possible to determine if the certificate has been revoked.
    BadCertificateRevocationUnknown = 0x801b0000,
    /// It was not possible to determine if the issuer certificate has been revoked.
    BadCertificateIssuerRevocationUnknown = 0x801c0000,
    /// The certificate has been revoked.
    BadCertificateRevoked = 0x801d0000,
    /// The certificate has expired or is not yet valid.
    BadCertificateTimeInvalid = 0x80140000,
    /// An issuer certificate has expired or is not yet valid.
    BadCertificateIssuerTimeInvalid = 0x80150000,
    /// The certificate may not be used for the requested operation.
    BadCertificateUseNotAllowed = 0x80180000,
    /// The issuer certificate may not be used for the requested operation.
    BadCertificateIssuerUseNotAllowed = 0x80190000,
    /// The issuer certificate has been revoked.
    BadCertificateIssuerRevoked = 0x801e0000,
    /// The sequence number is not valid.
    BadSequenceNumberInvalid = 0x80880000,
    /// The server does not support the requested service.
    BadServiceUnsupported = 0x800b0000,
}

impl ErrorCode {
    pub fn from_u32(code: u32) -> Option<Self> {
        match code {
            0x807d0000 => Some(ErrorCode::BadTcpServerTooBusy),
            0x807e0000 => Some(ErrorCode::BadTcpMessageTypeInvalid),
            0x807f0000 => Some(ErrorCode::BadTcpSecureChannelUnknown),
            0x80800000 => Some(ErrorCode::BadTcpMessageTooLarge),
            0x800a0000 => Some(ErrorCode::BadTimeout),
            0x80810000 => Some(ErrorCode::BadTcpNotEnoughResources),
            0x80820000 => Some(ErrorCode::BadTcpInternalError),
            0x80830000 => Some(ErrorCode::BadTcpEndpointUrlInvalid),
            0x80130000 => Some(ErrorCode::BadSecurityChecksFailed),
            0x80840000 => Some(ErrorCode::BadRequestInterrupted),
            0x80850000 => Some(ErrorCode::BadRequestTimeout),
            0x80860000 => Some(ErrorCode::BadSecureChannelClosed),
            0x80870000 => Some(ErrorCode::BadSecureChannelTokenUnknown),
            0x801a0000 => Some(ErrorCode::BadCertificateUntrusted),
            0x801b0000 => Some(ErrorCode::BadCertificateRevocationUnknown),
            0x801c0000 => Some(ErrorCode::BadCertificateIssuerRevocationUnknown),
            0x801d0000 => Some(ErrorCode::BadCertificateRevoked),
            0x80140000 => Some(ErrorCode::BadCertificateTimeInvalid),
            0x80150000 => Some(ErrorCode::BadCertificateIssuerTimeInvalid),
            0x80180000 => Some(ErrorCode::BadCertificateUseNotAllowed),
            0x80190000 => Some(ErrorCode::BadCertificateIssuerUseNotAllowed),
            0x801e0000 => Some(ErrorCode::BadCertificateIssuerRevoked),
            0x80880000 => Some(ErrorCode::BadSequenceNumberInvalid),
            0x800b0000 => Some(ErrorCode::BadServiceUnsupported),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, ua_macros::EncodeBinary)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: String,
}

impl Error {
    pub fn new(code: ErrorCode, reason: String) -> Self {
        Self { code, reason }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, reason: {}", self.code, self.reason)
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::{ErrorCode, Hello};
    use crate::msg::binary::EncodeBinary;

    #[derive(Clone, Copy, ua_macros::EncodeBinary)]
    #[repr(u8)]
    enum SmallCode {
        Value = 0xab,
    }

    #[derive(Clone, Copy, ua_macros::EncodeBinary)]
    #[repr(u16)]
    enum MediumCode {
        Value = 0xcdef,
    }

    #[test]
    fn hello_encodes_fields_in_declaration_order() {
        let hello = Hello {
            protocol_version: 1,
            receive_buffer_size: 2,
            send_buffer_size: 3,
            max_message_size: 4,
            max_chunk_count: 5,
            endpoint_url: "x".to_owned(),
        };
        let mut buf = BytesMut::new();

        assert_eq!(hello.encoded_len(), 25);
        hello.encode(&mut buf);
        assert_eq!(
            &buf[..],
            &[
                1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, b'x',
            ]
        );
    }

    #[test]
    fn repr_enums_encode_their_discriminants() {
        let mut buf = BytesMut::new();

        let small = SmallCode::Value;
        assert_eq!(small.encoded_len(), 1);
        small.encode(&mut buf);
        assert_eq!(&buf[..], &[0xab]);

        buf.clear();
        let medium = MediumCode::Value;
        assert_eq!(medium.encoded_len(), 2);
        medium.encode(&mut buf);
        assert_eq!(&buf[..], &[0xef, 0xcd]);

        buf.clear();
        let large = ErrorCode::BadTcpServerTooBusy;
        assert_eq!(large.encoded_len(), 4);
        large.encode(&mut buf);
        assert_eq!(&buf[..], &0x807d0000u32.to_le_bytes());
    }
}
