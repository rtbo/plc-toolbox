mod array;
mod attribute_id;
mod builtins;

pub use attribute_id::AttributeId;
pub use builtins::*;
 
// Temporary placeholder for build to compile
pub type ExtensionObject = u32;
 
include!(concat!(env!("OUT_DIR"), "/types.rs"));
