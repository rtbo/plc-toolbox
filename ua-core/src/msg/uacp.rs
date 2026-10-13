//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::BufMut;
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::msg::binary::{Decode, DecodingError, Encode, check_remaining};
use crate::ua;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Msg {
    Hello(Hello),
    Acknowledge(Acknowledge),
    Error(Error),
}

impl Encode for Msg {
    fn encoded_len(&self) -> u32 {
        8 + match self {
            Msg::Hello(msg) => msg.encoded_len(),
            Msg::Acknowledge(msg) => msg.encoded_len(),
            Msg::Error(msg) => msg.encoded_len(),
        }
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            Msg::Hello(msg) => {
                buf.put_slice(b"HELF");
                buf.put_u32_le(8 + msg.encoded_len());
                msg.encode(buf);
            }
            Msg::Acknowledge(msg) => {
                buf.put_slice(b"ACKF");
                buf.put_u32_le(8 + msg.encoded_len());
                msg.encode(buf);
            }
            Msg::Error(msg) => {
                buf.put_slice(b"ERRF");
                buf.put_u32_le(8 + msg.encoded_len());
                msg.encode(buf);
            }
        }
    }
}

impl Decode for Msg {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        let mut msg_type = [0u8; 3];
        check_remaining(buf, 8)?;
        buf.copy_to_slice(&mut msg_type);
        let _reserved = buf.get_u8();
        let message_size = buf.get_u32_le();
        check_remaining(buf, message_size as usize - 8)?;

        match &msg_type {
            b"HEL" => {
                let hello = Hello::decode(buf)?;
                Ok(Msg::Hello(hello))
            }
            b"ACK" => {
                let ack = Acknowledge::decode(buf)?;
                Ok(Msg::Acknowledge(ack))
            }
            b"ERR" => {
                let err = Error::decode(buf)?;
                Ok(Msg::Error(err))
            }
            _ => Err(DecodingError::BadMessageType),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Hello {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
    pub endpoint_url: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EncodeBinary, DecodeBinary)]
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

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
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

impl From<ErrorCode> for Error {
    fn from(code: ErrorCode) -> Self {
        let status_code = ua::StatusCode::from(code);
        Self {
            code,
            reason: status_code.explanation().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::{Acknowledge, Error, ErrorCode, Hello, Msg};
    use crate::msg::binary::{Decode, DecodingError, Encode};

    #[derive(Clone, Copy, ua_macros::EncodeBinary, ua_macros::DecodeBinary)]
    #[repr(u8)]
    enum SmallCode {
        Value = 0xab,
    }

    #[derive(Clone, Copy, ua_macros::EncodeBinary, ua_macros::DecodeBinary)]
    #[repr(u16)]
    enum MediumCode {
        Value = 0xcdef,
    }

    fn assert_roundtrip<T>(value: &T)
    where
        T: Encode + Decode + PartialEq + std::fmt::Debug,
    {
        let mut encoded = BytesMut::new();
        value.encode(&mut encoded);
        assert_eq!(encoded.len(), value.encoded_len() as usize);

        let mut remaining = encoded.clone();
        let decoded = T::decode(&mut remaining).expect("encoded value should decode");
        assert!(remaining.is_empty());
        assert_eq!(&decoded, value);

        let mut reencoded = BytesMut::new();
        decoded.encode(&mut reencoded);
        assert_eq!(reencoded, encoded);
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
    fn hello_msg_encodes_fields_in_declaration_order() {
        let hello = Msg::Hello(Hello {
            protocol_version: 1,
            receive_buffer_size: 2,
            send_buffer_size: 3,
            max_message_size: 4,
            max_chunk_count: 5,
            endpoint_url: "x".to_owned(),
        });
        let mut buf = BytesMut::new();

        assert_eq!(hello.encoded_len(), 33);
        hello.encode(&mut buf);
        assert_eq!(
            &buf[..],
            &[
                b'H', b'E', b'L', b'F', 33, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0,
                0, 5, 0, 0, 0, 1, 0, 0, 0, b'x',
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
        assert!(matches!(SmallCode::decode(&mut buf), Ok(SmallCode::Value)));

        buf.clear();
        let medium = MediumCode::Value;
        assert_eq!(medium.encoded_len(), 2);
        medium.encode(&mut buf);
        assert_eq!(&buf[..], &[0xef, 0xcd]);
        assert!(matches!(
            MediumCode::decode(&mut buf),
            Ok(MediumCode::Value)
        ));

        buf.clear();
        let large = ErrorCode::BadTcpServerTooBusy;
        assert_eq!(large.encoded_len(), 4);
        large.encode(&mut buf);
        assert_eq!(&buf[..], &0x807d0000u32.to_le_bytes());
    }

    #[test]
    fn repr_enum_decodes_through_try_from() {
        let mut buf = BytesMut::from(&0x807d0000u32.to_le_bytes()[..]);

        assert!(matches!(
            ErrorCode::decode(&mut buf),
            Ok(ErrorCode::BadTcpServerTooBusy)
        ));

        let mut buf = BytesMut::from(&0x12345678u32.to_le_bytes()[..]);
        assert!(matches!(
            ErrorCode::decode(&mut buf),
            Err(DecodingError::InvalidData)
        ));
    }

    #[test]
    fn uacp_types_roundtrip() {
        let hello = Hello {
            protocol_version: 1,
            receive_buffer_size: 8192,
            send_buffer_size: 4096,
            max_message_size: 65536,
            max_chunk_count: 10,
            endpoint_url: "opc.tcp://localhost:4840".to_owned(),
        };
        assert_roundtrip(&hello);
        assert_roundtrip(&Msg::Hello(hello));

        let acknowledge = Acknowledge {
            protocol_version: 1,
            receive_buffer_size: 8192,
            send_buffer_size: 4096,
            max_message_size: 65536,
            max_chunk_count: 10,
        };
        assert_roundtrip(&acknowledge);
        assert_roundtrip(&Msg::Acknowledge(acknowledge));

        let error = Error::new(
            ErrorCode::BadTcpMessageTooLarge,
            "message exceeds configured limit".to_owned(),
        );
        assert_roundtrip(&error);
        assert_roundtrip(&Msg::Error(error));

        let error_codes = [
            ErrorCode::BadTcpServerTooBusy,
            ErrorCode::BadTcpMessageTypeInvalid,
            ErrorCode::BadTcpSecureChannelUnknown,
            ErrorCode::BadTcpMessageTooLarge,
            ErrorCode::BadTimeout,
            ErrorCode::BadTcpNotEnoughResources,
            ErrorCode::BadTcpInternalError,
            ErrorCode::BadTcpEndpointUrlInvalid,
            ErrorCode::BadSecurityChecksFailed,
            ErrorCode::BadRequestInterrupted,
            ErrorCode::BadRequestTimeout,
            ErrorCode::BadSecureChannelClosed,
            ErrorCode::BadSecureChannelTokenUnknown,
            ErrorCode::BadCertificateUntrusted,
            ErrorCode::BadCertificateRevocationUnknown,
            ErrorCode::BadCertificateIssuerRevocationUnknown,
            ErrorCode::BadCertificateRevoked,
            ErrorCode::BadCertificateTimeInvalid,
            ErrorCode::BadCertificateIssuerTimeInvalid,
            ErrorCode::BadCertificateUseNotAllowed,
            ErrorCode::BadCertificateIssuerUseNotAllowed,
            ErrorCode::BadCertificateIssuerRevoked,
            ErrorCode::BadSequenceNumberInvalid,
            ErrorCode::BadServiceUnsupported,
        ];
        for code in error_codes {
            assert_roundtrip(&code);
        }
    }
}
