use crate::DataType;

pub type ByteString = Vec<u8>;

// encode and decode implemented as generics in ua::array

impl DataType for Vec<u8> {
    fn type_id() -> u32 {
        15
    }

    fn nullable() -> bool {
        true
    }
}
