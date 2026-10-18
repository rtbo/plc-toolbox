mod array;
mod attribute_id;
mod builtins;

pub use attribute_id::AttributeId;
pub use builtins::*;

include!(concat!(env!("OUT_DIR"), "/types.rs"));
