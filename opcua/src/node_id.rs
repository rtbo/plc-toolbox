use std::str::FromStr;

use crate::{ffi, status_code};

pub struct NodeId {
    ffi: ffi::UA_NodeId,
}

impl NodeId {
    /// Returns the underlying FFI representation of the NodeId.
    ///
    /// # Safety
    ///
    /// The returned `ffi::UA_NodeId` hold references to the internal data of the `NodeId`.
    /// The caller must ensure that the returned `ffi::UA_NodeId` is not used after the `NodeId` is dropped.
    pub unsafe fn as_ffi(&self) -> ffi::UA_NodeId {
        unsafe { std::ptr::read(&self.ffi) }
    }
}

impl FromStr for NodeId {
    type Err = status_code::StatusCode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_ffi = ffi::UA_String {
            length: s.len(),
            data: s.as_ptr() as *mut u8,
        };

        let mut node_id_ffi: ffi::UA_NodeId = unsafe { std::mem::zeroed() };

        // Safety: s_ffi is a valid UA_String, and UA_NodeId_parse doesn't keep a reference to it after returning.
        status_code::check(unsafe { ffi::UA_NodeId_parse(&mut node_id_ffi, s_ffi) })?;

        Ok(Self { ffi: node_id_ffi })
    }
}

impl Drop for NodeId {
    fn drop(&mut self) {
        unsafe { ffi::UA_NodeId_clear(&mut self.ffi) };
    }
}
