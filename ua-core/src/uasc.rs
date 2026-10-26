//! OPC/UA Secure Conversation (UASC) implementation

use crate::encoding::binary;
use ua_macros::{DecodeBinary, EncodeBinary};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsgType {
    Open,
    Close,
    Msg,
}

pub trait TypedMsg {
    const MSG_TYPE: MsgType;
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
    msg_type: MsgType,
    is_final: IsFinal,
    length: u32,
}

impl binary::Encode for Header {
    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        match self.msg_type {
            MsgType::Open => buf.put_slice(b"OPN"),
            MsgType::Close => buf.put_slice(b"CLO"),
            MsgType::Msg => buf.put_slice(b"MSG"),
        }
        buf.put_u8(self.is_final as u8);
        buf.put_u32_le(self.length);
    }
}

impl binary::Decode for Header {
    fn decode<B: binary::Buf>(buf: &mut B, _ctx: &binary::DecodingCtx) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let mut typ: [u8; 3] = [0u8; 3];
        buf.copy_to_slice(&mut typ);
        let is_final = buf.get_u8();
        let length = buf.get_u32_le();

        let msg_type = match &typ {
            b"OPN" => MsgType::Open,
            b"CLO" => MsgType::Close,
            b"MSG" => MsgType::Msg,
            _ => return Err(binary::DecodingError::InvalidData),
        };
        let is_final = match is_final {
            b'C' => IsFinal::Continue,
            b'F' => IsFinal::Final,
            b'A' => IsFinal::Abort,
            _ => return Err(binary::DecodingError::InvalidData),
        };
        Ok(Header {
            msg_type,
            is_final,
            length,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(EncodeBinary, DecodeBinary)]
pub struct SecurityHeader {
    security_policy_uri: Option<String>,
    sender_certificate: Option<Vec<u8>>,
    receiver_certificate_thumbprint: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(EncodeBinary, DecodeBinary)]
pub struct SequenceHeader {
    sequence_number: u32,
    request_id: u32,
}
