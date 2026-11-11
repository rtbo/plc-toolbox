use crate::msg::uasc::{self, AsymmetricSecurityHeader, SymmetricSecurityHeader};
use crate::transport::tcp;
use crate::ua;

#[allow(async_fn_in_trait)]
pub trait SecurityPolicy: Sized {
    fn uri(&self) -> &'static str;

    fn max_body_size(&self, max_send_size: u32) -> u32;

    fn asymmetric_security_header(&self) -> AsymmetricSecurityHeader;

    async fn send_msg_chunk(
        &mut self,
        stream: &mut tcp::SendHalf,
        buf: &bytes::BytesMut,
    ) -> Result<(), ua::StatusCode>;

    async fn recv_msg_chunk(
        &mut self,
        stream: &mut tcp::RecvHalf,
        buf: &mut bytes::BytesMut,
    ) -> Result<(), ua::StatusCode>;
}

#[derive(Debug, Clone)]
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

    async fn send_msg_chunk(
        &mut self,
        stream: &mut tcp::SendHalf,
        buf: &bytes::BytesMut,
    ) -> Result<(), ua::StatusCode> {
        stream
            .send_chunk(buf)
            .await
            .map_err(|_| ua::StatusCode::BadCommunicationError)?;
        Ok(())
    }

    async fn recv_msg_chunk(
        &mut self,
        stream: &mut tcp::RecvHalf,
        buf: &mut bytes::BytesMut,
    ) -> Result<(), ua::StatusCode> {
        stream.recv_chunk(buf).await
    }
}

#[derive(Debug, Clone)]
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
    async fn send_msg_chunk(
        &mut self,
        stream: &mut tcp::SendHalf,
        buf: &bytes::BytesMut,
    ) -> Result<(), ua::StatusCode> {
        match self {
            AnySecurityPolicy::None(policy) => policy.send_msg_chunk(stream, buf).await,
        }
    }

    async fn recv_msg_chunk(
        &mut self,
        stream: &mut tcp::RecvHalf,
        buf: &mut bytes::BytesMut,
    ) -> Result<(), ua::StatusCode> {
        match self {
            AnySecurityPolicy::None(policy) => policy.recv_msg_chunk(stream, buf).await,
        }
    }
}
