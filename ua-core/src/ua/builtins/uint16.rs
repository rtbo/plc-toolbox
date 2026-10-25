use bytes::{Buf, BufMut};

use crate::DataType;
use crate::encoding::binary;

pub type UInt16 = u16;

impl DataType for UInt16 {
    fn type_id() -> u32 {
        5
    }

    fn nullable() -> bool {
        false
    }
}

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

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 2)?;
        let val = buf.get_u16_le();
        Ok(val)
    }
}
