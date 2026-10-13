use crate::{DataType, ua};

pub type ByteString = Vec<u8>;

// encode and decode implemented as generics in ua::array

impl DataType for Vec<u8> {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 15)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        Vec::new()
    }
}
