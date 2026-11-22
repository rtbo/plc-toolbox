use bytes::{Buf, BufMut};

use crate::encoding::binary;
use crate::ua;

#[derive(Debug, Default)]
pub struct DataValue {
    pub value: ua::Variant,
    pub status_code: ua::StatusCode,
    pub source_timestamp: ua::DateTime,
    pub server_timestamp: ua::DateTime,
}

impl DataValue {
    fn value_is_default(&self) -> bool {
        matches!(self.value, ua::Variant::Null)
    }

    fn status_code_is_default(&self) -> bool {
        self.status_code == ua::StatusCode::Good
    }

    fn source_timestamp_is_default(&self) -> bool {
        self.source_timestamp == ua::DateTime::MIN
    }

    fn server_timestamp_is_default(&self) -> bool {
        self.server_timestamp == ua::DateTime::MIN
    }
}

impl binary::Encode for DataValue {
    fn encoded_len(&self) -> u32 {
        let mut len = 1; // flags
        if !self.value_is_default() {
            len += self.value.encoded_len();
        }
        if !self.status_code_is_default() {
            len += self.status_code.encoded_len();
        }
        if !self.source_timestamp_is_default() {
            len += self.source_timestamp.encoded_len();
            let kps = get_kilo_picoseconds(&self.source_timestamp);
            if kps != 0 {
                len += 2; // kilo-picoseconds
            }
        }
        if !self.server_timestamp_is_default() {
            len += self.server_timestamp.encoded_len();
            let kps = get_kilo_picoseconds(&self.server_timestamp);
            if kps != 0 {
                len += 2; // kilo-picoseconds
            }
        }
        len
    }
    fn encode(&self, buf: &mut binary::EncodeBuf<'_>) {
        let mut flags = 0;
        if !self.value_is_default() {
            flags |= 0x01;
        }
        if !self.status_code_is_default() {
            flags |= 0x02;
        }
        // do not compute the picosconds twice
        let src_ts = if !self.source_timestamp_is_default() {
            flags |= 0x04;
            let kps = get_kilo_picoseconds(&self.source_timestamp);
            if kps != 0 {
                flags |= 0x10; // kilo-picoseconds
            }
            Some((&self.source_timestamp, kps))
        } else {
            None
        };
        let srv_ts = if !self.server_timestamp_is_default() {
            flags |= 0x08;
            let kps = get_kilo_picoseconds(&self.server_timestamp);
            if kps != 0 {
                flags |= 0x20; // kilo-picoseconds
            }
            Some((&self.server_timestamp, kps))
        } else {
            None
        };
        buf.put_u8(flags);
        if !self.value_is_default() {
            self.value.encode(buf);
        }
        if !self.status_code_is_default() {
            self.status_code.encode(buf);
        }
        if let Some((ts, kps)) = src_ts {
            ts.encode(buf);
            if kps != 0 {
                buf.put_u16(kps);
            }
        }
        if let Some((ts, kps)) = srv_ts {
            ts.encode(buf);
            if kps != 0 {
                buf.put_u16(kps);
            }
        }
    }
}

impl binary::Decode for DataValue {
    fn decode(
        buf: &mut crate::encoding::binary::DecodeBuf,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        binary::check_remaining(buf, 1)?;
        let flags = buf.get_u8();
        let value = if flags & 0x01 != 0 {
            ua::Variant::decode(buf, ctx)?
        } else {
            ua::Variant::default()
        };
        let status_code = if flags & 0x02 != 0 {
            ua::StatusCode::decode(buf, ctx)?
        } else {
            ua::StatusCode::default()
        };
        let source_timestamp = if flags & 0x04 != 0 {
            let mut ts = ua::DateTime::decode(buf, ctx)?;
            if flags & 0x10 != 0 {
                let kps = u16::decode(buf, ctx)?;
                ts = add_kilo_picoseconds(ts, kps)
            }
            ts
        } else {
            ua::DateTime::default()
        };
        let server_timestamp = if flags & 0x08 != 0 {
            let mut ts = ua::DateTime::decode(buf, ctx)?;
            if flags & 0x20 != 0 {
                let kps = u16::decode(buf, ctx)?;
                ts = add_kilo_picoseconds(ts, kps)
            }
            ts
        } else {
            ua::DateTime::default()
        };

        Ok(DataValue {
            value,
            status_code,
            source_timestamp,
            server_timestamp,
        })
    }
}

// Get the picoseconds part of a DateTime (encoded as hecto-nanoseconds).
// In OPC/UA, picosecond precision is tens of picoseconds (kilo-ps)
// DateTime has a 100ns resolution, so there is never a sub-hecto-nanosecond part.
fn get_kilo_picoseconds(_dt: &ua::DateTime) -> u16 {
    0
}

// Sub-hecto-nanosecond precision cannot be represented and is dropped.
fn add_kilo_picoseconds(dt: ua::DateTime, _kps: u16) -> ua::DateTime {
    dt
}
