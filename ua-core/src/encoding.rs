use std::any::Any;

use crate::ua;

pub mod binary;

pub enum Encoding {
    /// UA-Binary encoding
    Binary,
    /// UA-JSON encoding
    Json,
}

/// An OPC UA request and the response type associated with it.
pub trait Request {
    type Response: Response;
    
    /// Get the request header of this request.
    fn header(&self) -> &ua::RequestHeader;
    /// Set the request header of this request.
    fn with_header(self, header: ua::RequestHeader) -> Self;
}

/// An OPC UA response 
pub trait Response {
    /// Get the response header of this response.
    fn header(&self) -> &ua::ResponseHeader;
    /// Set the response header of this response.
    fn with_header(self, header: ua::ResponseHeader) -> Self;
}



/// A trait that provides the namespace and ID for a DataType.
/// The namespace index (except for NS0) cannot be known in advance and is therefore not included in this trait.,
/// It must be retrieved from the server's NamespaceArray at runtime.
pub trait DataTypeId {
    /// The namespace URI that defines the DataType.
    const NS_URI: &'static str;

    /// The namespace index of this type if it is set explicitly.
    /// This is only set for types that are part of the standard OPC UA namespace (NS0).
    const NS_INDEX: Option<u16> = None;

    /// The numeric ID of the DataType NodeId
    const DATATYPE_ID: u32;

    /// Returns the NodeId for this DataType in the given namespace index.
    /// The namespace index must be determined at runtime based on the server's NamespaceArray.
    fn datatype_node_id(ns_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(ns_index, Self::DATATYPE_ID)
    }
}

/// A trait that provides ID for a binary DataTypeEncoding.
pub trait BinaryEncodingId {
    /// The ID of the binary DataTypeEncoding Object for this type.
    const BINARY_ENCODING_ID: u32;

    /// Returns the NodeId for this binary DataTypeEncoding.
    /// The namespace index must be determined at runtime based on the server's NamespaceArray.
    fn binary_encoding_node_id(ns_index: u16) -> ua::NodeId {
        ua::NodeId::Numeric(ns_index, Self::BINARY_ENCODING_ID)
    }
}

pub trait Ns0DataTypeId: DataTypeId {
    const DATATYPE_NODE_ID: ua::NodeId = ua::NodeId::Numeric(0, Self::DATATYPE_ID);
}

pub trait Ns0BinaryEncodingId: BinaryEncodingId {
    const BINARY_ENCODING_NODE_ID: ua::NodeId = ua::NodeId::Numeric(0, Self::BINARY_ENCODING_ID);
}

pub trait Value: Any + std::fmt::Debug + Send + Sync + binary::Encode {
    /// The URI of the datatype of this value, e.g. "http://opcfoundation.org/UA/" for NS0.
    fn datatype_uri(&self) -> &'static str;

    /// The numeric ID of the datatype of this value within the namespace, e.g. 12 for String in NS0.
    fn datatype_id(&self) -> u32;

    /// The numeric ID of the binary encoding
    fn binary_encoding_id(&self) -> u32 {
        self.datatype_id()
    }
}

impl Value for Box<dyn Value> {
    fn datatype_uri(&self) -> &'static str {
        (**self).datatype_uri()
    }

    fn datatype_id(&self) -> u32 {
        (**self).datatype_id()
    }

    fn binary_encoding_id(&self) -> u32 {
        (**self).binary_encoding_id()
    }
}

impl binary::Encode for Box<dyn Value> {
    fn encoded_len(&self) -> u32 {
        (**self).encoded_len()
    }

    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        (**self).encode(buf)
    }
}
