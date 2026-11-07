use crate::encoding::binary;
use crate::{encoding, ua};

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

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        self.type_id.encode(buf);
        (self.encoding as u8).encode(buf);
        if let Some(length) = self.length {
            length.encode(buf);
        }
    }
}

impl binary::Decode for Header {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
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

#[derive(Debug, PartialEq)]
pub struct ExtensionObject<T = Box<dyn encoding::Value>> {
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

impl<T> binary::Encode for ExtensionObject<T>
where
    T: encoding::Value,
{
    fn encoded_len(&self) -> u32 {
        let body_len = self.body.as_ref().map(|b| b.encoded_len() as i32);
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

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        let body_len = self.body.as_ref().map(|b| b.encoded_len() as i32);
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
            body.encode(buf);
        }
    }
}

impl binary::Decode for ExtensionObject<Box<dyn encoding::Value>> {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
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
                let body = decoding_fn(
                    buf,
                    header
                        .type_id
                        .numeric_identifier()
                        .expect("expected a numeric identifier"),
                    ctx,
                )?;
                Ok(ExtensionObject {
                    type_id: header.type_id,
                    body,
                })
            }
            Encoding::XmlElement => Err(ua::StatusCode::BadDataEncodingUnsupported), // XML encoding not supported
        }
    }
}

// #[derive(Debug, Clone, PartialEq, Eq, Default)]
// pub enum UnknownBody {
//     #[default]
//     None,
//     ByteString(ua::ByteString),
//     XmlElement(ua::XmlElement),
// }
//
// impl Body for UnknownBody {
//     fn binary_body_encoded_len(&self) -> u32 {
//         match self {
//             UnknownBody::None => 0,
//             UnknownBody::ByteString(b) => b.encoded_len() - 4,
//             UnknownBody::XmlElement(x) => x.encoded_len() - 4,
//         }
//     }
//
//     fn binary_body_encode(&self, buf: &mut dyn binary::BufMut) {
//         match self {
//             UnknownBody::None => {}
//             UnknownBody::ByteString(b) => {
//                 buf.put_slice(b.as_slice());
//             }
//             UnknownBody::XmlElement(x) => {
//                 buf.put_slice(x.0.as_bytes());
//             }
//         }
//     }
// }
