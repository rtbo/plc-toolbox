use bytes::{Buf, BufMut};

use crate::DataType;
use crate::encoding::binary;

pub type Double = f64;

impl DataType for Double {
    fn type_id() -> u32 {
        11
    }

    fn nullable() -> bool {
        false
    }
}

impl binary::Encode for Double {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_f64_le(*self)
    }
}

impl binary::Decode for Double {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(
        buf: &mut B,
        _ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let val = buf.get_f64_le();
        Ok(val)
    }
}
