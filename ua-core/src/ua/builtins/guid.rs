use std::fmt;

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

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7]
        )
    }
}

impl std::str::FromStr for Guid {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');
        let mut next = |len: usize, err: &'static str| {
            parts
                .next()
                .filter(|p| p.len() == len && p.bytes().all(|b| b.is_ascii_hexdigit()))
                .ok_or(err)
        };

        let p1 = next(8, "Invalid data1")?;
        let p2 = next(4, "Invalid data2")?;
        let p3 = next(4, "Invalid data3")?;
        let p4 = next(4, "Invalid data4")?;
        let p5 = next(12, "Invalid data4")?;
        if parts.next().is_some() {
            return Err("Invalid GUID format");
        }

        let data1 = u32::from_str_radix(p1, 16).map_err(|_| "Invalid data1")?;
        let data2 = u16::from_str_radix(p2, 16).map_err(|_| "Invalid data2")?;
        let data3 = u16::from_str_radix(p3, 16).map_err(|_| "Invalid data3")?;

        let mut data4 = [0u8; 8];
        for (i, byte) in data4.iter_mut().enumerate() {
            let p = if i < 2 {
                &p4[i * 2..]
            } else {
                &p5[(i - 2) * 2..]
            };
            *byte = u8::from_str_radix(&p[..2], 16).map_err(|_| "Invalid data4")?;
        }

        Ok(Guid {
            data1,
            data2,
            data3,
            data4,
        })
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

#[cfg(test)]
mod tests {
    use super::Guid;

    #[test]
    fn display_matches_opc_ua_guid_format() {
        let guid = Guid::new(
            0xC496_578A,
            0x0DFE,
            0x4B8F,
            [0x87, 0x0A, 0x74, 0x52, 0x38, 0xC6, 0xAE, 0xAE],
        );

        assert_eq!(guid.to_string(), "C496578A-0DFE-4B8F-870A-745238C6AEAE");
    }

    #[test]
    fn from_str_parses_valid_guid() {
        let guid_str = "C496578A-0DFE-4B8F-870A-745238C6AEAE";
        let guid = guid_str.parse::<Guid>().unwrap();
        assert_eq!(guid.to_string(), guid_str);
        assert_eq!(
            guid,
            Guid::new(
                0xC496_578A,
                0x0DFE,
                0x4B8F,
                [0x87, 0x0A, 0x74, 0x52, 0x38, 0xC6, 0xAE, 0xAE]
            )
        );
    }
}
