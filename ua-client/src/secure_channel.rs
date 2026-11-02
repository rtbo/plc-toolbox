use ua_core::encoding::binary::{self, Decode, Encode};
use ua_core::msg::{self, BinaryEncodingId, uasc};
use ua_core::{SecurityPolicy, SecurityPolicyNone, transport, ua};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Transport error: {0}")]
    Transport(#[from] ua_core::transport::Error),

    #[error("Decode error: {0}")]
    Decode(#[from] binary::DecodingError),

    #[error("Status code: {0}")]
    StatusCode(#[from] ua::StatusCode),
}

#[derive(Debug)]
pub struct SecureChannel<P: SecurityPolicy = SecurityPolicyNone> {
    uacp: transport::Uacp,
    security_policy: P,
    channel_id: u32,
    token_id: u32,
    sequence_number: u32,
    srv_sequence_number: u32,
}

impl<P: SecurityPolicy> SecureChannel<P> {
    pub async fn open(
        uacp: transport::Uacp,
        security_policy: P,
        request_id: u32,
        msg: ua::OpenSecureChannelRequest,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<Self, Error> {
        let mut sc = Self {
            uacp,
            security_policy,
            channel_id: 0,
            token_id: 0,
            sequence_number: 0,
            srv_sequence_number: 0,
        };

        let uri = sc.security_policy.uri();

        sc.send_open(msg, request_id).await?;

        let mut recv_buf = sc.security_policy.recv_msg_chunk(&mut sc.uacp).await?;
        let header = uasc::Header::decode(&mut recv_buf, decoding_ctx)?;
        if header.msg_type != uasc::MsgType::Open || header.is_final != uasc::IsFinal::Final {
            return Err(binary::DecodingError::InvalidData.into());
        }
        let security_header = uasc::AsymmetricSecurityHeader::decode(&mut recv_buf, decoding_ctx)?;
        if security_header.security_policy_uri.as_deref() != Some(uri) {
            return Err(binary::DecodingError::InvalidData.into());
        }
        let seq_header = uasc::SequenceHeader::decode(&mut recv_buf, decoding_ctx)?;
        if seq_header.request_id != request_id {
            return Err(binary::DecodingError::InvalidData.into());
        }
        let encoding_node_id = ua::NodeId::decode(&mut recv_buf, decoding_ctx)?;
        if encoding_node_id != ua::OpenSecureChannelResponse::binary_encoding_node_id(0) {
            return Err(binary::DecodingError::InvalidData.into());
        }
        let rsp = ua::OpenSecureChannelResponse::decode(&mut recv_buf, decoding_ctx)?;

        sc.srv_sequence_number = seq_header.sequence_number;
        sc.channel_id = rsp.security_token.channel_id;
        sc.token_id = rsp.security_token.token_id;
        Ok(sc)
    }

    async fn send_open(
        &mut self,
        msg: ua::OpenSecureChannelRequest,
        request_id: u32,
    ) -> Result<(), ua_core::transport::Error> {
        let encoding_node_id = ua::OpenSecureChannelRequest::binary_encoding_node_id(0);
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        let security_header = self.security_policy.asymmetric_security_header();
        let header = uasc::Header {
            msg_type: uasc::MsgType::Open,
            is_final: uasc::IsFinal::Final,
            msg_size: uasc::Header::LEN
                + security_header.encoded_len()
                + uasc::SequenceHeader::LEN
                + body_len,
            secure_channel_id: self.channel_id,
        };

        let seq_header = uasc::SequenceHeader {
            sequence_number: self.sequence_number,
            request_id,
        };

        self.security_policy
            .send_msg_chunk(&mut self.uacp, header.msg_size, |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encoding_node_id.encode(buf);
                msg.encode(buf);
            })
            .await?;

        self.sequence_number += 1;

        self.uacp.flush().await?;
        Ok(())
    }

    pub async fn send_msg<M>(&mut self, ns_index: u16, msg: M, request_id: u32) -> Result<(), Error>
    where
        M: binary::Encode + msg::BinaryEncodingId,
    {
        // TODO: implement chunking
        let encoding_node_id = M::binary_encoding_node_id(ns_index);
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        let security_header = uasc::SymmetricSecurityHeader {
            token_id: self.token_id,
        };
        let header = uasc::Header {
            msg_type: uasc::MsgType::Msg,
            is_final: uasc::IsFinal::Final,
            msg_size: uasc::Header::LEN
                + uasc::SymmetricSecurityHeader::LEN
                + uasc::SequenceHeader::LEN
                + body_len,
            secure_channel_id: self.channel_id,
        };

        let seq_header = uasc::SequenceHeader {
            sequence_number: self.sequence_number,
            request_id,
        };

        self.security_policy
            .send_msg_chunk(&mut self.uacp, header.msg_size, |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encoding_node_id.encode(buf);
                msg.encode(buf);
            })
            .await?;

        self.sequence_number += 1;

        self.uacp.flush().await?;
        Ok(())
    }

    pub async fn recv_msg(
        &mut self,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<(ua::NodeId, &[u8]), Error> {
        let mut recv_buf = self.security_policy.recv_msg_chunk(&mut self.uacp).await?;

        let header = uasc::Header::decode(&mut recv_buf, decoding_ctx)?;
        if header.msg_type != uasc::MsgType::Open || header.is_final != uasc::IsFinal::Final {
            return Err(binary::DecodingError::InvalidData.into());
        }
        let security_header = uasc::SymmetricSecurityHeader::decode(&mut recv_buf, decoding_ctx)?;
        if security_header.token_id != self.token_id {
            return Err(ua::StatusCode::BadSecureChannelTokenUnknown.into());
        }

        let seq_header = uasc::SequenceHeader::decode(&mut recv_buf, decoding_ctx)?;
        if seq_header.sequence_number != self.srv_sequence_number + 1 {
            return Err(ua::StatusCode::BadSequenceNumberInvalid.into());
        }
        self.srv_sequence_number = seq_header.sequence_number;

        let encoding_node_id = ua::NodeId::decode(&mut recv_buf, decoding_ctx)?;

        Ok((encoding_node_id, recv_buf))
    }
}
