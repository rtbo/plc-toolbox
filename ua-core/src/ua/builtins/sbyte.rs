use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type SByte = i8;

impl DataType for SByte {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 2)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        0
    }
}

impl binary::Encode for SByte {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i8(*self)
    }
}

impl binary::Decode for SByte {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_i8();
        Ok(val)
    }
}
