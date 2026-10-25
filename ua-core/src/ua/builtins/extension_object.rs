use std::any::Any;

use crate::encoding::binary;
use crate::{DataType, ua};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Encoding {
    #[default]
    None = 0x00,
    ByteString = 0x01,
    XmlElement = 0x02,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Header {
    pub type_id: ua::NodeId,
    pub encoding: Encoding,
    pub length: Option<i32>,
}

impl binary::Encode for Header {
    fn encoded_len(&self) -> u32 {
        let lenlen = match self.length {
            Some(_) => 4,
            None => 0,
        };
        self.type_id.encoded_len() + 1 + lenlen
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        self.type_id.encode(buf);
        (self.encoding as u8).encode(buf);
        if let Some(length) = self.length {
            length.encode(buf);
        }
    }
}

impl binary::Decode for Header {
    fn decode<B: binary::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let type_id = ua::NodeId::decode(buf, ctx)?;
        let encoding = u8::decode(buf, ctx)?;
        let encoding = match encoding {
            0x00 => Encoding::None,
            0x01 => Encoding::ByteString,
            0x02 => Encoding::XmlElement,
            _ => return Err(binary::DecodingError::InvalidData),
        };
        let length = if encoding == Encoding::None {
            None
        } else {
            Some(i32::decode(buf, ctx)?)
        };
        Ok(Header {
            type_id,
            encoding,
            length,
        })
    }
}

pub trait ExtensionObject: Any + std::fmt::Debug {
    fn ua_type_id(&self) -> ua::NodeId;

    fn binary_encoded_len_body(&self) -> u32;
    fn binary_encode_body(&self, buf: &mut dyn binary::BufMut);
}

impl binary::Encode for Option<Box<dyn ExtensionObject>> {
    fn encoded_len(&self) -> u32 {
        match self {
            Some(obj) => {
                let body_len = obj.binary_encoded_len_body();
                let header = Header {
                    type_id: obj.ua_type_id(),
                    encoding: Encoding::ByteString,
                    length: Some(body_len as i32),
                };
                header.encoded_len() + body_len
            }
            None => ua::NodeId::Numeric(0, 0).encoded_len() + 1,
        }
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        match self {
            Some(obj) => {
                obj.ua_type_id().encode(buf);
                (Encoding::ByteString as u8).encode(buf);
                let body_len = obj.binary_encoded_len_body();
                (body_len as i32).encode(buf);
                obj.binary_encode_body(buf);
            }
            None => {
                ua::NodeId::Numeric(0, 0).encode(buf);
                0u8.encode(buf);
            }
        }
    }
}

pub type BinaryDecodingFn =
    fn(
        &mut dyn binary::Buf,
        &Encoding,
        i32,
        &binary::DecodingCtx,
    ) -> Result<Option<Box<dyn ua::ExtensionObject>>, binary::DecodingError>;

impl binary::Decode for Option<Box<dyn ExtensionObject>> {
    fn decode<B: binary::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let header = Header::decode(buf, ctx)?;
        match header.encoding {
            Encoding::None => Ok(None),
            Encoding::ByteString | Encoding::XmlElement => {
                let decoding_fn = ctx
                    .get_decoding_fn(&header.type_id)
                    .ok_or(binary::DecodingError::DataTypeIdUnknown)?;
                let obj = decoding_fn(buf, &header.encoding, header.length.unwrap_or(0), ctx)?;
                Ok(obj)
            }
        }
    }
}

impl crate::DataType for Option<Box<dyn ExtensionObject>> {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 22)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnknownBody {
    None,
    ByteString(ua::ByteString),
    XmlElement(ua::XmlElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownExtensionObject {
    pub type_id: ua::NodeId,
    pub body: UnknownBody,
}

impl DataType for UnknownExtensionObject {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 22)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        UnknownExtensionObject {
            type_id: ua::NodeId::Numeric(0, 0),
            body: UnknownBody::None,
        }
    }
}

impl binary::Encode for UnknownExtensionObject {
    fn encoded_len(&self) -> u32 {
        self.type_id.encoded_len()
            + 1
            + match &self.body {
                UnknownBody::None => 0,
                UnknownBody::ByteString(b) => b.encoded_len(),
                UnknownBody::XmlElement(x) => x.encoded_len(),
            }
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        self.type_id.encode(buf);
        match &self.body {
            UnknownBody::None => {
                0u8.encode(buf);
            }
            UnknownBody::ByteString(b) => {
                1u8.encode(buf);
                b.encode(buf)
            }
            UnknownBody::XmlElement(x) => {
                2u8.encode(buf);
                x.encode(buf)
            }
        }
    }
}

impl binary::Decode for UnknownExtensionObject {
    fn decode<B: binary::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let type_id = ua::NodeId::decode(buf, ctx)?;
        let encoding = u8::decode(buf, ctx)?;
        // length field included in the bytestring or xmlelement, so we don't need to read it here
        let body = match encoding {
            0x00 => UnknownBody::None,
            0x01 => UnknownBody::ByteString(ua::ByteString::decode(buf, ctx)?),
            0x02 => UnknownBody::XmlElement(ua::XmlElement::decode(buf, ctx)?),
            _ => return Err(binary::DecodingError::InvalidData),
        };
        Ok(UnknownExtensionObject { type_id, body })
    }
}
