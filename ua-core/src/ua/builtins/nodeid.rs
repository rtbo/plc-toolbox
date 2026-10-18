use std::fmt;

use bytes::BufMut;

use crate::msg::binary::{self, Decode, Encode};
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeId {
    Numeric(u16, u32),
    String(u16, String),
    Guid(u16, ua::Guid),
    Opaque(u16, Vec<u8>),
}

/// A node-id that borrows its identifier instead of owning it. This is useful for encoding node-ids without allocating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

impl NodeIdRef<'_> {
    pub fn namespace_index(&self) -> u16 {
        match self {
            NodeIdRef::Numeric(namespace_index, ..) => *namespace_index,
            NodeIdRef::String(namespace_index, ..) => *namespace_index,
            NodeIdRef::Guid(namespace_index, ..) => *namespace_index,
            NodeIdRef::Opaque(namespace_index, ..) => *namespace_index,
        }
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ns = self.namespace_index();
        if ns != 0 {
            write!(f, "ns={};", ns)?;
        }
        match self {
            NodeId::Numeric(_, id) => {
                write!(f, "i={}", id)
            }
            NodeId::String(_, id) => {
                write!(f, "s={}", id)
            }
            NodeId::Guid(_, id) => {
                write!(f, "g={}", id)
            }
            NodeId::Opaque(_, id) => {
                use base64::Engine;
                use base64::engine::general_purpose::STANDARD;
                write!(f, "b={}", STANDARD.encode(id))
            }
        }
    }
}

impl fmt::Display for NodeIdRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ns = self.namespace_index();
        if ns != 0 {
            write!(f, "ns={};", ns)?;
        }
        match self {
            NodeIdRef::Numeric(_, id) => {
                write!(f, "i={}", id)
            }
            NodeIdRef::String(_, id) => {
                write!(f, "s={}", id)
            }
            NodeIdRef::Guid(_, id) => {
                write!(f, "g={}", id)
            }
            NodeIdRef::Opaque(_, id) => {
                use base64::Engine;
                use base64::engine::general_purpose::STANDARD;
                write!(f, "b={}", STANDARD.encode(id))
            }
        }
    }
}

impl std::str::FromStr for NodeId {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ns, id) = if let Some(ns_index) = s.strip_prefix("ns=") {
            let semicolon_pos = ns_index.find(';').ok_or("Invalid NodeId format")?;
            let ns_str = &ns_index[..semicolon_pos];
            let ns: u16 = ns_str.parse().map_err(|_| "Invalid namespace index")?;
            let id = &ns_index[semicolon_pos + 1..];
            (ns, id)
        } else {
            (0, s)
        };

        if let Some(id) = id.strip_prefix("i=") {
            let identifier: u32 = id.parse().map_err(|_| "Invalid numeric identifier")?;
            Ok(NodeId::Numeric(ns, identifier))
        } else if let Some(id) = id.strip_prefix("s=") {
            Ok(NodeId::String(ns, id.to_string()))
        } else if let Some(id) = id.strip_prefix("g=") {
            let guid: ua::Guid = id.parse().map_err(|_| "Invalid GUID identifier")?;
            Ok(NodeId::Guid(ns, guid))
        } else if let Some(id) = id.strip_prefix("b=") {
            use base64::Engine;
            use base64::engine::general_purpose::STANDARD;
            let bytes = STANDARD
                .decode(id)
                .map_err(|_| "Invalid base64 identifier")?;
            Ok(NodeId::Opaque(ns, bytes))
        } else {
            Err("Invalid NodeId format")
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_nodeid_display() {
        let node_id_numeric = NodeId::Numeric(0, 42);
        assert_eq!(node_id_numeric.to_string(), "i=42");

        let node_id_string = NodeId::String(1, "Hello".to_string());
        assert_eq!(node_id_string.to_string(), "ns=1;s=Hello");

        let guid = ua::Guid {
            data1: 0x12345678,
            data2: 0x1234,
            data3: 0x5678,
            data4: [0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78],
        };
        let node_id_guid = NodeId::Guid(2, guid);
        assert_eq!(
            node_id_guid.to_string(),
            "ns=2;g=12345678-1234-5678-90AB-CDEF12345678"
        );

        let node_id_opaque = NodeId::Opaque(3, vec![1, 2, 3, 4]);
        assert_eq!(node_id_opaque.to_string(), "ns=3;b=AQIDBA==");
    }

    #[test]
    fn test_nodeid_from_str() {
        let node_id_numeric: NodeId = "i=42".parse().unwrap();
        assert_eq!(node_id_numeric, NodeId::Numeric(0, 42));

        let node_id_string: NodeId = "ns=1;s=Hello".parse().unwrap();
        assert_eq!(node_id_string, NodeId::String(1, "Hello".to_string()));

        let guid_str = "12345678-1234-5678-90AB-CDEF12345678";
        let node_id_guid: NodeId = format!("ns=2;g={}", guid_str).parse().unwrap();
        let guid = ua::Guid {
            data1: 0x12345678,
            data2: 0x1234,
            data3: 0x5678,
            data4: [0x90, 0xab, 0xcd, 0xef, 0x12, 0x34, 0x56, 0x78],
        };
        assert_eq!(node_id_guid, NodeId::Guid(2, guid));

        let node_id_opaque: NodeId = "ns=3;b=AQIDBA==".parse().unwrap();
        assert_eq!(node_id_opaque, NodeId::Opaque(3, vec![1, 2, 3, 4]));
    }
}
