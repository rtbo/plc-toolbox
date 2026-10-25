use bytes::{Buf, BufMut};

use crate::DataType;
use crate::encoding::binary;

pub type Int64 = i64;

impl DataType for Int64 {
    fn type_id() -> u32 {
        8
    }

    fn nullable() -> bool {
        false
    }
}

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

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let val = buf.get_i64_le();
        Ok(val)
    }
}
