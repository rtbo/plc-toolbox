use crate::encoding::binary;
use crate::{DataType, ua};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Encoding {
    None = 0x00,
    ByteString = 0x01,
    XmlElement = 0x02,
} 

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    pub type_id: ua::NodeId,
    pub encoding: Encoding,
    pub length: u32,
}

impl binary::Encode for Header {
    fn encoded_len(&self) -> u32 {
        self.type_id.encoded_len() + 1 + 4
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        self.type_id.encode(buf);
        (self.encoding as u8).encode(buf);
        self.length.encode(buf);
    }
}

impl binary::Decode for Header {
    fn decode<B: binary::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let type_id = ua::NodeId::decode(buf)?;
        binary::check_remaining(buf, 5)?;
        let encoding = buf.get_u8();
        let length = buf.get_u32_le();
        let encoding = match encoding {
            0x00 => Encoding::None,
            0x01 => Encoding::ByteString,
            0x02 => Encoding::XmlElement,
            _ => return Err(binary::DecodingError::InvalidData),
        };
        Ok(Header {
            type_id,
            encoding,
            length,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    None,
    ByteString(ua::ByteString),
    XmlElement(ua::XmlElement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtensionObject {
    pub type_id: ua::NodeId,
    pub body: Body,
}

impl DataType for ExtensionObject {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 22)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        ExtensionObject {
            type_id: ua::NodeId::Numeric(0, 0),
            body: Body::None,
        }
    }
}

impl binary::Encode for ExtensionObject {
    fn encoded_len(&self) -> u32 {
        self.type_id.encoded_len()
            + 1
            + match &self.body {
                Body::None => 0,
                Body::ByteString(b) => b.encoded_len(),
                Body::XmlElement(x) => x.encoded_len(),
            }
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        self.type_id.encode(buf);
        match &self.body {
            Body::None => {
                0u8.encode(buf);
            }
            Body::ByteString(b) => {
                1u8.encode(buf);
                b.encode(buf)
            }
            Body::XmlElement(x) => {
                2u8.encode(buf);
                x.encode(buf)
            }
        }
    }
}

impl binary::Decode for ExtensionObject {
    fn decode<B: binary::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let type_id = ua::NodeId::decode(buf)?;
        let encoding = u8::decode(buf)?;
        let body = match encoding {
            0x00 => Body::None,
            0x01 => Body::ByteString(ua::ByteString::decode(buf)?),
            0x02 => Body::XmlElement(ua::XmlElement::decode(buf)?),
            _ => return Err(binary::DecodingError::InvalidData),
        };
        Ok(ExtensionObject { type_id, body })
    }
}
