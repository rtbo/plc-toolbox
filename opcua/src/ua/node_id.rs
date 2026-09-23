use std::{fmt, str::FromStr};

use crate::{ffi, status_code, ua};
use super::NodeId;

impl NodeId {
    fn print(&self) -> status_code::Result<ua::String> {
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
        let s: ua::String = self.print().map_err(|_| serde::ser::Error::custom("Failed to print NodeId"))?;
        s.serialize(serializer)
    }
}
