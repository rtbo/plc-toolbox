use std::any::Any;

use crate::encoding::binary::{self, Encode};
use crate::ua;

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
    ) -> Result<Self, ua::StatusCode> {
        let type_id = ua::NodeId::decode(buf, ctx)?;
        let encoding = u8::decode(buf, ctx)?;
        let encoding = match encoding {
            0x00 => Encoding::None,
            0x01 => Encoding::ByteString,
            0x02 => Encoding::XmlElement,
            _ => return Err(ua::StatusCode::BadDecodingError),
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

#[derive(Debug)]
pub struct ExtensionObject<T = Box<dyn Body>> {
    pub type_id: ua::NodeId,
    pub body: Option<T>,
}

impl<T> Default for ExtensionObject<T> {
    fn default() -> Self {
        ExtensionObject {
            type_id: ua::NodeId::default(),
            body: None,
        }
    }
}

pub trait Body: Any + std::fmt::Debug {
    fn binary_body_encoded_len(&self) -> u32;
    fn binary_body_encode(&self, buf: &mut dyn binary::BufMut);
}

impl Body for Box<dyn Body> {
    fn binary_body_encoded_len(&self) -> u32 {
        (**self).binary_body_encoded_len()
    }

    fn binary_body_encode(&self, buf: &mut dyn binary::BufMut) {
        (**self).binary_body_encode(buf)
    }
}

impl<T> binary::Encode for ExtensionObject<T>
where
    T: Body,
{
    fn encoded_len(&self) -> u32 {
        let body_len = self
            .body
            .as_ref()
            .map(|b| b.binary_body_encoded_len() as i32);
        let header = Header {
            type_id: self.type_id.clone(),
            encoding: if self.body.is_some() {
                Encoding::ByteString
            } else {
                Encoding::None
            },
            length: body_len,
        };
        header.encoded_len() + body_len.unwrap_or(0) as u32
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        let body_len = self
            .body
            .as_ref()
            .map(|b| b.binary_body_encoded_len() as i32);
        let header = Header {
            type_id: self.type_id.clone(),
            encoding: if self.body.is_some() {
                Encoding::ByteString
            } else {
                Encoding::None
            },
            length: body_len,
        };
        header.encode(buf);
        if let Some(body) = &self.body {
            body.binary_body_encode(buf);
        }
    }
}

pub type BinaryDecodingFn =
    fn(
        &mut dyn binary::Buf,
        &Header,
        &binary::DecodingCtx,
    ) -> Result<Option<Box<dyn ua::extension_object::Body>>, ua::StatusCode>;

/// Generic decoding function for the body of an extension object of type `T`.
/// Can be registered in the `DecodingCtx` type table.
pub fn decode_body<T>(
    mut buf: &mut dyn binary::Buf,
    header: &Header,
    ctx: &binary::DecodingCtx,
) -> Result<Option<Box<dyn ua::extension_object::Body>>, ua::StatusCode>
where
    T: binary::Decode + Body,
{
    if header.encoding != Encoding::ByteString {
        return Err(ua::StatusCode::BadDecodingError);
    }
    let Some(length) = header.length.map(|l| l as usize) else {
        return Err(ua::StatusCode::BadDecodingError);
    };
    let before = binary::Buf::remaining(&buf);
    let value = T::decode(&mut buf, ctx)?;
    if before - binary::Buf::remaining(&buf) != length {
        return Err(ua::StatusCode::BadDecodingError);
    }
    Ok(Some(Box::new(value)))
}

impl binary::Decode for ExtensionObject<Box<dyn Body>> {
    fn decode<B: binary::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let header = Header::decode(buf, ctx)?;
        match header.encoding {
            Encoding::None => Ok(ExtensionObject {
                type_id: header.type_id,
                body: None,
            }),
            Encoding::ByteString => {
                let decoding_fn = ctx
                    .get_decoding_fn(header.type_id.namespace_index())
                    .ok_or(ua::StatusCode::BadDataTypeIdUnknown)?;
                let body = decoding_fn(buf, &header, ctx)?;
                Ok(ExtensionObject {
                    type_id: header.type_id,
                    body,
                })
            }
            Encoding::XmlElement => Err(ua::StatusCode::BadDataEncodingUnsupported), // XML encoding not supported
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum UnknownBody {
    #[default]
    None,
    ByteString(ua::ByteString),
    XmlElement(ua::XmlElement),
}

impl Body for UnknownBody {
    fn binary_body_encoded_len(&self) -> u32 {
        match self {
            UnknownBody::None => 0,
            UnknownBody::ByteString(b) => b.encoded_len() - 4,
            UnknownBody::XmlElement(x) => x.encoded_len() - 4,
        }
    }

    fn binary_body_encode(&self, buf: &mut dyn binary::BufMut) {
        match self {
            UnknownBody::None => {}
            UnknownBody::ByteString(b) => {
                buf.put_slice(b.as_slice());
            }
            UnknownBody::XmlElement(x) => {
                buf.put_slice(x.0.as_bytes());
            }
        }
    }
}
