use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type Int64 = i64;

impl binary::Encode for Int64 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i64_le(*self)
    }
}

impl binary::Decode for Int64 {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(buf: &mut B, _ctx: &binary::DecodingCtx) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 8)?;
        let val = buf.get_i64_le();
        Ok(val)
    }
}
