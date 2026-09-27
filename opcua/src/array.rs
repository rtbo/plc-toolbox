
#[derive(Debug, Clone)]
pub struct Array<T: crate::DataType> {
    phantom: std::marker::PhantomData<T>,
    len: usize,
    raw: *mut std::ffi::c_void,
}

