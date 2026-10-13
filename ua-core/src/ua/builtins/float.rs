use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type Float = f32;

impl DataType for Float {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 10)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        0.0
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

    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_f32_le();
        Ok(val)
    }
}
