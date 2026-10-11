pub use bytes::{Buf, BufMut};
use thiserror::Error;

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
}

pub trait EncodeBinary {
    fn encoded_len(&self) -> u32;
    fn encode<B: BufMut>(&self, buf: &mut B);
}

pub trait DecodeBinary: Sized {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError>;
}

pub(crate) fn check_remaining<B: Buf>(buf: &mut B, required: usize) -> Result<(), DecodingError> {
    if buf.remaining() < required {
        return Err(DecodingError::NotEnoughData);
    }
    Ok(())
}

impl EncodeBinary for u8 {
    fn encoded_len(&self) -> u32 {
        1
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self);
    }
}

impl DecodeBinary for u8 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        check_remaining(buf, 1)?;
        Ok(buf.get_u8())
    }
}

impl<const N: usize> EncodeBinary for [u8; N] {
    fn encoded_len(&self) -> u32 {
        N as u32
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(self);
    }
}

impl<const N: usize> DecodeBinary for [u8; N] {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        check_remaining(buf, N)?;
        let mut arr = [0u8; N];
        buf.copy_to_slice(&mut arr);
        Ok(arr)
    }
}

impl EncodeBinary for u16 {
    fn encoded_len(&self) -> u32 {
        2
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u16_le(*self);
    }
}

impl DecodeBinary for u16 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        check_remaining(buf, 2)?;
        Ok(buf.get_u16_le())
    }
}

impl EncodeBinary for u32 {
    fn encoded_len(&self) -> u32 {
        4
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(*self);
    }
}

impl DecodeBinary for u32 {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        check_remaining(buf, 4)?;
        Ok(buf.get_u32_le())
    }
}

impl EncodeBinary for &str {
    fn encoded_len(&self) -> u32 {
        4 + self.len() as u32
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(self.len() as u32);
        buf.put_slice(self.as_bytes());
    }
}

impl EncodeBinary for String {
    fn encoded_len(&self) -> u32 {
        4 + self.len() as u32
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(self.len() as u32);
        buf.put_slice(self.as_bytes());
    }
}

impl DecodeBinary for String {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, DecodingError> {
        let len = u32::decode(buf)? as usize;
        check_remaining(buf, len)?;
        let mut bytes = vec![0u8; len];
        buf.copy_to_slice(&mut bytes);
        Ok(String::from_utf8(bytes)?)
    }
}
