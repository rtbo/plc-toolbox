//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::BufMut;

use crate::msg::binary::EncodeBinary;

#[derive(Debug, Clone, Copy)]
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

impl EncodeBinary for Header {
    fn encoded_len(&self) -> u32 {
        Self::encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.message_type.encode(buf);
        self.reserved.encode(buf);
        self.message_size.encode(buf);
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

#[derive(Debug, Clone)]
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

impl EncodeBinary for Hello {
    fn encoded_len(&self) -> u32 {
        self.protocol_version.encoded_len()
            + self.receive_buffer_size.encoded_len()
            + self.send_buffer_size.encoded_len()
            + self.max_message_size.encoded_len()
            + self.max_chunk_count.encoded_len()
            + self.endpoint_url.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.protocol_version.encode(buf);
        self.receive_buffer_size.encode(buf);
        self.send_buffer_size.encode(buf);
        self.max_message_size.encode(buf);
        self.max_chunk_count.encode(buf);
        self.endpoint_url.encode(buf);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
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
