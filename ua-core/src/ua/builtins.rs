use crate::msg::binary;

mod boolean;
mod byte;
mod bytestring;
mod datavalue;
mod datetime;
mod diagnostic_info;
mod double;
mod expanded_nodeid;
pub mod extension_object;
mod float;
mod guid;
mod int16;
mod int32;
mod int64;
mod localized_text;
mod nodeid;
mod qualified_name;
mod sbyte;
mod status_code;
mod string;
mod uint16;
mod uint32;
mod uint64;
mod variant;
mod xmlelement;

pub use boolean::Boolean;
pub use byte::Byte;
pub use bytestring::ByteString;
pub use datavalue::DataValue;
pub use datetime::DateTime;
pub use diagnostic_info::DiagnosticInfo;
pub use double::Double;
pub use expanded_nodeid::ExpandedNodeId;
pub use extension_object::ExtensionObject;
pub use float::Float;
pub use guid::Guid;
pub use int16::Int16;
pub use int32::Int32;
pub use int64::Int64;
pub use localized_text::{LocaleId, LocalizedText};
pub use nodeid::NodeId;
pub use qualified_name::QualifiedName;
pub use sbyte::SByte;
pub use status_code::StatusCode;
pub use string::String;
pub use uint16::UInt16;
pub use uint32::UInt32;
pub use uint64::UInt64;
pub use variant::Variant;
pub use xmlelement::XmlElement;

unsafe impl binary::ReprEncoded for SByte {}
unsafe impl binary::ReprEncoded for Byte {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for Int16 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for UInt16 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for Int32 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for UInt32 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for Int64 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for UInt64 {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for Float {}
#[cfg(target_endian = "little")]
unsafe impl binary::ReprEncoded for Double {}
