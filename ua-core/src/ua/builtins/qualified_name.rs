use ua_macros::{DecodeBinary, EncodeBinary};

#[derive(Debug, Clone, PartialEq, Eq, Default, DecodeBinary, EncodeBinary)]
pub struct QualifiedName {
    pub namespace_index: u16,
    pub name: String,
}
