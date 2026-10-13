use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement(pub String);

impl DataType for XmlElement {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 16)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        XmlElement(String::new())
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
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let s = String::decode(buf)?;
        Ok(XmlElement(s))
    }
}
