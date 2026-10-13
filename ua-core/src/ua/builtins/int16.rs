use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type Int16 = i16;

impl DataType for Int16 {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 4)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        0
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

    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 2)?;
        let val = buf.get_i16_le();
        Ok(val)
    }
}
