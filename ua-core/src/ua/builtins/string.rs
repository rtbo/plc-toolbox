use crate::encoding::binary;
use crate::ua;

pub type String = std::string::String;

impl binary::Encode for &str {
    fn encoded_len(&self) -> u32 {
        self.as_bytes().encoded_len()
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        self.as_bytes().encode(buf)
    }
}

impl binary::Encode for String {
    fn encoded_len(&self) -> u32 {
        self.as_bytes().encoded_len()
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        self.as_bytes().encode(buf)
    }
}

impl binary::Encode for Option<&str> {
    fn encoded_len(&self) -> u32 {
        self.map(|s| s.as_bytes()).encoded_len()
    }
    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        self.map(|s| s.as_bytes()).encode(buf)
    }
}

impl binary::Encode for Option<String> {
    fn encoded_len(&self) -> u32 {
        self.as_deref().encoded_len()
    }
    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        self.as_deref().encode(buf)
    }
}

impl binary::Decode for String {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let bytes: Vec<u8> = binary::Decode::decode(buf, ctx)?;
        let str =
            std::string::String::from_utf8(bytes).map_err(|_| ua::StatusCode::BadDecodingError)?;
        Ok(str)
    }
}

impl binary::Decode for Option<String> {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let bytes: Option<Vec<u8>> = binary::Decode::decode(buf, ctx)?;
        bytes
            .map(|b| std::string::String::from_utf8(b))
            .transpose()
            .map_err(|_| ua::StatusCode::BadDecodingError)
    }
}
