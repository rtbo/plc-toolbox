use std::any::Any;

use crate::ua;

pub mod binary;

pub trait Value: Any + std::fmt::Debug {
    /// The URI of the datatype of this value, e.g. "http://opcfoundation.org/UA/" for NS0.
    fn datatype_uri(&self) -> &'static str;

    /// The numeric ID of the datatype of this value within the namespace, e.g. 12 for String in NS0.
    fn datatype_id(&self) -> u32;

    /// The numeric ID of the binary encoding
    fn binary_encoding_id(&self) -> u32 {
        self.datatype_id()
    }

    fn binary_encoded_len(&self) -> u32;
    fn binary_encode(&self, buf: &mut dyn binary::BufMut);
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

    fn binary_encoded_len(&self) -> u32 {
        (**self).binary_encoded_len()
    }

    fn binary_encode(&self, buf: &mut dyn binary::BufMut) {
        (**self).binary_encode(buf)
    }
}
