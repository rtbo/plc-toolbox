use std::mem;

use crate::{ffi, status_code};

pub unsafe trait DataType {
    type Raw;
    const UA_TYPE_IDX: usize;
    const NAME: &'static str;

    /// Constructs an instance of the type from its raw FFI representation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `raw` is a valid instance of the underlying FFI type
    /// and that the raw value is not used elsewhere.
    unsafe fn from_raw(raw: Self::Raw) -> Self;

    fn into_raw(self) -> Self::Raw;

    fn as_raw(&self) -> &Self::Raw;

    fn as_raw_mut(&mut self) -> &mut Self::Raw;

    fn as_ptr(&self) -> *const Self::Raw {
        self.as_raw() as *const Self::Raw
    }

    fn as_mut_ptr(&mut self) -> *mut Self::Raw {
        self.as_raw_mut() as *mut Self::Raw
    }

    fn clone_raw(&self) -> Self::Raw {
        let mut dst = mem::MaybeUninit::<Self::Raw>::uninit();
        // SAFETY: self.as_raw() is always valid and properly initialized, 
        // corresponding to the right FFI type.
        let code = unsafe {
            ffi::UA_copy(
                self.as_ptr() as _,
                dst.as_mut_ptr() as *mut _,
                &ffi::UA_TYPES[Self::UA_TYPE_IDX],
            )
        };
        status_code::expect_good(code);
        // SAFETY: dst has been properly initialized by ffi::UA_copy.
        unsafe { dst.assume_init() }
    }
}

// #[macro_export]
// macro_rules! data_type_struct {
//     ($ty:ident, $raw_ty:ty, $ua_ty_idx:expr) => {
//         pub struct $ty {
//             raw: $raw_ty,
//         }

//         unsafe impl $crate::data_type::DataType for $ty {
//             type Raw = $raw_ty;
//             const UA_TYPE_IDX: usize = $ua_ty_idx as usize;

//             unsafe fn from_raw(raw: Self::Raw) -> Self {
//                 Self { raw }
//             }

//             fn into_raw(self) -> Self::Raw {
//                 let t = unsafe { std::ptr::read(&self.raw) };
//                 std::mem::forget(self);
//                 t
//             }

//             fn as_raw(&self) -> &Self::Raw {
//                 &self.raw
//             }

//             fn as_raw_mut(&mut self) -> &mut Self::Raw {
//                 &mut self.raw
//             }
//         }

//         impl Clone for $ty {
//             fn clone(&self) -> Self {
//                 use $crate::data_type::DataType;

//                 unsafe { Self::from_raw(self.clone_raw()) }
//             }
//         }

//         impl Drop for $ty {
//             fn drop(&mut self) {
//                 use $crate::ffi;
//                 use $crate::data_type::DataType;
//                 unsafe {
//                     ffi::UA_clear(
//                         self.as_mut_ptr() as *mut _,
//                         &ffi::UA_TYPES[$ty::UA_TYPE_IDX],
//                     )
//                 }
//             }
//         }
//     }
// }

// pub(crate) use data_type_struct;
