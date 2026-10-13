use crate::msg::binary;

impl<T> binary::Encode for &[T]
where
    T: binary::Encode,
{
    fn encoded_len(&self) -> u32 {
        4 + T::slice_encoded_len(self) // length prefix + body
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        buf.put_i32_le(self.len() as i32);
        T::encode_slice(self, buf);
    }
}

impl<T> binary::Encode for Option<&[T]>
where
    T: binary::Encode,
{
    fn encoded_len(&self) -> u32 {
        match self {
            Some(s) => s.encoded_len(),
            None => 4,
        }
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        match self {
            Some(s) => {
                s.encode(buf);
            }
            None => {
                buf.put_i32_le(-1);
            }
        }
    }
}

impl<T> binary::Encode for Vec<T>
where
    T: binary::Encode,
{
    fn encoded_len(&self) -> u32 {
        self.as_slice().encoded_len()
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        self.as_slice().encode(buf);
    }
}

impl<T> binary::Encode for Option<Vec<T>>
where
    T: binary::Encode,
{
    fn encoded_len(&self) -> u32 {
        match self {
            Some(s) => s.encoded_len(),
            None => 4,
        }
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        match self {
            Some(s) => {
                s.encode(buf);
            }
            None => {
                buf.put_i32_le(-1);
            }
        }
    }
}

impl<T> binary::Decode for Vec<T>
where
    T: binary::Decode,
{
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let len = buf.get_i32_le();
        if len == -1 {
            return Ok(Vec::new());
        }
        if len < 0 {
            return Err(binary::DecodingError::InvalidLengthPrefix);
        }
        T::decode_vec(buf, len as usize)
    }
}

impl<T> binary::Decode for Option<Vec<T>>
where
    T: binary::Decode,
{
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 4)?;
        let len = buf.get_i32_le();
        if len == -1 {
            return Ok(None);
        }
        if len < 0 {
            return Err(binary::DecodingError::InvalidLengthPrefix);
        }
        let vec = T::decode_vec(buf, len as usize)?;
        Ok(Some(vec))
    }
}
