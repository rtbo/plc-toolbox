use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type Int32 = i32;

impl binary::Encode for Int32 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32_le(*self)
    }
}

impl binary::Decode for Int32 {
    binary::repr_decode_methods!();

    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_i32_le();
        Ok(val)
    }
}
