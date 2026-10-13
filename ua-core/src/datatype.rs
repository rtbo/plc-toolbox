use crate::msg::binary;
use crate::ua;

pub trait DataType: Sized + binary::Encode + binary::Decode {
    fn type_id() -> ua::NodeId;
    fn nullable() -> bool;
    fn default() -> Self;
}
