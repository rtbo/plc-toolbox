use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type String = std::string::String;

impl DataType for String {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 12)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        String::new()
    }
}

impl binary::Encode for &str {
    fn encoded_len(&self) -> u32 {
        self.as_bytes().encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.as_bytes().encode(buf)
    }
}

impl binary::Encode for String {
    fn encoded_len(&self) -> u32 {
        self.as_bytes().encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.as_bytes().encode(buf)
    }
}

impl binary::Encode for Option<&str> {
    fn encoded_len(&self) -> u32 {
        self.map(|s| s.as_bytes()).encoded_len()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.map(|s| s.as_bytes()).encode(buf)
    }
}

impl binary::Encode for Option<String> {
    fn encoded_len(&self) -> u32 {
        self.as_deref().encoded_len()
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.as_deref().encode(buf)
    }
}

impl binary::Decode for String {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let bytes: Vec<u8> = binary::Decode::decode(buf)?;
        Ok(std::string::String::from_utf8(bytes)?)
    }
}

impl binary::Decode for Option<String> {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let bytes: Option<Vec<u8>> = binary::Decode::decode(buf)?;
        bytes
            .map(|b| std::string::String::from_utf8(b))
            .transpose()
            .map_err(|e| e.into())
    }
}
