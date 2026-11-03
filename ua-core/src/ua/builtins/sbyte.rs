use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

pub type SByte = i8;

impl binary::Encode for SByte {
    binary::repr_slice_methods!();

    fn encoded_len(&self) -> u32 {
        1
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        buf.put_i8(*self)
    }
}

impl binary::Decode for SByte {
    binary::repr_decode_methods!();

    fn decode<B: Buf>(buf: &mut B, _ctx: &binary::DecodingCtx) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 1)?;
        let val = buf.get_i8();
        Ok(val)
    }
}
