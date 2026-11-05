use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type UInt32 = u32;

impl binary::Encode for UInt32 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        buf.put_u32_le(*self)
    }
}

impl binary::Decode for UInt32 {
    binary::repr_decode_methods!();

    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_u32_le();
        Ok(val)
    }
}
