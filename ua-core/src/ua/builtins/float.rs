use bytes::{Buf, BufMut};

use crate::DataType;
use crate::encoding::binary;

pub type Float = f32;

impl DataType for Float {
    fn type_id() -> u32 {
        10
    }

    fn nullable() -> bool {
        false
    }
}

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

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_f32_le();
        Ok(val)
    }
}
