mod attribute_id;
mod node_id;

pub use attribute_id::AttributeId;

pub type Boolean = bool;
pub type SByte = i8;
pub type Byte = u8;
pub type Int16 = i16;
pub type UInt16 = u16;
pub type Int32 = i32;
pub type UInt32 = u32;
pub type Int64 = i64;
pub type UInt64 = u64;
pub type Float = f32;
pub type Double = f64;
pub type String = std::string::String;
pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Guid = uuid::Uuid;
pub type ByteString = Vec<u8>;
pub struct XmlElement(pub String);
