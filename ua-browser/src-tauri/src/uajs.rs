use open62541::ua;
mod attr_id;
pub use attr_id::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceDescription {
    pub node_id: String,
    pub browse_name: String,
    pub display_name: String,
    pub node_class: String,
    pub type_definition: String,
    pub is_forward: bool,
    pub reference_type_id: String,
}

impl From<ua::ReferenceDescription> for ReferenceDescription {
    fn from(value: ua::ReferenceDescription) -> Self {
        Self {
            node_id: value.node_id().to_string(),
            browse_name: value.browse_name().to_string(),
            display_name: value.display_name().text().to_string(),
            node_class: value.node_class().to_string(),
            type_definition: value.type_definition().to_string(),
            is_forward: value.is_forward(),
            reference_type_id: value.reference_type_id().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Variant {
    Empty,
    Scalar(Scalar),
    Array1D(Vec<Scalar>),
    Array {
        dims: Vec<usize>,
        elems: Vec<Scalar>,
    },
}

impl From<&open62541::ua::Variant> for Variant {
    fn from(value: &open62541::ua::Variant) -> Self {
        if value.is_scalar() {
            let varval = value.to_value();
            let open62541::VariantValue::Scalar(scalar) = varval else {
                panic!("Expected a scalar value, but got {:?}", varval);
            };
            Variant::Scalar(scalar.into())
        } else if value.is_empty() {
            Variant::Empty
        } else {
            todo!()
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Scalar {
    /// Represents an unsupported or unrecognized scalar value.
    Unsupported,
    /// Represents a boolean value. (ns=0;i=1)
    Boolean(bool),
    /// Represents a signed 8-bit integer value. (ns=0;i=2)
    SByte(i8),
    /// Represents an unsigned 8-bit integer value. (ns=0;i=3)
    Byte(u8),
    /// Represents a signed 16-bit integer value. (ns=0;i=4)
    Int16(i16),
    /// Represents an unsigned 16-bit integer value. (ns=0;i=5)
    UInt16(u16),
    /// Represents a signed 32-bit integer value. (ns=0;i=6)
    Int32(i32),
    /// Represents an unsigned 32-bit integer value. (ns=0;i=7)
    UInt32(u32),
    /// Represents a signed 64-bit integer value. (ns=0;i=8)
    Int64(i64),
    /// Represents an unsigned 64-bit integer value. (ns=0;i=9)
    UInt64(u64),
    /// Represents a 32-bit floating point value. (ns=0;i=10)
    Float(f32),
    /// Represents a 64-bit floating point value. (ns=0;i=11)
    Double(f64),
    /// Represents a string value. (ns=0;i=12)
    String(String),
    /// Represents a date and time value. (ns=0;i=13)
    /// Represents the number of milliseconds since the Unix epoch.
    DateTime(u64),
    /// Represents a GUID value. (ns=0;i=14)
    Guid(String),
    /// Represents a byte string value. (ns=0;i=15)
    ByteString(Vec<u8>),
    /// Represents a node ID value. (ns=0;i=17)
    NodeId(String),
    /// Represents an expanded node ID value. (ns=0;i=18)
    ExpandedNodeId(String),
    /// Represents a qualified name value. (ns=0;i=20)
    QualifiedName(String),
    /// Represents a localized text value. (ns=0;i=21)
    LocalizedText(String),
}

impl From<open62541::ScalarValue> for Scalar {
    fn from(value: open62541::ScalarValue) -> Self {
        match value {
            open62541::ScalarValue::Boolean(v) => Scalar::Boolean(v.value()),
            open62541::ScalarValue::SByte(v) => Scalar::SByte(v.value()),
            open62541::ScalarValue::Byte(v) => Scalar::Byte(v.value()),
            open62541::ScalarValue::Int16(v) => Scalar::Int16(v.value()),
            open62541::ScalarValue::UInt16(v) => Scalar::UInt16(v.value()),
            open62541::ScalarValue::Int32(v) => Scalar::Int32(v.value()),
            open62541::ScalarValue::UInt32(v) => Scalar::UInt32(v.value()),
            open62541::ScalarValue::Int64(v) => Scalar::Int64(v.value()),
            open62541::ScalarValue::UInt64(v) => Scalar::UInt64(v.value()),
            open62541::ScalarValue::Float(v) => Scalar::Float(v.value()),
            open62541::ScalarValue::Double(v) => Scalar::Double(v.value()),
            open62541::ScalarValue::String(v) => Scalar::String(v.to_string()),
            open62541::ScalarValue::DateTime(v) => {
                Scalar::DateTime((v.as_unix_timestamp_nanos() / 1_000_000i128) as u64)
            }
            open62541::ScalarValue::Guid(v) => Scalar::Guid(v.to_uuid().to_string()),
            open62541::ScalarValue::ByteString(v) => {
                Scalar::ByteString(v.as_bytes().map(|b| b.to_vec()).unwrap_or(Vec::new()))
            }
            open62541::ScalarValue::NodeId(v) => Scalar::NodeId(v.to_string()),
            open62541::ScalarValue::ExpandedNodeId(v) => Scalar::ExpandedNodeId(v.to_string()),
            open62541::ScalarValue::QualifiedName(v) => Scalar::QualifiedName(v.to_string()),
            open62541::ScalarValue::LocalizedText(v) => Scalar::LocalizedText(v.text().to_string()),
            _ => Scalar::Unsupported,
        }
    }
}
