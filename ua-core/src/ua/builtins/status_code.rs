use crate::encoding::binary;

include!(concat!(env!("OUT_DIR"), "/status_code.rs"));

impl Default for StatusCode {
    fn default() -> Self {
        StatusCode::Good
    }
}

impl StatusCode {
    pub fn is_good(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0
    }

    pub fn is_uncertain(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0x4000_0000
    }

    pub fn is_bad(&self) -> bool {
        let code = *self as u32;
        code & 0xC000_0000 == 0x8000_0000
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}, reason: {}", self.name(), self.explanation())
    }
}

impl std::error::Error for StatusCode {}

impl binary::Encode for StatusCode {
    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(*self as u32)
    }
}

impl binary::Decode for StatusCode {
    fn decode<B: bytes::Buf>(buf: &mut B, _ctx: &binary::DecodingCtx) -> Result<Self, StatusCode> {
        binary::check_remaining(buf, 4)?;
        let code = buf.get_u32_le();
        Self::try_from(code).map_err(|_| StatusCode::BadDecodingError)
    }
}
