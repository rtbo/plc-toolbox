use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type Int32 = i32;

impl DataType for Int32 {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 6)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        0
    }
}

impl binary::Encode for Int32 {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        4
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i32_le(*self)
    }
}

impl binary::Decode for Int32 {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let val = buf.get_i32_le();
        Ok(val)
    }
}
