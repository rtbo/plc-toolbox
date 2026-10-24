//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::BufMut;
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::encoding::binary::{Decode, DecodingError, Encode, check_remaining};
use crate::ua;

pub trait MsgType {
    const MSG_TYPE: &'static [u8; 3];
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

impl MsgType for Hello {
    const MSG_TYPE: &'static [u8; 3] = b"HEL";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

impl MsgType for Acknowledge {
    const MSG_TYPE: &'static [u8; 3] = b"ACK";
}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Error {
    pub code: ua::StatusCode,
    pub reason: String,
}

impl MsgType for Error {
    const MSG_TYPE: &'static [u8; 3] = b"ERR";
}

impl Error {
    pub fn new(code: ua::StatusCode, reason: String) -> Self {
        Self { code, reason }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, reason: {}", self.code, self.reason)
    }
}

impl std::error::Error for Error {}

impl From<ua::StatusCode> for Error {
    fn from(code: ua::StatusCode) -> Self {
        Self {
            code,
            reason: code.explanation().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg<M: MsgType>(pub M);

impl<M: MsgType + Encode> Encode for Msg<M> {
    fn encoded_len(&self) -> u32 {
        8 + self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(M::MSG_TYPE);
        buf.put_u8(b'F');
        buf.put_u32_le(self.0.encoded_len() + 8);
        self.0.encode(buf);
    }
}

impl<M: MsgType + Decode> Decode for Msg<M> {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        let mut msg_type = [0u8; 3];
        check_remaining(buf, 8)?;
        buf.copy_to_slice(&mut msg_type);
        let _reserved = buf.get_u8();
        let message_size = buf.get_u32_le();
        check_remaining(buf, message_size as usize - 8)?;

        if &msg_type != M::MSG_TYPE {
            return Err(DecodingError::BadMessageType);
        }

        let inner = M::decode(buf)?;
        Ok(Msg(inner))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::{Acknowledge, Error, Hello, Msg};
    use crate::encoding::binary::{Decode, Encode};
    use crate::ua;

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
        let hello = Msg(Hello {
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
            ua::StatusCode::BadTcpMessageTooLarge,
            "message exceeds configured limit".to_owned(),
        );
        assert_roundtrip(&error);
    }
}
