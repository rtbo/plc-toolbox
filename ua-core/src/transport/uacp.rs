//! This module contains the implementation of UA Connection Protocol (UACP)
use bytes::{BufMut, BytesMut};
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::encoding::binary::{self, Decode, DecodingError, Encode, check_remaining};
use crate::ua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Hello,
    Acknowledge,
    Error,
}

pub trait TypedMsg {
    const MSG_TYPE: MsgType;
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
        check_remaining(buf, 8)?;
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
        Ok(Header {
            msg_type,
            length,
        })
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

impl TypedMsg for Hello {
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

impl TypedMsg for Acknowledge {
    const MSG_TYPE: MsgType = MsgType::Acknowledge;
}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct Error {
    pub code: ua::StatusCode,
    pub reason: String,
}

impl TypedMsg for Error {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg<M: TypedMsg>(pub M);

impl<M: TypedMsg + Encode> Encode for Msg<M> {
    fn encoded_len(&self) -> u32 {
        8 + self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let header = Header {
            msg_type: M::MSG_TYPE,
            length: self.encoded_len(),
        };
        header.encode(buf);
        self.0.encode(buf);
    }
}

impl<M: TypedMsg + Decode> Decode for Msg<M> {
    fn decode<B: bytes::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, DecodingError> {
        let header = Header::decode(buf, ctx)?;
        check_remaining(buf, header.length as usize - 8)?;

        if header.msg_type != M::MSG_TYPE {
            return Err(DecodingError::BadMessageType);
        }

        let inner = M::decode(buf, ctx)?;
        Ok(Msg(inner))
    }
}

/// Any UACP message, decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyMsg {
    Hello(Hello),
    Acknowledge(Acknowledge),
    Error(Error),
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
    match chunk.get(..4)? {
        b"HELF" => Some(MsgType::Hello),
        b"ACKF" => Some(MsgType::Acknowledge),
        b"ERRF" => Some(MsgType::Error),
        _ => None,
    }
}

/// UACP layer: encodes/decodes UACP messages on top of the raw chunk transport.
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

    pub fn transport_mut(&mut self) -> &mut super::tcp::Transport {
        &mut self.transport
    }

    pub fn into_transport(self) -> super::tcp::Transport {
        self.transport
    }

    /// Encodes a UACP message directly into the send buffer (not flushed).
    pub async fn send<M>(&mut self, msg: Msg<M>) -> Result<(), super::Error>
    where
        M: TypedMsg + Encode,
    {
        self.transport.send_msg_chunk(|buf| msg.encode(buf)).await
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
        let Some(msg_type) = msg_type(&chunk) else {
            return Ok(Incoming::Chunk(chunk));
        };
        let ctx = &self.decoding_ctx;
        let msg = match msg_type {
            MsgType::Hello => Msg::<Hello>::decode(&mut chunk, ctx)?.0.into(),
            MsgType::Acknowledge => Msg::<Acknowledge>::decode(&mut chunk, ctx)?.0.into(),
            MsgType::Error => Msg::<Error>::decode(&mut chunk, ctx)?.0.into(),
        };
        Ok(Incoming::Uacp(msg))
    }

    /// Receives a UACP message of a specific type (e.g. during the handshake).
    /// Any other chunk is a `BadMessageType` decoding error.
    pub async fn recv_typed<M>(&mut self) -> Result<Msg<M>, super::Error>
    where
        M: TypedMsg + Decode,
    {
        let mut chunk = self.transport.recv_msg_chunk().await?;
        Ok(Msg::decode(&mut chunk, &self.decoding_ctx)?)
    }
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
