use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type Float = f32;

impl binary::Encode for Float {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_f32_le(*self)
    }
}

impl binary::Decode for Float {
    binary::repr_decode_methods!();

    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_f32_le();
        Ok(val)
    }
}
