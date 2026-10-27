use crate::ua;

pub mod uasc;

/// A trait that provides the namespace and ID for a DataType.
/// The namespace index is not always stable across different servers and is therefore not included in this trait.,
/// It must be retrieved from the server's NamespaceArray at runtime.
pub trait DataTypeId {
    /// The namespace URI that defines the DataType.
    const NS_URI: &'static str;

    /// The ID of the DataType for this type.
    const DATATYPE_ID: u32;

    /// Construct the NodeID of the DataType for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and isn't checked by this function.
    fn datatype_node_id(namespace_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(namespace_index, Self::DATATYPE_ID)
    }
}

/// A trait that provides ID for a binary DataTypeEncoding.
pub trait BinaryEncodingId: DataTypeId {
    /// The ID of the binary DataTypeEncoding Object for this type.
    const BINARY_ENCODING_ID: u32;

    /// Construct the NodeID of the DataTypeEncoding for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and isn't checked by this function.
    fn binary_encoding_node_id(namespace_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(namespace_index, Self::BINARY_ENCODING_ID)
    }
}

/// A trait that provides ID for a XML DataTypeEncoding.
pub trait XmlEncodingId: DataTypeId {
    /// The ID of the XML DataTypeEncoding Object for this type.
    const XML_ENCODING_ID: u32;

    /// Construct the NodeID of the DataTypeEncoding for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and isn't checked by this function.
    fn xml_encoding_node_id(namespace_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(namespace_index, Self::XML_ENCODING_ID)
    }
}

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum Type {
//     /// HEL UACP message
//     Hello,
//     /// ACK UACP message
//     Acknowledge,
//     /// ERR UACP message
//     Error,
//     /// RHE UACP message
//     ReverseHello,
//     /// OPN UASC message
//     Open,
//     /// CLO UASC message
//     Close,
//     /// MSG UASC message
//     Message,
// }
//
// impl Type {
//     pub fn as_bytes(&self) -> &[u8; 3] {
//         match self {
//             Type::Hello => b"HEL",
//             Type::Acknowledge => b"ACK",
//             Type::Error => b"ERR",
//             Type::ReverseHello => b"RHE",
//             Type::Open => b"OPN",
//             Type::Close => b"CLO",
//             Type::Message => b"MSG",
//         }
//     }
// }
//
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[repr(u8)]
// pub enum IsFinal {
//     Continue = b'C',
//     Final = b'F',
//     Abort = b'A',
// }
//
// /// A generic binary header for messages.
// /// This maps the first 8 bytes of every binary message and every chunk.
// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub struct BinaryHeader {
//     msg_type: Type,
//     is_final: IsFinal,
//     msg_size: u32,
// }
//
// impl binary::Encode for BinaryHeader {
//     fn encoded_len(&self) -> u32 {
//         8
//     }
//
//     fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
//         buf.put_slice(self.msg_type.as_bytes());
//         buf.put_u8(self.is_final as u8);
//         buf.put_u32(self.msg_size);
//     }
// }
//
// impl binary::Decode for BinaryHeader {
//     fn decode<B: bytes::Buf>(
//         buf: &mut B,
//         _ctx: &binary::DecodingCtx,
//     ) -> Result<Self, binary::DecodingError> {
//         let mut msg_type_bytes = [0u8; 3];
//         buf.copy_to_slice(&mut msg_type_bytes);
//         let msg_type = match &msg_type_bytes {
//             b"HEL" => Type::Hello,
//             b"ACK" => Type::Acknowledge,
//             b"ERR" => Type::Error,
//             b"RHE" => Type::ReverseHello,
//             b"OPN" => Type::Open,
//             b"CLO" => Type::Close,
//             b"MSG" => Type::Message,
//             _ => return Err(binary::DecodingError::BadMessageType),
//         };
//         let is_final_byte = buf.get_u8();
//         let is_final = match is_final_byte {
//             b'C' => IsFinal::Continue,
//             b'F' => IsFinal::Final,
//             b'A' => IsFinal::Abort,
//             _ => return Err(binary::DecodingError::InvalidData),
//         };
//         let msg_size = buf.get_u32();
//         Ok(Self {
//             msg_type,
//             is_final,
//             msg_size,
//         })
//     }
// }
//
// pub trait BinaryMsg: binary::Encode + binary::Decode {
// }
