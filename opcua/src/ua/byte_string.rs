use std::fmt;

use crate::ffi;

#[repr(transparent)]
pub struct ByteString {
    raw: ffi::UA_ByteString,
}

unsafe impl crate::DataType for ByteString {
    type Raw = ffi::UA_ByteString;
    const UA_TYPE_IDX: usize = ffi::UA_TYPES_BYTESTRING as usize;
    const NAME: &'static str = "ByteString";

    unsafe fn from_raw(raw: Self::Raw) -> Self {
        Self { raw }
    }

    fn into_raw(self) -> Self::Raw {
        self.raw
    }

    fn as_raw(&self) -> &Self::Raw {
        &self.raw
    }

    fn as_raw_mut(&mut self) -> &mut Self::Raw {
        &mut self.raw
    }
}

impl ByteString {
    pub fn new(s: &str) -> Self {
        let raw = unsafe {
            let mut raw = ffi::UA_ByteString::default();
            let s = ffi::UA_ByteString {
                length: s.len(),
                data: s.as_ptr() as *mut u8,
            }; 
            ffi::UA_String_append(&mut raw, s);
            raw
        };
        Self { raw }
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: self.raw.data is always valid and properly initialized
        unsafe {
            std::slice::from_raw_parts(self.raw.data, self.raw.length)
        }
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        // SAFETY: self.raw.data is always valid and properly initialized
        unsafe {
            std::slice::from_raw_parts_mut(self.raw.data, self.raw.length)
        }
    }
}

impl fmt::Debug for ByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.as_slice();
        s.fmt(f)
    }
}

impl serde::Serialize for ByteString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.as_slice();
        s.serialize(serializer)
    }
}
