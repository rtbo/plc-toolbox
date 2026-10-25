use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::DataType;

pub type Int16 = i16;

impl DataType for Int16 {
    fn type_id() -> u32 {
        4
    }

    fn nullable() -> bool {
        false
    }
}

impl binary::Encode for Int16 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        2
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i16_le(*self)
    }
}

impl binary::Decode for Int16 {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 2)?;
        let val = buf.get_i16_le();
        Ok(val)
    }
}
