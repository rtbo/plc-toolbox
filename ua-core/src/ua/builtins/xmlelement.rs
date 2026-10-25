use bytes::{Buf, BufMut};

use crate::DataType;
use crate::encoding::binary;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement(pub String);

impl DataType for XmlElement {
    fn type_id() -> u32 {
        16
    }

    fn nullable() -> bool {
        true
    }
}

impl binary::Encode for XmlElement {
    fn encoded_len(&self) -> u32 {
        self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.0.encode(buf)
    }
}

impl binary::Decode for XmlElement {
    fn decode<B: Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        let s = String::decode(buf, ctx)?;
        Ok(XmlElement(s))
    }
}
