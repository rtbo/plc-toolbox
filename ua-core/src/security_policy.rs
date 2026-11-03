use crate::msg::uasc::{self, AsymmetricSecurityHeader, SymmetricSecurityHeader};
use crate::transport::Uacp;
use crate::transport::tcp::SendBuffer;
use crate::ua;

#[allow(async_fn_in_trait)]
pub trait SecurityPolicy: Sized {
    fn uri(&self) -> &'static str;

    fn max_body_size(&self, max_send_size: u32) -> u32;

    fn asymmetric_security_header(&self) -> AsymmetricSecurityHeader;

    async fn send_msg_chunk<F>(
        &mut self,
        uacp: &mut Uacp,
        msg_size: u32,
        encode_fn: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut SendBuffer<'_>);

    async fn recv_msg_chunk<'a>(
        &'a mut self,
        uacp: &'a mut Uacp,
    ) -> Result<&'a [u8], ua::StatusCode>;
}

#[derive(Debug)]
pub struct SecurityPolicyNone;

impl SecurityPolicy for SecurityPolicyNone {
    fn uri(&self) -> &'static str {
        "http://opcfoundation.org/UA/SecurityPolicy#None"
    }

    fn max_body_size(&self, max_send_size: u32) -> u32 {
        max_send_size - uasc::Header::LEN - SymmetricSecurityHeader::LEN - uasc::SequenceHeader::LEN
    }

    fn asymmetric_security_header(&self) -> AsymmetricSecurityHeader {
        AsymmetricSecurityHeader {
            security_policy_uri: Some(self.uri().to_string()),
            sender_certificate: None,
            receiver_certificate_thumbprint: None,
        }
    }

    async fn send_msg_chunk<F>(
        &mut self,
        uacp: &mut Uacp,
        chunk_size: u32,
        encode_fn: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut SendBuffer<'_>),
    {
        uacp.send_chunk(chunk_size as usize, encode_fn).await?;
        Ok(())
    }

    async fn recv_msg_chunk<'a>(
        &'a mut self,
        uacp: &'a mut Uacp,
    ) -> Result<&'a [u8], ua::StatusCode> {
        let buf = uacp.recv_chunk().await?;
        Ok(buf)
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
    fn max_body_size(&self, max_send_size: u32) -> u32 {
        match self {
            AnySecurityPolicy::None(policy) => policy.max_body_size(max_send_size),
        }
    }
    fn asymmetric_security_header(&self) -> AsymmetricSecurityHeader {
        match self {
            AnySecurityPolicy::None(policy) => policy.asymmetric_security_header(),
        }
    }
    async fn send_msg_chunk<F>(
        &mut self,
        uacp: &mut Uacp,
        msg_size: u32,
        encode_fn: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut SendBuffer<'_>),
    {
        match self {
            AnySecurityPolicy::None(policy) => {
                policy.send_msg_chunk(uacp, msg_size, encode_fn).await
            }
        }
    }

    async fn recv_msg_chunk<'a>(
        &'a mut self,
        uacp: &'a mut Uacp,
    ) -> Result<&'a [u8], ua::StatusCode> {
        match self {
            AnySecurityPolicy::None(policy) => policy.recv_msg_chunk(uacp).await,
        }
    }
}
