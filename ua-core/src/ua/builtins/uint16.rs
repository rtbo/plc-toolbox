use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type UInt16 = u16;

impl binary::Encode for UInt16 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        2
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u16_le(*self)
    }
}

impl binary::Decode for UInt16 {
    binary::repr_decode_methods!();

    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 2)?;
        let val = buf.get_u16_le();
        Ok(val)
    }
}
