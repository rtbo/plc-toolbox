pub use bytes::{Buf, BufMut};
use thiserror::Error;

pub trait Encode {
    fn encoded_len(&self) -> u32;
    fn encode<B: BufMut>(&self, buf: &mut B);

    /// Encoded length of a slice body (without the length prefix).
    /// Types implementing `ReprEncoded` override this with a constant-time computation.
    fn slice_encoded_len(items: &[Self]) -> u32
    where
        Self: Sized,
    {
        items.iter().map(Encode::encoded_len).sum()
    }

    /// Encodes a slice body (without the length prefix).
    /// Types implementing `ReprEncoded` override this with a raw byte copy.
    fn encode_slice<B: BufMut>(items: &[Self], buf: &mut B)
    where
        Self: Sized,
    {
        for item in items {
            item.encode(buf);
        }
    }
}

#[derive(Debug, Clone, Error)]
pub enum DecodingError {
    #[error("Not enough data to decode the value")]
    NotEnoughData,

    #[error("The message type is invalid")]
    BadMessageType,

    #[error("Not UTF-8 data")]
    NotUtf8(#[from] std::string::FromUtf8Error),

    #[error("Invalid data on the stream")]
    InvalidData,

    #[error("Invalid length prefix")]
    InvalidLengthPrefix,

    #[error("Unknown data type ID")]
    DataTypeIdUnknown,
}

#[derive(Debug, Default)]
pub struct DecodingCtx {
    ns_type_lookups: Vec<Option<ua::extension_object::BinaryDecodingFn>>,
}

impl DecodingCtx {
    pub fn new() -> Self {
        Self {
            ns_type_lookups: Vec::new(),
        }
    }

    pub fn register_ns(
        &mut self,
        ns_index: u16,
        lookup_fn: ua::extension_object::BinaryDecodingFn,
    ) {
        if ns_index as usize >= self.ns_type_lookups.len() {
            self.ns_type_lookups.resize(ns_index as usize + 1, None);
        }
        self.ns_type_lookups[ns_index as usize] = Some(lookup_fn);
    }

    pub fn get_decoding_fn(
        &self,
        ns_index: u16,
    ) -> Option<&ua::extension_object::BinaryDecodingFn> {
        self.ns_type_lookups
            .get(ns_index as usize)
            .and_then(|opt| opt.as_ref())
    }
}

pub trait Decode: Sized {
    fn decode<B: Buf>(buf: &mut B, ctx: &DecodingCtx) -> Result<Self, DecodingError>;

    /// Decodes `len` consecutive items (without the length prefix).
    /// Types implementing `ReprEncoded` override this with a raw byte copy.
    fn decode_vec<B: Buf>(
        buf: &mut B,
        len: usize,
        ctx: &DecodingCtx,
    ) -> Result<Vec<Self>, DecodingError> {
        // Do not trust `len` for the allocation: it comes from the wire.
        // This assumes a one byte length size, which is often incorrect.
        // It is only a preflight check though.
        if len > buf.remaining() {
            return Err(DecodingError::InvalidLengthPrefix);
        }
        let mut items = Vec::new();
        items
            .try_reserve(len)
            .map_err(|_| DecodingError::InvalidLengthPrefix)?;
        for _ in 0..len {
            items.push(Self::decode(buf, ctx)?);
        }
        Ok(items)
    }
}

/// Marker trait for types that are encoded by their own binary representation.
/// This is used to optimize the encoding of arrays of such types,
/// as they can be copied directly from the source buffer to the destination buffer without decoding and re-encoding each element.
///
/// # Safety
/// The type must have no padding and its in-memory layout must be identical to its
/// little-endian wire encoding (so implementations must be limited to little-endian targets).
/// Zeroed memory must also be a valid value, and every bit pattern must be valid.
pub(crate) unsafe trait ReprEncoded: Sized {}

pub(crate) fn repr_decode_vec<T: ReprEncoded, B: Buf>(
    buf: &mut B,
    len: usize,
) -> Result<Vec<T>, DecodingError> {
    let bytes = len
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(DecodingError::InvalidData)?;
    check_remaining(buf, bytes)?;
    let mut items: Vec<T> = Vec::with_capacity(len);
    // SAFETY: `ReprEncoded` guarantees zeroed memory is a valid `T`, capacity is `len`,
    // and the bytes view covers exactly the initialized `len` items.
    unsafe {
        std::ptr::write_bytes(items.as_mut_ptr(), 0, len);
        items.set_len(len);
        let dst = std::slice::from_raw_parts_mut(items.as_mut_ptr().cast::<u8>(), bytes);
        buf.copy_to_slice(dst);
    }
    Ok(items)
}

pub(crate) fn repr_slice_bytes<T: ReprEncoded>(items: &[T]) -> &[u8] {
    // SAFETY: `ReprEncoded` guarantees the layout matches the wire format and has no padding.
    unsafe { std::slice::from_raw_parts(items.as_ptr().cast::<u8>(), std::mem::size_of_val(items)) }
}

/// Overrides the slice methods of `Encode` with a raw byte copy.
/// Must be used inside the `Encode` impl of a type implementing `ReprEncoded`.
macro_rules! repr_slice_methods {
    () => {
        #[cfg(target_endian = "little")]
        fn slice_encoded_len(items: &[Self]) -> u32 {
            std::mem::size_of_val(items) as u32
        }

        #[cfg(target_endian = "little")]
        fn encode_slice<B: BufMut>(items: &[Self], buf: &mut B) {
            buf.put_slice($crate::encoding::binary::repr_slice_bytes(items));
        }
    };
}
pub(crate) use repr_slice_methods;

pub(crate) fn check_remaining<B: Buf>(buf: &mut B, required: usize) -> Result<(), DecodingError> {
    if buf.remaining() < required {
        return Err(DecodingError::NotEnoughData);
    }
    Ok(())
}

/// Overrides the slice methods of `Decode` with a raw byte copy.
/// Must be used inside the `Decode` impl of a type implementing `ReprEncoded`.
macro_rules! repr_decode_methods {
    () => {
        #[cfg(target_endian = "little")]
        fn decode_vec<B: Buf>(
            buf: &mut B,
            len: usize,
            _ctx: &$crate::encoding::binary::DecodingCtx,
        ) -> Result<Vec<Self>, $crate::encoding::binary::DecodingError> {
            $crate::encoding::binary::repr_decode_vec(buf, len)
        }
    };
}
pub(crate) use repr_decode_methods;

use crate::ua;

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::*;
    use crate::ua;

    #[derive(Debug, Clone, Copy, PartialEq, ua_macros::EncodeBinary, ua_macros::DecodeBinary)]
    #[repr(u8)]
    enum SmallCode {
        Value = 0xab,
    }

    #[derive(Debug, Clone, Copy, PartialEq, ua_macros::EncodeBinary, ua_macros::DecodeBinary)]
    #[repr(u16)]
    enum MediumCode {
        Value = 0xcdef,
    }

    fn assert_roundtrip<T>(value: &T)
    where
        T: Encode + Decode + PartialEq + std::fmt::Debug,
    {
        let mut encoded = BytesMut::new();
        value.encode(&mut encoded);
        assert_eq!(encoded.len(), value.encoded_len() as usize);

        let mut remaining = encoded.clone();
        let decoded =
            T::decode(&mut remaining, &Default::default()).expect("encoded value should decode");
        assert!(remaining.is_empty());
        assert_eq!(&decoded, value);

        let mut reencoded = BytesMut::new();
        decoded.encode(&mut reencoded);
        assert_eq!(reencoded, encoded);
    }

    #[test]
    fn repr_enums_encode_their_discriminants() {
        let mut buf = BytesMut::new();

        let small = SmallCode::Value;
        assert_eq!(small.encoded_len(), 1);
        small.encode(&mut buf);
        assert_eq!(&buf[..], &[0xab]);
        assert!(matches!(
            SmallCode::decode(&mut buf, &Default::default()),
            Ok(SmallCode::Value)
        ));

        buf.clear();
        let medium = MediumCode::Value;
        assert_eq!(medium.encoded_len(), 2);
        medium.encode(&mut buf);
        assert_eq!(&buf[..], &[0xef, 0xcd]);
        assert!(matches!(
            MediumCode::decode(&mut buf, &Default::default()),
            Ok(MediumCode::Value)
        ));

        buf.clear();
        let large = ua::StatusCode::BadTcpServerTooBusy;
        assert_eq!(large.encoded_len(), 4);
        large.encode(&mut buf);
        assert_eq!(&buf[..], &0x807d0000u32.to_le_bytes());
    }

    #[test]
    fn repr_enums_roundtrip() {
        assert_roundtrip(&SmallCode::Value);
        assert_roundtrip(&MediumCode::Value);
        assert_roundtrip(&ua::StatusCode::BadTcpServerTooBusy);
    }

    #[test]
    fn repr_enum_decodes_through_try_from() {
        let mut buf = BytesMut::from(&0x807d0000u32.to_le_bytes()[..]);

        assert!(matches!(
            ua::StatusCode::decode(&mut buf, &Default::default()),
            Ok(ua::StatusCode::BadTcpServerTooBusy)
        ));

        let mut buf = BytesMut::from(&0x12345678u32.to_le_bytes()[..]);
        assert!(matches!(
            ua::StatusCode::decode(&mut buf, &Default::default()),
            Err(DecodingError::InvalidData)
        ));
    }
}
