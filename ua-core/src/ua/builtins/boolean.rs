use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

/// Boolean data type, representing a true or false value.
/// Type node Id: `i=1`
pub type Boolean = bool;

impl binary::Encode for Boolean {
    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(if *self { 1 } else { 0 })
    }
}

impl binary::Decode for Boolean {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_u8();
        Ok(val != 0)
    }
}
