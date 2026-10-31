use crate::encoding::binary::Encode;
use crate::msg::uasc::{self, AsymmetricSecurityHeader, SymmetricSecurityHeader};
use crate::transport::Uacp;

#[derive(Debug)]
pub struct HeaderInfo {
    pub msg_type: uasc::MsgType,
    pub is_final: uasc::IsFinal,
    pub body_len: u32,
    pub channel_id: u32,
    pub seq_number: u32,
    pub request_id: u32,
}

impl HeaderInfo {
    fn into_uasc(self, security_header_len: u32) -> (uasc::Header, uasc::SequenceHeader) {
        let header = uasc::Header {
            msg_type: self.msg_type,
            is_final: self.is_final,
            msg_size: uasc::Header::LEN
                + security_header_len
                + uasc::SequenceHeader::LEN
                + self.body_len,
            secure_channel_id: self.channel_id,
        };
        let seq_header = uasc::SequenceHeader {
            sequence_number: self.seq_number,
            request_id: self.request_id,
        };
        (header, seq_header)
    }
}

#[allow(async_fn_in_trait)]
pub trait SecurityPolicy: Sized {
    fn uri(&self) -> &'static str;

    fn max_body_size(&self) -> u32;

    async fn send_msg_chunk<F>(
        &mut self,
        header_info: HeaderInfo,
        body_encode_fn: F,
    ) -> Result<(), crate::transport::Error>
    where
        F: FnOnce(&mut bytes::BytesMut) -> ();
}

#[derive(Debug)]
pub struct SecurityPolicyNone {
    uacp: Uacp,
}

impl SecurityPolicy for SecurityPolicyNone {
    fn uri(&self) -> &'static str {
        "http://opcfoundation.org/UA/SecurityPolicy#None"
    }

    fn max_body_size(&self) -> u32 {
        self.uacp.max_send_size()
            - uasc::Header::LEN
            - SymmetricSecurityHeader::LEN
            - uasc::SequenceHeader::LEN
    }

    async fn send_msg_chunk<F>(
        &mut self,
        header_info: HeaderInfo,
        body_encode_fn: F,
    ) -> Result<(), crate::transport::Error>
    where
        F: FnOnce(&mut bytes::BytesMut) -> (),
    {
        let security_header = AsymmetricSecurityHeader {
            security_policy_uri: Some(self.uri().to_string()),
            sender_certificate: None,
            receiver_certificate_thumbprint: None,
        };

        let (header, seq_header) = header_info.into_uasc(security_header.encoded_len());

        self.uacp
            .send_chunk(|buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                body_encode_fn(buf);
            })
            .await?;

        Ok(())
    }
}

impl SecurityPolicyNone {
    pub fn new(uacp: Uacp) -> Self {
        Self { uacp }
    }
}

#[derive(Debug)]
pub enum AnySecurityPolicy {
    None(SecurityPolicyNone),
}

impl SecurityPolicy for AnySecurityPolicy {
    fn uri(&self) -> &'static str {
        match self {
            AnySecurityPolicy::None(policy) => policy.uri(),
        }
    }
    fn max_body_size(&self) -> u32 {
        match self {
            AnySecurityPolicy::None(policy) => policy.max_body_size(),
        }
    }
    async fn send_msg_chunk<F>(
        &mut self,
        header_info: HeaderInfo,
        body_encode_fn: F,
    ) -> Result<(), crate::transport::Error>
    where
        F: FnOnce(&mut bytes::BytesMut) -> (),
    {
        match self {
            AnySecurityPolicy::None(policy) => {
                policy.send_msg_chunk(header_info, body_encode_fn).await
            }
        }
    }
}
