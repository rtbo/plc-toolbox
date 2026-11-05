//! OPC/UA Secure Conversation (UASC) implementation

use bytes::{Buf, BufMut};
use ua_macros::{DecodeBinary, EncodeBinary};

use crate::encoding::binary;
use crate::ua;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Open,
    Close,
    Msg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum IsFinal {
    Continue = b'C',
    Final = b'F',
    Abort = b'A',
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub msg_type: MsgType,
    pub is_final: IsFinal,
    pub msg_size: u32,
    pub secure_channel_id: u32,
}

impl Header {
    pub const LEN: u32 = 12;
}

impl binary::Encode for Header {
    fn encoded_len(&self) -> u32 {
        12
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        match self.msg_type {
            MsgType::Open => buf.put_slice(b"OPN"),
            MsgType::Close => buf.put_slice(b"CLO"),
            MsgType::Msg => buf.put_slice(b"MSG"),
        }
        buf.put_u8(self.is_final as u8);
        buf.put_u32_le(self.msg_size);
        buf.put_u32_le(self.secure_channel_id);
    }
}

impl binary::Decode for Header {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 8)?;
        let mut typ: [u8; 3] = [0u8; 3];
        buf.copy_to_slice(&mut typ);
        let is_final = buf.get_u8();
        let length = buf.get_u32_le();
        let secure_channel_id = buf.get_u32_le();

        let msg_type = match &typ {
            b"OPN" => MsgType::Open,
            b"CLO" => MsgType::Close,
            b"MSG" => MsgType::Msg,
            _ => return Err(ua::StatusCode::BadTcpMessageTypeInvalid),
        };
        let is_final = match is_final {
            b'C' => IsFinal::Continue,
            b'F' => IsFinal::Final,
            b'A' => IsFinal::Abort,
            _ => return Err(ua::StatusCode::BadDecodingError),
        };
        Ok(Header {
            msg_type,
            is_final,
            msg_size: length,
            secure_channel_id,
        })
    }
}

/// Security header used for asymmetric algorightm negotiation (OpenSecureChannel)
#[derive(Debug, Clone, PartialEq, Eq, Default, EncodeBinary, DecodeBinary)]
pub struct AsymmetricSecurityHeader {
    pub security_policy_uri: Option<String>,
    pub sender_certificate: Option<Vec<u8>>,
    pub receiver_certificate_thumbprint: Option<Vec<u8>>,
}

/// Security header used for symmetric algorightm negotiation (all except OpenSecureChannel)
#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct SymmetricSecurityHeader {
    pub token_id: u32,
}

impl SymmetricSecurityHeader {
    pub const LEN: u32 = 4;
}

#[derive(Debug, Clone, PartialEq, Eq, EncodeBinary, DecodeBinary)]
pub struct SequenceHeader {
    pub sequence_number: u32,
    pub request_id: u32,
}

impl SequenceHeader {
    pub const LEN: u32 = 8;
}
