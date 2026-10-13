use crate::DataType;
use crate::msg::binary;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Guid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl Guid {
    pub fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        Self {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

#[cfg(feature = "uuid")]
impl From<uuid::Uuid> for Guid {
    fn from(uuid: uuid::Uuid) -> Self {
        let (data1, data2, data3, data4) = uuid.as_fields();
        Guid {
            data1,
            data2,
            data3,
            data4: *data4,
        }
    }
}

#[cfg(feature = "uuid")]
impl From<Guid> for uuid::Uuid {
    fn from(guid: Guid) -> Self {
        uuid::Uuid::from_fields(guid.data1, guid.data2, guid.data3, &guid.data4)
    }
}

impl DataType for Guid {
    fn type_id() -> crate::ua::NodeId {
        crate::ua::NodeId::Numeric(0, 14)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        Guid {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8],
        }
    }
}

impl binary::Encode for Guid {
    fn encoded_len(&self) -> u32 {
        16
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        buf.put_u32_le(self.data1);
        buf.put_u16_le(self.data2);
        buf.put_u16_le(self.data3);
        buf.put_slice(&self.data4);
    }
}

impl binary::Decode for Guid {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 16)?;
        let data1 = buf.get_u32_le();
        let data2 = buf.get_u16_le();
        let data3 = buf.get_u16_le();
        let mut data4 = [0u8; 8];
        buf.copy_to_slice(&mut data4);
        Ok(Guid {
            data1,
            data2,
            data3,
            data4,
        })
    }
}
