use std::fmt;

use crate::ffi;

#[repr(transparent)]
pub struct String {
    raw: ffi::UA_String,
}

unsafe impl crate::DataType for String {
    type Raw = ffi::UA_String;
    const UA_TYPE_IDX: usize = ffi::UA_TYPES_STRING as usize;
    const NAME: &'static str = "String";

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

impl String {
    pub fn new(s: &str) -> Self {
        let raw = unsafe {
            let mut raw = ffi::UA_String::default();
            let s = ffi::UA_String {
                length: s.len(),
                data: s.as_ptr() as *mut u8,
            }; 
            ffi::UA_String_append(&mut raw, s);
            raw
        };
        Self { raw }
    }

    pub fn as_str(&self) -> &str {
        // SAFETY: self.raw.data is always valid and properly initialized
        let slice = unsafe {
            std::slice::from_raw_parts(self.raw.data, self.raw.length)
        };
        std::str::from_utf8(slice).unwrap()
    }
}

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.as_str();
        f.write_str(s)
    }
}

impl serde::Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.as_str();
        serializer.serialize_str(s)
    }
}

impl<'de> serde::de::Deserialize<'de> for String {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = String;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a UTF-8 encoded string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(String::new(v))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
