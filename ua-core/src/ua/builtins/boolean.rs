use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

/// Boolean data type, representing a true or false value.
/// Type node Id: `i=1`
pub type Boolean = bool;

impl DataType for Boolean {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 1)
    }

    fn nullable() -> bool {
        false
    }

    fn default() -> Self {
        false
    }
}

impl binary::Encode for Boolean {
    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(if *self { 1 } else { 0 })
    }
}

impl binary::Decode for Boolean {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_u8();
        Ok(val != 0)
    }
}
