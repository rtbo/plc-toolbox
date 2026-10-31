//! This module contains the implementation of UA Connection Protocol (UACP)
use binary::{Decode, Encode};
use bytes::{BufMut, BytesMut};
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::encoding::binary::{self, DecodingError};
use crate::ua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Hello,
    Acknowledge,
    Error,
}

pub trait Msg: binary::Encode + std::fmt::Debug {
    fn msg_type(&self) -> MsgType;
    fn msg_encoded_len(&self) -> u32 {
        8 + self.encoded_len()
    }
    fn msg_encode<B: BufMut>(&self, buf: &mut B) {
        let header = Header {
            msg_type: self.msg_type(),
            length: self.msg_encoded_len(),
        };
        header.encode(buf);
        self.encode(buf);
    }

    fn msg_decode<B: bytes::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError>
    where
        Self: Sized;
}

pub trait StaticMsg: Msg + binary::Decode {
    const MSG_TYPE: MsgType;
}

impl<T: StaticMsg> Msg for T {
    fn msg_type(&self) -> MsgType {
        Self::MSG_TYPE
    }

    fn msg_decode<B: bytes::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let header = Header::decode(buf, ctx)?;
        if header.msg_type != Self::MSG_TYPE {
            return Err(binary::DecodingError::InvalidData);
        }
        let msg = T::decode(buf, ctx)?;
        Ok(msg)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub msg_type: MsgType,
    pub length: u32,
}

impl binary::Encode for Header {
    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        match self.msg_type {
            MsgType::Hello => buf.put_slice(b"HELF"),
            MsgType::Acknowledge => buf.put_slice(b"ACKF"),
            MsgType::Error => buf.put_slice(b"ERRF"),
        }
        buf.put_u32_le(self.length);
    }
}

impl binary::Decode for Header {
    fn decode<B: bytes::Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let mut typ: [u8; 3] = [0u8; 3];
        buf.copy_to_slice(&mut typ);
        let _reserved = buf.get_u8();
        let length = buf.get_u32_le();

        let msg_type = match &typ {
            b"HEL" => MsgType::Hello,
            b"ACK" => MsgType::Acknowledge,
            b"ERR" => MsgType::Error,
            _ => return Err(binary::DecodingError::InvalidData),
        };
        Ok(Header { msg_type, length })
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

impl StaticMsg for Hello {
    const MSG_TYPE: MsgType = MsgType::Hello;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Acknowledge {
    pub protocol_version: u32,
    pub receive_buffer_size: u32,
    pub send_buffer_size: u32,
    pub max_message_size: u32,
    pub max_chunk_count: u32,
}

impl StaticMsg for Acknowledge {
    const MSG_TYPE: MsgType = MsgType::Acknowledge;
}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Error {
    pub code: ua::StatusCode,
    pub reason: String,
}

impl StaticMsg for Error {
    const MSG_TYPE: MsgType = MsgType::Error;
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

/// Any UACP message, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyMsg {
    Hello(Hello),
    Acknowledge(Acknowledge),
    Error(Error),
}

impl From<Hello> for AnyMsg {
    fn from(m: Hello) -> Self {
        AnyMsg::Hello(m)
    }
}
impl From<Acknowledge> for AnyMsg {
    fn from(m: Acknowledge) -> Self {
        AnyMsg::Acknowledge(m)
    }
}
impl From<Error> for AnyMsg {
    fn from(m: Error) -> Self {
        AnyMsg::Error(m)
    }
}

impl binary::Encode for AnyMsg {
    fn encoded_len(&self) -> u32 {
        match self {
            AnyMsg::Hello(m) => m.encoded_len(),
            AnyMsg::Acknowledge(m) => m.encoded_len(),
            AnyMsg::Error(m) => m.encoded_len(),
        }
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        match self {
            AnyMsg::Hello(m) => m.encode(buf),
            AnyMsg::Acknowledge(m) => m.encode(buf),
            AnyMsg::Error(m) => m.encode(buf),
        }
    }
}

impl Msg for AnyMsg {
    fn msg_type(&self) -> MsgType {
        match self {
            AnyMsg::Hello(_) => MsgType::Hello,
            AnyMsg::Acknowledge(_) => MsgType::Acknowledge,
            AnyMsg::Error(_) => MsgType::Error,
        }
    }

    fn msg_decode<B: bytes::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError>
    where
        Self: Sized,
    {
        let header = Header::decode(buf, ctx)?;
        match header.msg_type {
            MsgType::Hello => {
                let m = Hello::decode(buf, ctx)?;
                Ok(AnyMsg::Hello(m))
            }
            MsgType::Acknowledge => {
                let m = Acknowledge::decode(buf, ctx)?;
                Ok(AnyMsg::Acknowledge(m))
            }
            MsgType::Error => {
                let m = Error::decode(buf, ctx)?;
                Ok(AnyMsg::Error(m))
            }
        }
    }
}

/// What the UACP layer hands to the layer above.
#[derive(Debug)]
pub enum Incoming {
    /// A decoded UACP message.
    Uacp(AnyMsg),
    /// A chunk that is not UACP (e.g. a chunk of secure conversation message), passed through as-is.
    Chunk(bytes::BytesMut),
}

/// Returns the UACP message type of a chunk, or `None` if it is not a UACP chunk.
pub fn msg_type(chunk: &[u8]) -> Option<MsgType> {
    match chunk.get(..3)? {
        b"HEL" => Some(MsgType::Hello),
        b"ACK" => Some(MsgType::Acknowledge),
        b"ERR" => Some(MsgType::Error),
        _ => None,
    }
}

/// UACP layer: encodes/decodes UACP messages on top of the raw chunk transport.
#[derive(Debug)]
pub struct Uacp {
    transport: super::tcp::Transport,
    decoding_ctx: binary::DecodingCtx,
}

impl Uacp {
    pub fn new(transport: super::tcp::Transport) -> Self {
        Self {
            transport,
            decoding_ctx: Default::default(),
        }
    }

    pub fn max_send_size(&self) -> u32 {
        self.transport.max_send_size() as _
    }

    pub fn max_recv_size(&self) -> u32 {
        self.transport.max_recv_size() as _
    }

    pub fn transport_mut(&mut self) -> &mut super::tcp::Transport {
        &mut self.transport
    }

    pub fn into_transport(self) -> super::tcp::Transport {
        self.transport
    }

    /// Encodes a UACP message directly into the send buffer (not flushed).
    pub async fn send<M>(&mut self, msg: M) -> Result<(), super::Error>
    where
        M: Msg,
    {
        self.transport
            .send_msg_chunk(|buf| msg.msg_encode(buf))
            .await
    }

    /// Buffers a chunk from an upper layer (e.g. secure channel) as-is (not flushed).
    pub async fn send_chunk<F>(&mut self, write: F) -> Result<(), super::Error>
    where
        F: FnOnce(&mut BytesMut),
    {
        self.transport.send_msg_chunk(write).await
    }

    pub async fn flush(&mut self) -> Result<(), super::Error> {
        self.transport.flush().await
    }

    /// Receives the next chunk: UACP messages are decoded, others are passed through.
    pub async fn recv(&mut self) -> Result<Incoming, super::Error> {
        let mut chunk = self.transport.recv_msg_chunk().await?;
        let ctx = &self.decoding_ctx;
        let header = match Header::decode(&mut chunk, ctx) {
            Ok(header) => header,
            Err(DecodingError::BadMessageType) => {
                // Not a UACP chunk, pass through
                return Ok(Incoming::Chunk(chunk));
            }
            Err(err) => {
                return Err(super::Error::Decoding(err));
            }
        };
        let msg = match header.msg_type {
            MsgType::Hello => {
                let m = Hello::decode(&mut chunk, ctx)?;
                AnyMsg::Hello(m)
            }
            MsgType::Acknowledge => {
                let m = Acknowledge::decode(&mut chunk, ctx)?;
                AnyMsg::Acknowledge(m)
            }
            MsgType::Error => {
                let m = Error::decode(&mut chunk, ctx)?;
                AnyMsg::Error(m)
            }
        };
        Ok(Incoming::Uacp(msg))
    }

    /// Receives a UACP message of a specific type (e.g. during the handshake).
    /// Any other chunk is a `BadMessageType`
    pub async fn recv_typed<M>(&mut self) -> Result<M, super::Error>
    where
        M: Msg,
    {
        let mut chunk = self.transport.recv_msg_chunk().await?;
        Ok(M::msg_decode(&mut chunk, &self.decoding_ctx)?)
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
        let decoded =
            T::decode(&mut remaining, &Default::default()).expect("encoded value should decode");
        assert!(remaining.is_empty());
        assert_eq!(&decoded, value);

        let mut reencoded = BytesMut::new();
        decoded.encode(&mut reencoded);
        assert_eq!(reencoded, encoded);
    }

    fn assert_roundtrip_msg<T>(value: &T)
    where
        T: Msg + PartialEq + std::fmt::Debug,
    {
        let mut encoded = BytesMut::new();
        value.msg_encode(&mut encoded);
        assert_eq!(encoded.len(), value.msg_encoded_len() as usize);

        let mut remaining = encoded.clone();
        let decoded = T::msg_decode(&mut remaining, &Default::default())
            .expect("encoded value should decode");
        assert!(remaining.is_empty());
        assert_eq!(&decoded, value);

        let mut reencoded = BytesMut::new();
        decoded.msg_encode(&mut reencoded);
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
        let hello = Hello {
            protocol_version: 1,
            receive_buffer_size: 2,
            send_buffer_size: 3,
            max_message_size: 4,
            max_chunk_count: 5,
            endpoint_url: "x".to_owned(),
        };
        let mut buf = BytesMut::new();

        assert_eq!(hello.msg_encoded_len(), 33);
        hello.msg_encode(&mut buf);
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
        assert_roundtrip_msg(&hello);

        let acknowledge = Acknowledge {
            protocol_version: 1,
            receive_buffer_size: 8192,
            send_buffer_size: 4096,
            max_message_size: 65536,
            max_chunk_count: 10,
        };
        assert_roundtrip(&acknowledge);
        assert_roundtrip_msg(&acknowledge);

        let error = Error::new(
            ua::StatusCode::BadTcpMessageTooLarge,
            "message exceeds configured limit".to_owned(),
        );
        assert_roundtrip(&error);
        assert_roundtrip_msg(&error);
    }
}
