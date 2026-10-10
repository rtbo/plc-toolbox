use bytes::BufMut;

pub trait EncodeBinary {
    fn encoded_len(&self) -> u32;
    fn encode<B: BufMut>(&self, buf: &mut B);
}

impl EncodeBinary for u8 {
    fn encoded_len(&self) -> u32 {
        1
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(*self);
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

impl EncodeBinary for &[u8] {
    fn encoded_len(&self) -> u32 {
        self.len() as u32
    }
    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_slice(self);
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
