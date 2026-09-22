use crate::ffi;

pub struct StringRef<'a> {
    ptr: *const u8,
    len: usize,
    _marker: std::marker::PhantomData<&'a u8>,
}

impl<'a> StringRef<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            ptr: s.as_ptr(),
            len: s.len(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Converts the string reference to an FFI-compatible `UA_String`.
    ///
    /// # Safety
    ///
    /// The returned `UA_String` contains a raw pointer to the string data.
    /// The caller must ensure that the `StringRef` outlives the `UA_String`.
    pub unsafe fn as_ffi(&self) -> ffi::UA_String {
        ffi::UA_String {
            length: self.len,
            data: self.ptr as *mut u8,
        }
    }
}
