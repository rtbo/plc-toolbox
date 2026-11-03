use bytes::BufMut;

use crate::encoding::binary;
use crate::ua;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlElement(pub String);

impl binary::Encode for XmlElement {
    fn encoded_len(&self) -> u32 {
        self.0.encoded_len()
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        self.0.encode(buf)
    }
}

impl binary::Decode for XmlElement {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let s = String::decode(buf, ctx)?;
        Ok(XmlElement(s))
    }
}
