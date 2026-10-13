use crate::msg::binary::{self, Encode};
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq)]
pub enum Value<T> {
    Scalar(T),
    Array(Vec<T>),
    MultiDimArray(Vec<T>, Vec<i32>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Variant {
    Null,
    Boolean(Value<bool>),
    SByte(Value<i8>),
    Byte(Value<u8>),
    Int16(Value<i16>),
    UInt16(Value<u16>),
    Int32(Value<i32>),
    UInt32(Value<u32>),
    Int64(Value<i64>),
    UInt64(Value<u64>),
    Float(Value<f32>),
    Double(Value<f64>),
    String(Value<String>),
    DateTime(Value<ua::DateTime>),
    Guid(Value<ua::Guid>),
    ByteString(Value<Vec<u8>>),
    XmlElement(Value<ua::XmlElement>),
    NodeId(Value<ua::NodeId>),
    ExpandedNodeId(Value<ua::ExpandedNodeId>),
    StatusCode(Value<ua::StatusCode>),
    QualifiedName(Value<ua::QualifiedName>),
    LocalizedText(Value<ua::LocalizedText>),
    //ExtensionObject(VariantVal<ua::ExtensionObject>),
    DataValue(Box<Value<ua::DataValue>>),
    Variant(Box<Value<Variant>>), // Nested variant
}

impl DataType for Variant {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 24)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        Variant::Null
    }
}

impl binary::Encode for Variant {
    fn encoded_len(&self) -> u32 {
        match self {
            Variant::Null => 1,
            Variant::Boolean(val) => value_encoded_len(val),
            Variant::SByte(val) => value_encoded_len(val),
            Variant::Byte(val) => value_encoded_len(val),
            Variant::Int16(val) => value_encoded_len(val),
            Variant::UInt16(val) => value_encoded_len(val),
            Variant::Int32(val) => value_encoded_len(val),
            Variant::UInt32(val) => value_encoded_len(val),
            Variant::Int64(val) => value_encoded_len(val),
            Variant::UInt64(val) => value_encoded_len(val),
            Variant::Float(val) => value_encoded_len(val),
            Variant::Double(val) => value_encoded_len(val),
            Variant::String(val) => value_encoded_len(val),
            Variant::DateTime(val) => value_encoded_len(val),
            Variant::Guid(val) => value_encoded_len(val),
            Variant::ByteString(val) => value_encoded_len(val),
            Variant::XmlElement(val) => value_encoded_len(val),
            Variant::NodeId(val) => value_encoded_len(val),
            Variant::ExpandedNodeId(val) => value_encoded_len(val),
            Variant::StatusCode(val) => value_encoded_len(val),
            Variant::QualifiedName(val) => value_encoded_len(val),
            Variant::LocalizedText(val) => value_encoded_len(val),
            Variant::DataValue(val) => value_encoded_len(val),
            Variant::Variant(val) => value_encoded_len(val),
        }
    }

    fn encode<B: bytes::BufMut>(&self, buf: &mut B) {
        match self {
            Variant::Null => {
                0u8.encode(buf);
            }
            Variant::Boolean(val) => {
                encode_value(1, val, buf);
            }
            Variant::SByte(val) => {
                encode_value(2, val, buf);
            }
            Variant::Byte(val) => {
                encode_value(3, val, buf);
            }
            Variant::Int16(val) => {
                encode_value(4, val, buf);
            }
            Variant::UInt16(val) => {
                encode_value(5, val, buf);
            }
            Variant::Int32(val) => {
                encode_value(6, val, buf);
            }
            Variant::UInt32(val) => {
                encode_value(7, val, buf);
            }
            Variant::Int64(val) => {
                encode_value(8, val, buf);
            }
            Variant::UInt64(val) => {
                encode_value(9, val, buf);
            }
            Variant::Float(val) => {
                encode_value(10, val, buf);
            }
            Variant::Double(val) => {
                encode_value(11, val, buf);
            }
            Variant::String(val) => {
                encode_value(12, val, buf);
            }
            Variant::DateTime(val) => {
                encode_value(13, val, buf);
            }
            Variant::Guid(val) => encode_value(14, val, buf),
            Variant::ByteString(val) => encode_value(15, val, buf),
            Variant::XmlElement(val) => encode_value(16, val, buf),
            Variant::NodeId(val) => encode_value(17, val, buf),
            Variant::ExpandedNodeId(val) => encode_value(18, val, buf),
            Variant::StatusCode(val) => encode_value(19, val, buf),
            Variant::QualifiedName(val) => encode_value(20, val, buf),
            Variant::LocalizedText(val) => encode_value(21, val, buf),
            Variant::DataValue(val) => encode_value(23, val, buf),
            Variant::Variant(val) => encode_value(24, val, buf),
        }
    }
}

impl binary::Decode for Variant {
    fn decode<B: bytes::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        let encoded_mask = buf.get_u8();
        let builtin_typeid = encoded_mask & 0x3F;
        let is_array = (encoded_mask & 0x40) != 0;
        let has_multidim = (encoded_mask & 0x80) != 0;
        match builtin_typeid {
            0 => Ok(Variant::Null),
            1 => Ok(Variant::Boolean(decode_value(is_array, has_multidim, buf)?)),
            2 => Ok(Variant::SByte(decode_value(is_array, has_multidim, buf)?)),
            3 => Ok(Variant::Byte(decode_value(is_array, has_multidim, buf)?)),
            4 => Ok(Variant::Int16(decode_value(is_array, has_multidim, buf)?)),
            5 => Ok(Variant::UInt16(decode_value(is_array, has_multidim, buf)?)),
            6 => Ok(Variant::Int32(decode_value(is_array, has_multidim, buf)?)),
            7 => Ok(Variant::UInt32(decode_value(is_array, has_multidim, buf)?)),
            8 => Ok(Variant::Int64(decode_value(is_array, has_multidim, buf)?)),
            9 => Ok(Variant::UInt64(decode_value(is_array, has_multidim, buf)?)),
            10 => Ok(Variant::Float(decode_value(is_array, has_multidim, buf)?)),
            11 => Ok(Variant::Double(decode_value(is_array, has_multidim, buf)?)),
            12 => Ok(Variant::String(decode_value(is_array, has_multidim, buf)?)),
            13 => Ok(Variant::DateTime(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            14 => Ok(Variant::Guid(decode_value(is_array, has_multidim, buf)?)),
            15 => Ok(Variant::ByteString(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            16 => Ok(Variant::XmlElement(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            17 => Ok(Variant::NodeId(decode_value(is_array, has_multidim, buf)?)),
            18 => Ok(Variant::ExpandedNodeId(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            19 => Ok(Variant::StatusCode(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            20 => Ok(Variant::QualifiedName(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            21 => Ok(Variant::LocalizedText(decode_value(
                is_array,
                has_multidim,
                buf,
            )?)),
            23 => Ok(Variant::Variant(Box::new(decode_value(
                is_array,
                has_multidim,
                buf,
            )?))),
            24 => Ok(Variant::Variant(Box::new(decode_value(
                is_array,
                has_multidim,
                buf,
            )?))),
            _ => Err(binary::DecodingError::InvalidData),
        }
    }
}

fn value_encoded_len<T>(value: &Value<T>) -> u32
where
    T: DataType,
{
    1 + match value {
        Value::Scalar(val) => val.encoded_len(),
        Value::Array(vals) => vals.encoded_len(),
        Value::MultiDimArray(vals, dimensions) => vals.encoded_len() + dimensions.encoded_len(),
    }
}

fn encode_value<T, B>(builtin_typeid: u8, value: &Value<T>, buf: &mut B)
where
    T: DataType,
    B: bytes::BufMut,
{
    let encoding_mask = builtin_typeid
        | match value {
            Value::Scalar(_) => 0x00,
            Value::Array(_) => 0x40,
            Value::MultiDimArray(_, _) => 0xC0,
        };
    buf.put_u8(encoding_mask);

    match value {
        Value::Scalar(val) => val.encode(buf),
        Value::Array(vals) => {
            vals.encode(buf);
        }
        Value::MultiDimArray(vals, dimensions) => {
            vals.encode(buf);
            dimensions.encode(buf);
        }
    }
}

fn decode_value<T: DataType, B: bytes::Buf>(
    is_array: bool,
    has_multidim: bool,
    buf: &mut B,
) -> Result<Value<T>, binary::DecodingError> {
    match (is_array, has_multidim) {
        (true, true) => {
            let values: Vec<T> = binary::Decode::decode(buf)?;
            let dimensions: Vec<i32> = binary::Decode::decode(buf)?;
            Ok(Value::MultiDimArray(values, dimensions))
        }
        (true, false) => {
            let values: Vec<T> = binary::Decode::decode(buf)?;
            Ok(Value::Array(values))
        }
        (false, true) => {
            return Err(binary::DecodingError::InvalidData);
        }
        (false, false) => {
            let value: T = binary::Decode::decode(buf)?;
            Ok(Value::Scalar(value))
        }
    }
}
