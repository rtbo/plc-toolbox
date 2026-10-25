use crate::encoding::binary;
use crate::ua;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataValue {
    pub value: Option<ua::Variant>,
    pub status_code: Option<ua::StatusCode>,
    pub source_timestamp: Option<ua::DateTime>,
    pub server_timestamp: Option<ua::DateTime>,
}

impl binary::Encode for DataValue {
    fn encoded_len(&self) -> u32 {
        let mut len = 1; // flags
        if let Some(value) = &self.value {
            len += value.encoded_len();
        }
        if let Some(status_code) = &self.status_code {
            len += status_code.encoded_len();
        }
        if let Some(ts) = &self.source_timestamp {
            len += ts.encoded_len();
            let kps = get_kilo_picoseconds(ts);
            if kps != 0 {
                len += 2; // kilo-picoseconds
            }
        }
        if let Some(ts) = &self.server_timestamp {
            len += ts.encoded_len();
            let kps = get_kilo_picoseconds(ts);
            if kps != 0 {
                len += 2; // kilo-picoseconds
            }
        }
        len
    }
    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        let mut flags = 0;
        if self.value.is_some() {
            flags |= 0x01;
        }
        if self.status_code.is_some() {
            flags |= 0x02;
        }
        // do not compute the picosconds twice
        let src_ts = if let Some(ts) = &self.source_timestamp {
            flags |= 0x04;
            let kps = get_kilo_picoseconds(ts);
            if kps != 0 {
                flags |= 0x10; // kilo-picoseconds
            }
            Some((ts, kps))
        } else {
            None
        };
        let srv_ts = if let Some(ts) = &self.server_timestamp {
            flags |= 0x08;
            let kps = get_kilo_picoseconds(ts);
            if kps != 0 {
                flags |= 0x20; // kilo-picoseconds
            }
            Some((ts, kps))
        } else {
            None
        };
        buf.put_u8(flags);
        if let Some(value) = &self.value {
            value.encode(buf);
        }
        if let Some(status_code) = &self.status_code {
            status_code.encode(buf);
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
    fn decode<B: binary::Buf>(
        buf: &mut B,
        ctx: &binary::DecodingCtx,
    ) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let flags = buf.get_u8();
        let value = if flags & 0x01 != 0 {
            Some(ua::Variant::decode(buf, ctx)?)
        } else {
            None
        };
        let status_code = if flags & 0x02 != 0 {
            Some(ua::StatusCode::decode(buf, ctx)?)
        } else {
            None
        };
        let source_timestamp = if flags & 0x04 != 0 {
            let mut ts = ua::DateTime::decode(buf, ctx)?;
            if flags & 0x10 != 0 {
                let kps = u16::decode(buf, ctx)?;
                ts = add_kilo_picoseconds(ts, kps)
            }
            Some(ts)
        } else {
            None
        };
        let server_timestamp = if flags & 0x08 != 0 {
            let mut ts = ua::DateTime::decode(buf, ctx)?;
            if flags & 0x20 != 0 {
                let kps = u16::decode(buf, ctx)?;
                ts = add_kilo_picoseconds(ts, kps)
            }
            Some(ts)
        } else {
            None
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
