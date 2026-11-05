use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type Byte = u8;

impl binary::Encode for Byte {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        buf.put_u8(*self)
    }
}

impl binary::Decode for Byte {
    binary::repr_decode_methods!();

    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_u8();
        Ok(val)
    }
}
