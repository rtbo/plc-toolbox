use bytes::{Buf, BufMut};

use crate::msg::binary;
use crate::{DataType, ua};

pub type DateTime = chrono::DateTime<chrono::Utc>;

impl DataType for DateTime {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 13)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        DT_MIN
    }
}

impl binary::Encode for DateTime {
    fn encoded_len(&self) -> u32 {
        8
    }

    fn encode<B: BufMut>(&self, buf: &mut B) {
        let timestamp = to_opcua(self);
        buf.put_i64_le(timestamp);
    }
}

impl binary::Decode for DateTime {
    fn decode<B: Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 8)?;
        let timestamp = buf.get_i64_le();
        Ok(from_opcua(timestamp))
    }
}

const DT_MIN: chrono::DateTime<chrono::Utc> = from_opcua(i64::MIN);

/// Convert from chrono::DateTime<chrono::Utc> to OPC/UA timestamp (hnsecs since 1601-01-01T00:00:00Z)
const fn to_opcua(dt: &chrono::DateTime<chrono::Utc>) -> i64 {
    let seconds_unix = dt.timestamp();
    let seconds_opcua = seconds_unix + (1970 - 1601) * 365 * 24 * 60 * 60;
    let subsec_nanos = dt.timestamp_subsec_nanos();
    seconds_opcua * 10_000_000 + (subsec_nanos / 100) as i64
}

/// Convert from OPC/UA timestamp (hnsecs since 1601-01-01T00:00:00Z) to chrono::DateTime<chrono::Utc>
const fn from_opcua(timestamp: i64) -> chrono::DateTime<chrono::Utc> {
    let seconds_opcua = timestamp / 10_000_000;
    let seconds_unix = seconds_opcua - (1970 - 1601) * 365 * 24 * 60 * 60;
    let subsec_nanos = timestamp.rem_euclid(10_000_000) * 100;
    chrono::DateTime::from_timestamp(seconds_unix, subsec_nanos as u32)
        .expect("OPC/UA datetime should be valid chrono datetime")
}
