use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type Int64 = i64;

impl DataType for Int64 {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 8)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        0
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

    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let val = buf.get_i64_le();
        Ok(val)
    }
}
