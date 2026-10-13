use ua_macros::{DecodeBinary, EncodeBinary};

use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq, DecodeBinary, EncodeBinary)]
pub struct QualifiedName {
    pub namespace_index: u16,
    pub name: String,
}

impl DataType for QualifiedName {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 20)
    }
    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        QualifiedName {
            namespace_index: 0,
            name: String::new(),
        }
    }
}
