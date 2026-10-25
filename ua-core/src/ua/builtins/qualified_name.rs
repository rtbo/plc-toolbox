use ua_macros::{DecodeBinary, EncodeBinary};

use crate::DataType;

#[derive(Debug, Clone, PartialEq, Eq, Default, DecodeBinary, EncodeBinary)]
pub struct QualifiedName {
    pub namespace_index: u16,
    pub name: String,
}

impl DataType for QualifiedName {
    fn type_id() -> u32 {
        20
    }
    fn nullable() -> bool {
        true
    }
}
