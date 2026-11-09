use std::sync::Arc;

use ua_core::encoding::binary::{self, Decode, Encode};
use ua_core::msg::{BinaryEncodingId, uasc};
use ua_core::transport::tcp;
use ua_core::{BufferPool, SecurityPolicy, ua};


pub async fn open<P: SecurityPolicy + Clone>(
    send: tcp::SendHalf,
    recv: tcp::RecvHalf,
    send_pool: Arc<BufferPool>,
    recv_pool: Arc<BufferPool>,
    security_policy: P,
    request_id: u32,
    msg: ua::OpenSecureChannelRequest,
    decoding_ctx: Arc<binary::DecodingCtx>,
) -> Result<(SendHalf<P>, RecvHalf<P>), ua::StatusCode> {

    let mut send_half = SendHalf::new(send, send_pool, security_policy.clone());

    let mut recv_half = RecvHalf::new(recv, recv_pool.clone(), security_policy, decoding_ctx);

    send_half.send_open(msg, request_id).await?;

    let mut recv_buf = recv_pool.acquire().await;
    let rsp = recv_half.recv_open(&mut recv_buf, request_id).await?;

    send_half.channel_id = rsp.security_token.channel_id;
    send_half.token_id = rsp.security_token.token_id;
    recv_half.channel_id = rsp.security_token.channel_id;
    recv_half.token_id = rsp.security_token.token_id;

    Ok((send_half, recv_half))
}

pub struct SendHalf<P> {
    send: tcp::SendHalf,
    pool: Arc<BufferPool>,
    security_policy: P,
    channel_id: u32,
    token_id: u32,
    sequence_number: u32,
}

impl<P: SecurityPolicy> SendHalf<P> {
    fn new(send: tcp::SendHalf, pool: Arc<BufferPool>, security_policy: P) -> Self {
        Self {
            send,
            pool,
            security_policy,
            channel_id: 0,
            token_id: 0,
            sequence_number: 0,
        }
    }
    async fn send_open(
        &mut self,
        msg: ua::OpenSecureChannelRequest,
        request_id: u32,
    ) -> Result<(), ua::StatusCode> {
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

        let mut buf = self.pool.acquire().await;

        header.encode(&mut buf);
        security_header.encode(&mut buf);
        seq_header.encode(&mut buf);
        encoding_node_id.encode(&mut buf);
        msg.encode(&mut buf);
        self.security_policy.send_msg_chunk(&mut self.send, &mut buf).await?;

        self.pool.release(buf).await;

        self.sequence_number += 1;

        Ok(())
    }

    pub async fn send_msg<F>(
        &mut self,
        request_id: u32,
        body_len: u32,
        encode_fn: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut bytes::BytesMut),
    {
        // TODO: implement chunking

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

        let mut buf = self.pool.acquire().await;

        header.encode(&mut buf);
        security_header.encode(&mut buf);
        seq_header.encode(&mut buf);
        encode_fn(&mut buf);
        self.security_policy.send_msg_chunk(&mut self.send, &mut buf).await?;

        self.pool.release(buf).await;

        self.sequence_number += 1;

        Ok(())
    }
}

pub struct RecvHalf<P> {
    recv: tcp::RecvHalf,
    security_policy: P,
    _pool: Arc<BufferPool>,
    decoding_ctx: Arc<binary::DecodingCtx>,
    channel_id: u32,
    token_id: u32,
    srv_seq_number: u32,
}

impl<P: SecurityPolicy> RecvHalf<P> {
    fn new(recv: tcp::RecvHalf, pool: Arc<BufferPool>, security_policy: P, decoding_ctx: Arc<binary::DecodingCtx>) -> Self {
        Self {
            recv,
            _pool: pool,
            security_policy,
            decoding_ctx,
            channel_id: 0,
            token_id: 0,
            srv_seq_number: 0,
        }
    }

    pub async fn recv_open(&mut self, buf: &mut bytes::BytesMut, request_id: u32) -> Result<ua::OpenSecureChannelResponse, ua::StatusCode> {
        self.security_policy.recv_msg_chunk(&mut self.recv, buf).await?;

        let header = uasc::Header::decode(buf, &self.decoding_ctx)?;
        if header.msg_type != uasc::MsgType::Open || header.is_final != uasc::IsFinal::Final {
            return Err(ua::StatusCode::BadTcpMessageTypeInvalid);
        }
        self.channel_id = header.secure_channel_id;

        let security_header = uasc::AsymmetricSecurityHeader::decode(buf, &self.decoding_ctx)?;

        if security_header.security_policy_uri.as_deref() != Some(self.security_policy.uri()) {
            return Err(ua::StatusCode::BadSecurityChecksFailed);
        }
        let seq_header = uasc::SequenceHeader::decode(buf, &self.decoding_ctx)?;
        if seq_header.request_id != request_id {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let encoding_node_id = ua::NodeId::decode(buf, &self.decoding_ctx)?;
        if encoding_node_id != ua::OpenSecureChannelResponse::binary_encoding_node_id(0) {
            return Err(ua::StatusCode::BadDecodingError);
        }
        let rsp = ua::OpenSecureChannelResponse::decode(buf, &self.decoding_ctx)?;

        self.srv_seq_number = seq_header.sequence_number;

        Ok(rsp)
    }

    pub async fn recv_msg(&mut self, buf: &mut bytes::BytesMut) -> Result<u32, ua::StatusCode> {
        
        self.security_policy.recv_msg_chunk(&mut self.recv, buf).await?;

        let header = uasc::Header::decode(buf, &self.decoding_ctx)?;

        if header.msg_type != uasc::MsgType::Msg || header.is_final != uasc::IsFinal::Final {
            return Err(ua::StatusCode::BadTcpMessageTypeInvalid);
        }
        if header.secure_channel_id != self.channel_id {
            return Err(ua::StatusCode::BadSecureChannelIdInvalid);
        }
        let security_header = uasc::SymmetricSecurityHeader::decode(buf, &self.decoding_ctx)?;
        if security_header.token_id != self.token_id {
            return Err(ua::StatusCode::BadSecureChannelTokenUnknown);
        }

        let seq_header = uasc::SequenceHeader::decode(buf, &self.decoding_ctx)?;
        if seq_header.sequence_number != self.srv_seq_number + 1 {
            return Err(ua::StatusCode::BadSequenceNumberInvalid);
        }
        self.srv_seq_number = seq_header.sequence_number;
        Ok(seq_header.request_id)
    }
}