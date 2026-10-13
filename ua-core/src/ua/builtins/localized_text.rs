use crate::msg::binary;
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedText {
    pub locale: Option<LocaleId>,
    pub text: String,
}

impl DataType for LocalizedText {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 21)
    }
    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        LocalizedText {
            locale: None,
            text: String::new(),
        }
    }
}

impl binary::Encode for LocalizedText {
    fn encoded_len(&self) -> u32 {
        let mut len = 1;
        if let Some(locale) = &self.locale {
            len += locale.0.encoded_len();
        }
        len += self.text.encoded_len();
        len
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        let mut flags = 0x02;
        if self.locale.is_some() {
            flags |= 0x01;
        }
        buf.put_u8(flags);
        if let Some(locale) = &self.locale {
            locale.0.encode(buf);
        }
        self.text.encode(buf);
    }
}

impl binary::Decode for LocalizedText {
    fn decode<B: binary::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let flags = buf.get_u8();
        let locale = if flags & 0x01 != 0 {
            Some(LocaleId(String::decode(buf)?))
        } else {
            None
        };
        let text = if flags & 0x02 != 0 {
            String::decode(buf)?
        } else {
            String::new()
        };
        Ok(LocalizedText { locale, text })
    }
}
