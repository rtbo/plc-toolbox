use std::any::Any;

pub mod binary;

pub trait Value: Any + std::fmt::Debug + binary::Encode {
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
