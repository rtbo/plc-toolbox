use crate::encoding::binary;

pub trait DataType: Sized + binary::Encode + binary::Decode {
    /// Identifier of the data type, without its namespace.
    /// The namespace index depends on the server's namespace array.
    fn type_id() -> u32;
    fn nullable() -> bool;
}
