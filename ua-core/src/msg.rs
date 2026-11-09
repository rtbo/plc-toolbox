use crate::ua;

pub mod uasc;

/// An OPC UA request and the response type associated with it.
pub trait Request {
    type Response;
}

/// A trait that provides the namespace and ID for a DataType.
/// The namespace index is not always stable across different servers and is therefore not included in this trait.,
/// It must be retrieved from the server's NamespaceArray at runtime.
pub trait DataTypeId {
    /// The namespace URI that defines the DataType.
    const NS_URI: &'static str;

    /// The ID of the DataType for this type.
    const DATATYPE_ID: u32;

    /// The namespace index of this type if it is set explicitly.
    /// This is set only for types that are part of the standard OPC UA namespace (NS0).
    const NS_INDEX: Option<u16> = None;

    /// Construct the NodeID of the DataType for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and must correspond to the namespace URI of this type.
    fn datatype_node_id(ns_index: u16) -> ua::NodeId {
        debug_assert!(Self::NS_INDEX.map_or_else(|| ns_index != 0, |idx| idx == ns_index && ns_index == 0));
        ua::NodeId::Numeric(ns_index, Self::DATATYPE_ID)
    }
}

/// A trait that provides ID for a binary DataTypeEncoding.
pub trait BinaryEncodingId {
    /// The ID of the binary DataTypeEncoding Object for this type.
    const BINARY_ENCODING_ID: u32;

    /// Construct the NodeID of the DataTypeEncoding for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and isn't checked by this function.
    fn binary_encoding_node_id(ns_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(ns_index, Self::BINARY_ENCODING_ID)
    }
}

/// A trait that provides ID for a XML DataTypeEncoding.
pub trait XmlEncodingId {
    /// The ID of the XML DataTypeEncoding Object for this type.
    const XML_ENCODING_ID: u32;

    /// Construct the NodeID of the DataTypeEncoding for this type, provided the namespace index.
    /// The index must be retrieved from the server's NamespaceArray at runtime and isn't checked by this function.
    fn xml_encoding_node_id(ns_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(ns_index, Self::XML_ENCODING_ID)
    }
}
