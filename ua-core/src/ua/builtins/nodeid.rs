use bytes::BufMut;

use crate::msg::binary::{self, Decode, Encode};
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeId {
    Numeric(u16, u32),
    String(u16, String),
    Guid(u16, ua::Guid),
    Opaque(u16, Vec<u8>),
}

/// A node-id that borrows its identifier instead of owning it. This is useful for encoding node-ids without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeIdRef<'a> {
    Numeric(u16, u32),
    String(u16, &'a str),
    Guid(u16, ua::Guid),
    Opaque(u16, &'a [u8]),
}

impl NodeId {
    pub fn namespace_index(&self) -> u16 {
        match self {
            NodeId::Numeric(namespace_index, ..) => *namespace_index,
            NodeId::String(namespace_index, ..) => *namespace_index,
            NodeId::Guid(namespace_index, ..) => *namespace_index,
            NodeId::Opaque(namespace_index, ..) => *namespace_index,
        }
    }

    pub fn numeric_identifier(&self) -> Option<u32> {
        match self {
            NodeId::Numeric(_, identifier) => Some(*identifier),
            _ => None,
        }
    }

    pub fn string_identifier(&self) -> Option<&String> {
        match self {
            NodeId::String(_, identifier) => Some(identifier),
            _ => None,
        }
    }

    pub fn guid_identifier(&self) -> Option<&ua::Guid> {
        match self {
            NodeId::Guid(_, identifier) => Some(identifier),
            _ => None,
        }
    }

    pub fn opaque_identifier(&self) -> Option<&Vec<u8>> {
        match self {
            NodeId::Opaque(_, identifier) => Some(identifier),
            _ => None,
        }
    }

    pub fn as_ref(&self) -> NodeIdRef<'_> {
        match self {
            NodeId::Numeric(ns, id) => NodeIdRef::Numeric(*ns, *id),
            NodeId::String(ns, id) => NodeIdRef::String(*ns, id.as_str()),
            NodeId::Guid(ns, id) => NodeIdRef::Guid(*ns, *id),
            NodeId::Opaque(ns, id) => NodeIdRef::Opaque(*ns, id.as_slice()),
        }
    }
}

impl DataType for NodeId {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 17)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        NodeId::Numeric(0, 0)
    }
}

impl binary::Encode for NodeId {
    fn encoded_len(&self) -> u32 {
        nodeid_encoded_len(self)
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        nodeid_encode(self, buf, 0);
    }
}

impl binary::Encode for NodeIdRef<'_> {
    fn encoded_len(&self) -> u32 {
        nodeidref_encoded_len(self)
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        nodeidref_encode(self, buf, 0);
    }
}

impl binary::Decode for NodeId {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let (nodeid, _) = nodeid_decode(buf)?;
        Ok(nodeid)
    }
}

pub(super) fn nodeid_encoded_len(node_id: &NodeId) -> u32 {
    match node_id {
        NodeId::Numeric(ns, i) if *ns == 0 && *i <= u8::MAX as u32 => 1 + 1,
        NodeId::Numeric(ns, i) if *ns <= u8::MAX as u16 && *i <= u16::MAX as u32 => 1 + 1 + 2,
        NodeId::Numeric(_, _) => 1 + 2 + 4,
        NodeId::String(_, s) => 1 + 2 + s.encoded_len(),
        NodeId::Guid(_, g) => 1 + 2 + g.encoded_len(),
        NodeId::Opaque(_, b) => 1 + 2 + b.encoded_len(),
    }
}

pub(super) fn nodeid_encode<B: BufMut>(node_id: &NodeId, buf: &mut B, flags: u8) {
    match node_id {
        NodeId::Numeric(ns, i) if *ns == 0 && *i <= u8::MAX as u32 => {
            buf.put_u8(0x00 | flags);
            buf.put_u8(*i as u8);
        }
        NodeId::Numeric(ns, i) if *ns <= u8::MAX as u16 && *i <= u16::MAX as u32 => {
            buf.put_u8(0x01 | flags);
            buf.put_u8(*ns as u8);
            buf.put_u16_le(*i as u16);
        }
        NodeId::Numeric(ns, i) => {
            buf.put_u8(0x02 | flags);
            buf.put_u16_le(*ns);
            buf.put_u32_le(*i);
        }
        NodeId::String(namespace_index, identifier) => {
            buf.put_u8(0x03 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
        NodeId::Guid(namespace_index, identifier) => {
            buf.put_u8(0x04 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
        NodeId::Opaque(namespace_index, identifier) => {
            buf.put_u8(0x05 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
    }
}

pub(super) fn nodeid_decode<B: bytes::Buf>(
    buf: &mut B,
) -> Result<(NodeId, u8), binary::DecodingError> {
    let type_byte = buf.get_u8();
    let flags = type_byte & 0xF0;
    let node_id_type = type_byte & 0x0F;

    match node_id_type {
        0x00 => {
            let identifier = buf.get_u8() as u32;
            Ok((NodeId::Numeric(0, identifier), flags))
        }
        0x01 => {
            let namespace_index = buf.get_u8() as u16;
            let identifier = buf.get_u16_le() as u32;
            Ok((NodeId::Numeric(namespace_index, identifier), flags))
        }
        0x02 => {
            let namespace_index = buf.get_u16_le();
            let identifier = buf.get_u32_le();
            Ok((NodeId::Numeric(namespace_index, identifier), flags))
        }
        0x03 => {
            let namespace_index = buf.get_u16_le();
            let identifier = String::decode(buf)?;
            Ok((NodeId::String(namespace_index, identifier), flags))
        }
        0x04 => {
            let namespace_index = buf.get_u16_le();
            let identifier = ua::Guid::decode(buf)?;
            Ok((NodeId::Guid(namespace_index, identifier), flags))
        }
        0x05 => {
            let namespace_index = buf.get_u16_le();
            let identifier = Vec::<u8>::decode(buf)?;
            Ok((NodeId::Opaque(namespace_index, identifier), flags))
        }
        _ => Err(binary::DecodingError::InvalidData),
    }
}

pub(super) fn nodeidref_encoded_len(node_id: &NodeIdRef) -> u32 {
    match node_id {
        NodeIdRef::Numeric(ns, i) if *ns == 0 && *i <= u8::MAX as u32 => 1 + 1,
        NodeIdRef::Numeric(ns, i) if *ns <= u8::MAX as u16 && *i <= u16::MAX as u32 => 1 + 1 + 2,
        NodeIdRef::Numeric(_, _) => 1 + 2 + 4,
        NodeIdRef::String(_, s) => 1 + 2 + s.encoded_len(),
        NodeIdRef::Guid(_, g) => 1 + 2 + g.encoded_len(),
        NodeIdRef::Opaque(_, b) => 1 + 2 + b.encoded_len(),
    }
}

pub(super) fn nodeidref_encode<B: BufMut>(node_id: &NodeIdRef, buf: &mut B, flags: u8) {
    match node_id {
        NodeIdRef::Numeric(ns, i) if *ns == 0 && *i <= u8::MAX as u32 => {
            buf.put_u8(0x00 | flags);
            buf.put_u8(*i as u8);
        }
        NodeIdRef::Numeric(ns, i) if *ns <= u8::MAX as u16 && *i <= u16::MAX as u32 => {
            buf.put_u8(0x01 | flags);
            buf.put_u8(*ns as u8);
            buf.put_u16_le(*i as u16);
        }
        NodeIdRef::Numeric(ns, i) => {
            buf.put_u8(0x02 | flags);
            buf.put_u16_le(*ns);
            buf.put_u32_le(*i);
        }
        NodeIdRef::String(namespace_index, identifier) => {
            buf.put_u8(0x03 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
        NodeIdRef::Guid(namespace_index, identifier) => {
            buf.put_u8(0x04 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
        NodeIdRef::Opaque(namespace_index, identifier) => {
            buf.put_u8(0x05 | flags);
            buf.put_u16_le(*namespace_index);
            identifier.encode(buf);
        }
    }
}
