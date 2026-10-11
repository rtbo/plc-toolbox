//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::BufMut;
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::msg::binary::{DecodeBinary, DecodingError, EncodeBinary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Hello,
    Acknowledge,
    Error,
}

impl MsgType {
    pub fn to_bytes(&self) -> [u8; 3] {
        match self {
            MsgType::Hello => *b"HEL",
            MsgType::Acknowledge => *b"ACK",
            MsgType::Error => *b"ERR",
        }
    }

    pub fn from_bytes(bytes: &[u8; 3]) -> Result<Self, DecodingError> {
        match bytes {
            b"HEL" => Ok(MsgType::Hello),
            b"ACK" => Ok(MsgType::Acknowledge),
            b"ERR" => Ok(MsgType::Error),
            _ => Err(DecodingError::InvalidData),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub message_type: MsgType,
    pub message_size: u32,
}

impl EncodeBinary for Header {
    fn encoded_len(&self) -> u32 {
        3 + 1 + 4
    }
     fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(&self.message_type.to_bytes());
        buf.put_u8(b'F');
        buf.put_u32_le(self.message_size);
     }
}

impl DecodeBinary for Header {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        let mut message_type_bytes = [0u8; 3];
        buf.copy_to_slice(&mut message_type_bytes);
        let message_type = MsgType::from_bytes(&message_type_bytes)?;
        let _reserved = buf.get_u8();
        let message_size = buf.get_u32_le();
        Ok(Header {
            message_type,
            message_size,
        })
    }    
}

impl Header {
    pub fn encoded_len() -> u32 {
        3 + 1 + 4
    }
}

pub trait UacpMessage: EncodeBinary + DecodeBinary {
    fn message_type() -> MsgType;
}

pub struct Msg<T: UacpMessage>(pub T);

impl<T: UacpMessage> EncodeBinary for Msg<T> {
    fn encoded_len(&self) -> u32 {
        Header::encoded_len() + self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let header = Header {
            message_type: T::message_type(),
            message_size: self.encoded_len(),
        };
        header.encode(buf);
        self.0.encode(buf);
    }
}

impl<T: UacpMessage> DecodeBinary for Msg<T> {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        let header = Header::decode(buf)?;
        if header.message_type != T::message_type() {
            return Err(crate::msg::binary::DecodingError::BadMessageType);
        }
        let message = T::decode(buf)?;
        Ok(Msg(message))
    }
}

impl<T: UacpMessage> std::fmt::Debug for Msg<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Msg").field(&self.0).finish()
    }
}

impl<T: UacpMessage> PartialEq for Msg<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: UacpMessage> Eq for Msg<T> where T: Eq {}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Hello {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
    pub endpoint_url: String,
}

impl UacpMessage for Hello {
    fn message_type() -> MsgType {
        MsgType::Hello
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

impl UacpMessage for Acknowledge {
    fn message_type() -> MsgType {
        MsgType::Acknowledge
    }
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

#[derive(Debug, Clone, PartialEq, EncodeBinary, DecodeBinary)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: String,
}

impl UacpMessage for Error {
    fn message_type() -> MsgType {
        MsgType::Error
    }    
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
        let status_code = crate::StatusCode::from(code);
        Self {
            code,
            reason: status_code.explanation().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::{Acknowledge, Error, ErrorCode, Header, Hello, Msg, MsgType};
    use crate::msg::binary::{DecodeBinary, DecodingError, EncodeBinary};

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
        T: EncodeBinary + DecodeBinary + PartialEq + std::fmt::Debug,
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
        let header = Header {
            message_type: MsgType::Hello,
            message_size: 8,
        };
        assert_roundtrip(&header);

        let hello = Hello {
            protocol_version: 1,
            receive_buffer_size: 8192,
            send_buffer_size: 4096,
            max_message_size: 65536,
            max_chunk_count: 10,
            endpoint_url: "opc.tcp://localhost:4840".to_owned(),
        };
        assert_roundtrip(&hello);
        assert_roundtrip(&Msg(hello));

        let acknowledge = Acknowledge {
            protocol_version: 1,
            receive_buffer_size: 8192,
            send_buffer_size: 4096,
            max_message_size: 65536,
            max_chunk_count: 10,
        };
        assert_roundtrip(&acknowledge);

        let error = Error::new(
            ErrorCode::BadTcpMessageTooLarge,
            "message exceeds configured limit".to_owned(),
        );
        assert_roundtrip(&error);

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
