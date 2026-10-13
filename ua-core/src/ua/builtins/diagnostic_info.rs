use crate::msg::binary;
use crate::{DataType, ua};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticInfo {
    pub symbolic_id: Option<i32>,
    pub namespace_uri: Option<i32>,
    pub localized_text: Option<i32>,
    pub locale: Option<i32>,
    pub additional_info: Option<String>,
    pub inner_status_code: Option<ua::StatusCode>,
    pub inner_diagnostic_info: Option<Box<DiagnosticInfo>>,
}

impl DataType for DiagnosticInfo {
    fn type_id() -> ua::NodeId {
        ua::NodeId::Numeric(0, 25)
    }

    fn nullable() -> bool {
        true
    }

    fn default() -> Self {
        DiagnosticInfo {
            symbolic_id: None,
            namespace_uri: None,
            localized_text: None,
            locale: None,
            additional_info: None,
            inner_status_code: None,
            inner_diagnostic_info: None,
        }
    }
}

impl binary::Encode for DiagnosticInfo {
    fn encoded_len(&self) -> u32 {
        let mut len = 1;
        if let Some(symbolic_id) = &self.symbolic_id {
            len += symbolic_id.encoded_len();
        }
        if let Some(namespace_uri) = &self.namespace_uri {
            len += namespace_uri.encoded_len();
        }
        if let Some(localized_text) = &self.localized_text {
            len += localized_text.encoded_len();
        }
        if let Some(locale) = &self.locale {
            len += locale.encoded_len();
        }
        if let Some(additional_info) = &self.additional_info {
            len += additional_info.encoded_len();
        }
        if let Some(inner_status_code) = &self.inner_status_code {
            len += inner_status_code.encoded_len();
        }
        if let Some(inner_diagnostic_info) = &self.inner_diagnostic_info {
            len += inner_diagnostic_info.encoded_len();
        }
        len
    }

    fn encode<B: binary::BufMut>(&self, buf: &mut B) {
        let mut flags = 0x00;
        if self.symbolic_id.is_some() {
            flags |= 0x01;
        }
        if self.namespace_uri.is_some() {
            flags |= 0x02;
        }
        if self.localized_text.is_some() {
            flags |= 0x04;
        }
        if self.locale.is_some() {
            flags |= 0x08;
        }
        if self.additional_info.is_some() {
            flags |= 0x10;
        }
        if self.inner_status_code.is_some() {
            flags |= 0x20;
        }
        if self.inner_diagnostic_info.is_some() {
            flags |= 0x40;
        }
        buf.put_u8(flags);
        if let Some(symbolic_id) = &self.symbolic_id {
            symbolic_id.encode(buf);
        }
        if let Some(namespace_uri) = &self.namespace_uri {
            namespace_uri.encode(buf);
        }
        if let Some(localized_text) = &self.localized_text {
            localized_text.encode(buf);
        }
        if let Some(locale) = &self.locale {
            locale.encode(buf);
        }
        if let Some(additional_info) = &self.additional_info {
            additional_info.encode(buf);
        }
        if let Some(inner_status_code) = &self.inner_status_code {
            inner_status_code.encode(buf);
        }
        if let Some(inner_diagnostic_info) = &self.inner_diagnostic_info {
            inner_diagnostic_info.encode(buf);
        }
    }
}

impl binary::Decode for DiagnosticInfo {
    fn decode<B: binary::Buf>(buf: &mut B) -> Result<Self, binary::DecodingError> {
        binary::check_remaining(buf, 1)?;
        let flags = buf.get_u8();
        let symbolic_id = if flags & 0x01 != 0 {
            Some(i32::decode(buf)?)
        } else {
            None
        };
        let namespace_uri = if flags & 0x02 != 0 {
            Some(i32::decode(buf)?)
        } else {
            None
        };
        let localized_text = if flags & 0x04 != 0 {
            Some(i32::decode(buf)?)
        } else {
            None
        };
        let locale = if flags & 0x08 != 0 {
            Some(i32::decode(buf)?)
        } else {
            None
        };
        let additional_info = if flags & 0x10 != 0 {
            Some(String::decode(buf)?)
        } else {
            None
        };
        let inner_status_code = if flags & 0x20 != 0 {
            Some(ua::StatusCode::decode(buf)?)
        } else {
            None
        };
        let inner_diagnostic_info = if flags & 0x40 != 0 {
            Some(Box::new(DiagnosticInfo::decode(buf)?))
        } else {
            None
        };

        Ok(DiagnosticInfo {
            symbolic_id,
            namespace_uri,
            localized_text,
            locale,
            additional_info,
            inner_status_code,
            inner_diagnostic_info,
        })
    }
}
