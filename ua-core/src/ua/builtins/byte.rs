use bytes::{Buf, BufMut};

use crate::encoding::binary;

pub type Byte = u8;

impl binary::Encode for Byte {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self)
    }
}

impl binary::Decode for Byte {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_u8();
        Ok(val)
    }
}
