use std::{fmt, str::FromStr};

use super::NodeId;
use crate::{ffi, status_code, ua};

impl NodeId {
    pub const fn numeric(ns_index: u16, numeric: u32) -> Self {
        let numeric_storage = if cfg!(target_endian = "big") {
            (numeric as u64) << 32
        } else {
            numeric as u64
        };

        Self {
            raw: ffi::UA_NodeId {
                namespaceIndex: ns_index,
                identifierType: ffi::UA_NodeIdType::UA_NODEIDTYPE_NUMERIC,
                identifier: ffi::UA_NodeId__bindgen_ty_1 {
                    numeric: ffi::__BindgenUnionField::new(),
                    string: ffi::__BindgenUnionField::new(),
                    guid: ffi::__BindgenUnionField::new(),
                    byteString: ffi::__BindgenUnionField::new(),
                    bindgen_union_field: [numeric_storage, 0],
                },
            },
        }
    }
    // pub fn numeric(ns_index: u16, numeric: u32) -> Self {
    //     // let mut raw = ffi::UA_NodeId::default();
    //     // raw.namespaceIndex = ns_index;
    //     // raw.identifierType = ffi::UA_NodeIdType::UA_NODEIDTYPE_NUMERIC;
    //     // raw.identifier.numeric = numeric.into();

    //     let inner = unsafe { ffi::UA_NODEID_NUMERIC(ns_index, numeric) };
    //     Self { raw: inner }
    // }

    pub fn print(&self) -> status_code::Result<ua::String> {
        use crate::DataType;

        let mut buffer = ffi::UA_String::default();
        let code = unsafe { ffi::UA_NodeId_print(&self.raw, &mut buffer) };
        status_code::check(code)?;

        // SAFETY: buffer is properly initialized and contains a valid UA_String.
        Ok(unsafe { ua::String::from_raw(buffer) })
    }
}

impl FromStr for NodeId {
    type Err = status_code::StatusCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_ffi = ffi::UA_String {
            length: s.len(),
            data: s.as_ptr() as *mut u8,
        };

        let mut raw = ffi::UA_NodeId::default();

        // Safety: s_ffi is a valid UA_String, and UA_NodeId_parse doesn't keep a reference to it after returning.
        let code = unsafe { ffi::UA_NodeId_parse(&mut raw, s_ffi) };
        status_code::check(code)?;

        Ok(Self { raw })
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: ua::String = self.print().map_err(|_| fmt::Error)?;
        f.write_str(s.as_str())
    }
}

impl serde::Serialize for NodeId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s: ua::String = self
            .print()
            .map_err(|_| serde::ser::Error::custom("Failed to print NodeId"))?;
        s.serialize(serializer)
    }
}

impl<'de> serde::de::Deserialize<'de> for NodeId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = NodeId;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a NodeId string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                NodeId::from_str(v).map_err(|_| serde::de::Error::custom("Failed to parse NodeId"))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}