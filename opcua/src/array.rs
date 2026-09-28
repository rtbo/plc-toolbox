use std::ops::{Index, IndexMut};

use crate::{StatusCode, ffi};

#[derive(Debug)]
pub struct Array<T: crate::DataType> {
    phantom: std::marker::PhantomData<T>,
    len: usize,
    raw: *mut std::ffi::c_void,
}

impl<T: crate::DataType> Array<T> {
    /// Creates a new array with the specified size.
    /// Each member is initialized with the default value of the type `T`.
    /// Returns `None` if the array could not be created.
    pub fn new(len: usize) -> Option<Self> {
        let raw = unsafe { ffi::UA_Array_new(len, T::data_type()) };
        if raw.is_null() {
            return None;
        }
        Some(Self {
            phantom: std::marker::PhantomData,
            len,
            raw,
        })
    }

    /// Creates an array from raw parts.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `raw` points to a valid array of `T::Raw` elements
    /// with the specified `len`. The array takes ownership of the memory and will
    /// free it when dropped.
    pub unsafe fn from_raw_parts(raw: *mut T::Raw, len: usize) -> Self {
        Self {
            phantom: std::marker::PhantomData,
            len,
            raw: raw as _,
        }
    }

    /// Returns `true` if the array contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of elements in the array.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns a raw pointer to the underlying array data.
    pub fn as_ptr(&self) -> *const T::Raw {
        self.raw.cast()
    }

    /// Returns a mutable raw pointer to the underlying array data.
    pub fn as_mut_ptr(&self) -> *mut T::Raw {
        self.raw.cast()
    }

    /// Returns a slice to the array elements.
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.raw.cast::<T>(), self.len) }
        }
    }

    /// Returns a mutable slice to the array elements.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.raw.cast::<T>(), self.len) }
        }
    }
}

impl<T: crate::DataType> Index<usize> for Array<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len, "Index out of bounds");
        unsafe { &*(self.raw.cast::<T>().add(index)) }
    }
}

impl<T: crate::DataType> IndexMut<usize> for Array<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < self.len, "Index out of bounds");
        unsafe { &mut *(self.raw.cast::<T>().add(index)) }
    }
}

/// Create an array with a single element
impl<T: crate::DataType> From<T> for Array<T> {
    fn from(value: T) -> Self {
        let mut arr = Self::new(1).expect("Cannot create array");
        arr[0] = value;
        arr
    }
}

// impl<T: crate::DataType, I> From<I> for Array<T>
// where
//     I: ExactSizeIterator<Item = T>,
// {
//     fn from(values: I) -> Self {
//         let mut values = values;
//         let len = values.len();
//         let mut array = Self::new(len).expect("Failed to create array");
//         for i in 0..len {
//             array[i] = values.next().expect("Iterator length mismatch");
//         }
//         array
//     }
// }

impl<T: crate::DataType> Array<T> {
    /// Consumes the array and returns an iterator that owns each element.
    pub fn into_iter(self) -> ArrayIter<T> {
        ArrayIter {
            array: self,
            index: 0,
        }
    }

    /// Returns an iterator over the array elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns a mutable iterator over the array elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}

/// An iterator that owns an array and yields its elements by value.
pub struct ArrayIter<T: crate::DataType> {
    array: Array<T>,
    index: usize,
}

impl<T: crate::DataType> Iterator for ArrayIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.array.len {
            return None;
        }

        let raw = unsafe { self.array.raw.cast::<T::Raw>().add(self.index) };
        self.index += 1;
        // SAFETY: This slot contains an initialized raw T. Moving it out and
        // reinitializing the slot keeps the owning Array valid for its Drop.
        let value = unsafe {
            let value = raw.read();
            ffi::UA_init(raw.cast(), T::data_type());
            T::from_raw(value)
        };
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.array.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T: crate::DataType> ExactSizeIterator for ArrayIter<T> {}

impl<T: crate::DataType> std::iter::FusedIterator for ArrayIter<T> {}

impl<T: crate::DataType> IntoIterator for Array<T> {
    type Item = T;
    type IntoIter = ArrayIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Array::into_iter(self)
    }
}

impl<'a, T: crate::DataType> IntoIterator for &'a Array<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: crate::DataType> IntoIterator for &'a mut Array<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: crate::DataType> Default for Array<T> {
    fn default() -> Self {
        Self::new(0).unwrap()
    }
}

impl<T: crate::DataType> Clone for Array<T> {
    fn clone(&self) -> Self {
        let mut dst = std::ptr::null_mut();
        let code = unsafe { ffi::UA_Array_copy(self.raw, self.len, &mut dst as _, T::data_type()) };
        // Safety: We assume that the status code returned by `ffi::UA_Array_copy` is valid.
        let code = unsafe { StatusCode::from_raw_unchecked(code) };
        if code != StatusCode::Good {
            panic!("Failed to clone array: status code {}", code);
        }
        Self {
            phantom: std::marker::PhantomData,
            len: self.len,
            raw: dst,
        }
    }
}

impl<T: crate::DataType> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe {
            ffi::UA_Array_delete(self.raw, self.len, T::data_type());
        }
    }
}
