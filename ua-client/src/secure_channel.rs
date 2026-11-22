use std::sync::Arc;

use ua_core::encoding::Ns0BinaryEncodingId;
use ua_core::encoding::binary::{self, Decode, Encode};
use ua_core::transport::{self, RecvChunk, SendChunk};
use ua_core::{BufferPool, SecurityPolicy, ua, uasc};

pub async fn open<P, S, R>(
    security_policy: P,
    send: SendChunk<S>,
    recv: RecvChunk<R>,
    config: Arc<crate::Config>,
    send_pool: Arc<BufferPool>,
    recv_pool: Arc<BufferPool>,
    request_id: u32,
) -> Result<(SendHalf<P, S>, RecvHalf<P, R>), ua::StatusCode>
where
    P: SecurityPolicy + Clone,
    S: transport::SendData + Send,
    R: transport::RecvData + Send,
{
    let mut send_half = SendHalf::new(send, config, send_pool, security_policy.clone());
    let mut recv_half = RecvHalf::new(recv, recv_pool.clone(), security_policy);

    let msg = ua::OpenSecureChannelRequest {
        request_header: ua::RequestHeader {
            timestamp: ua::DateTime::now(),
            ..Default::default()
        },
        client_protocol_version: 0,
        security_mode: ua::MessageSecurityMode::None,
        request_type: ua::SecurityTokenRequestType::Issue,
        client_nonce: vec![],
        requested_lifetime: 60000,
    };

    send_half.send_open(msg, request_id).await?;

    let rsp = recv_pool
        .with_buffer(async |recv_buf| recv_half.recv_open(recv_buf, request_id).await)
        .await?;

    send_half.channel_id = rsp.security_token.channel_id;
    send_half.token_id = rsp.security_token.token_id;
    recv_half.channel_id = rsp.security_token.channel_id;
    recv_half.token_id = rsp.security_token.token_id;

    Ok((send_half, recv_half))
}

#[derive(Debug)]
pub struct SendHalf<P, S> {
    send: SendChunk<S>,
    config: Arc<crate::Config>,
    pool: Arc<BufferPool>,
    security_policy: P,
    channel_id: u32,
    token_id: u32,
    sequence_number: u32,
}

impl<P, S> SendHalf<P, S>
where
    P: SecurityPolicy,
    S: transport::SendData + Send,
{
    pub(super) fn new(
        send: SendChunk<S>,
        config: Arc<crate::Config>,
        pool: Arc<BufferPool>,
        security_policy: P,
    ) -> Self {
        Self {
            send,
            config,
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
        let encoding_node_id = ua::OpenSecureChannelRequest::BINARY_ENCODING_NODE_ID;
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

        self.pool
            .with_buffer(async |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encoding_node_id.encode(buf);
                msg.encode(buf);
                self.security_policy
                    .send_msg_chunk(&mut self.send, buf)
                    .await
            })
            .await?;

        self.sequence_number += 1;

        Ok(())
    }

    pub async fn send_close(
        &mut self,
        request_id: u32,
        msg: ua::CloseSecureChannelRequest,
    ) -> Result<(), ua::StatusCode> {
        let encoding_node_id = ua::CloseSecureChannelRequest::BINARY_ENCODING_NODE_ID;
        let body_len = encoding_node_id.encoded_len() + msg.encoded_len();

        self.send_msg_chunk(
            uasc::MsgType::Close,
            uasc::IsFinal::Final,
            request_id,
            body_len,
            |buf| {
                encoding_node_id.encode(buf);
                msg.encode(buf);
            },
        )
        .await?;

        self.token_id = 0;
        self.sequence_number = 0;
        self.channel_id = 0;

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
        let max_body_len = self
            .security_policy
            .max_body_size(self.config.buffer.send_size);

        if body_len as usize <= max_body_len {
            // Single chunk message
            self.send_msg_chunk(
                uasc::MsgType::Msg,
                uasc::IsFinal::Final,
                request_id,
                body_len,
                encode_fn,
            )
            .await?;
        } else {
            // Message chunking logic
            let body_len = body_len as usize;
            let mut buf = bytes::BytesMut::with_capacity(body_len);
            encode_fn(&mut buf);

            let mut offset = 0;
            while offset < buf.len() {
                let chunk_size = max_body_len.min(buf.len() - offset);
                let chunk = &buf[offset..offset + chunk_size];

                let is_final = if offset + chunk_size == buf.len() {
                    uasc::IsFinal::Final
                } else {
                    uasc::IsFinal::Continue
                };

                self.send_msg_chunk(
                    uasc::MsgType::Msg,
                    is_final,
                    request_id,
                    chunk_size as u32,
                    |buf| buf.extend_from_slice(chunk),
                )
                .await?;

                offset += chunk_size;
            }
        }

        Ok(())
    }

    async fn send_msg_chunk<F>(
        &mut self,
        msg_type: uasc::MsgType,
        is_final: uasc::IsFinal,
        request_id: u32,
        body_len: u32,
        encode_fn: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut bytes::BytesMut),
    {
        let security_header = uasc::SymmetricSecurityHeader {
            token_id: self.token_id,
        };
        let header = uasc::Header {
            msg_type,
            is_final,
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

        self.pool
            .with_buffer(async |buf| {
                header.encode(buf);
                security_header.encode(buf);
                seq_header.encode(buf);
                encode_fn(buf);
                self.security_policy
                    .send_msg_chunk(&mut self.send, buf)
                    .await
            })
            .await?;

        self.sequence_number += 1;

        Ok(())
    }
}

#[derive(Debug)]
pub struct RecvHalf<P, R> {
    recv: RecvChunk<R>,
    security_policy: P,
    _pool: Arc<BufferPool>,
    decoding_ctx: binary::DecodingCtx,
    channel_id: u32,
    token_id: u32,
    srv_seq_number: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct RecvChunkInfo {
    pub request_id: u32,
    pub is_final: uasc::IsFinal,
}

impl<P, R> RecvHalf<P, R>
where
    P: SecurityPolicy,
    R: transport::RecvData + Send,
{
    pub(super) fn new(recv: RecvChunk<R>, pool: Arc<BufferPool>, security_policy: P) -> Self {
        Self {
            recv,
            _pool: pool,
            security_policy,
            // This structs only decodes headers and can live with an empty decoding context.
            decoding_ctx: binary::DecodingCtx::new(),
            channel_id: 0,
            token_id: 0,
            srv_seq_number: 0,
        }
    }

    pub async fn recv_open(
        &mut self,
        buf: &mut bytes::BytesMut,
        request_id: u32,
    ) -> Result<ua::OpenSecureChannelResponse, ua::StatusCode> {
        self.security_policy
            .recv_msg_chunk(&mut self.recv, buf)
            .await?;

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
        let rsp = match encoding_node_id {
            ua::ServiceFault::BINARY_ENCODING_NODE_ID => {
                let resp = ua::ServiceFault::decode(buf, &self.decoding_ctx)?;
                return Err(resp.response_header.service_result);
            }
            ua::OpenSecureChannelResponse::BINARY_ENCODING_NODE_ID => {
                ua::OpenSecureChannelResponse::decode(buf, &self.decoding_ctx)?
            }
            _ => {
                return Err(ua::StatusCode::BadNodeIdInvalid);
            }
        };

        self.srv_seq_number = seq_header.sequence_number;

        Ok(rsp)
    }

    pub async fn recv_msg(
        &mut self,
        buf: &mut bytes::BytesMut,
    ) -> Result<RecvChunkInfo, ua::StatusCode> {
        self.security_policy
            .recv_msg_chunk(&mut self.recv, buf)
            .await?;

        let header = uasc::Header::decode(buf, &self.decoding_ctx)?;

        if header.msg_type != uasc::MsgType::Msg {
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
        Ok(RecvChunkInfo {
            request_id: seq_header.request_id,
            is_final: header.is_final,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    struct MockSendData {
        chunks: Vec<bytes::BytesMut>,
    }

    impl MockSendData {
        fn new() -> Self {
            Self { chunks: Vec::new() }
        }
    }

    impl transport::SendData for MockSendData {
        async fn send_data(&mut self, data: &bytes::BytesMut) -> Result<(), io::Error> {
            self.chunks.push(data.clone());
            Ok(())
        }
    }

    struct MockSecurityPolicy;

    impl SecurityPolicy for MockSecurityPolicy {
        fn uri(&self) -> &'static str {
            "http://opcfoundation.org/UA/SecurityPolicy#None"
        }

        fn max_body_size(&self, max_send_size: usize) -> usize {
            // Similar to SecurityPolicyNone: buffer size minus headers
            max_send_size
                - uasc::Header::LEN as usize
                - uasc::SymmetricSecurityHeader::LEN as usize
                - uasc::SequenceHeader::LEN as usize
        }

        fn asymmetric_security_header(&self) -> uasc::AsymmetricSecurityHeader {
            uasc::AsymmetricSecurityHeader {
                security_policy_uri: Some(self.uri().to_string()),
                sender_certificate: None,
                receiver_certificate_thumbprint: None,
            }
        }

        async fn send_msg_chunk<T: transport::SendData + Send>(
            &mut self,
            stream: &mut transport::SendChunk<T>,
            buf: &bytes::BytesMut,
        ) -> Result<(), ua::StatusCode> {
            stream
                .send_chunk(buf)
                .await
                .map_err(|_| ua::StatusCode::BadCommunicationError)?;
            Ok(())
        }

        async fn recv_msg_chunk<T: transport::RecvData + Send>(
            &mut self,
            stream: &mut transport::RecvChunk<T>,
            buf: &mut bytes::BytesMut,
        ) -> Result<(), ua::StatusCode> {
            stream.recv_chunk(buf).await
        }
    }

    fn send_half() -> SendHalf<MockSecurityPolicy, MockSendData> {
        let pool = Arc::new(BufferPool::new(65536, 10));
        let send = SendChunk::new(MockSendData::new(), 65536);
        let config = Arc::new(crate::Config::default());
        let policy = MockSecurityPolicy;
        SendHalf::new(send, config, pool, policy)
    }

    #[test]
    fn test_send_half_initialization() {
        let send_half = send_half();

        assert_eq!(send_half.channel_id, 0);
        assert_eq!(send_half.token_id, 0);
        assert_eq!(send_half.sequence_number, 0);
    }

    #[tokio::test]
    async fn test_message_fits_in_single_chunk() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        let seq_before = send_half.sequence_number;
        let body_len = 100_u32;

        let result = send_half
            .send_msg(1, body_len, |buf| {
                buf.extend_from_slice(&vec![0xAB; 100]);
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(send_half.sequence_number, seq_before + 1);
    }

    #[tokio::test]
    async fn test_message_requires_multiple_chunks() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        let seq_before = send_half.sequence_number;
        let body_len = 120000_u32;

        let result = send_half
            .send_msg(1, body_len, |buf| {
                buf.extend_from_slice(&vec![0xCD; 120000]);
            })
            .await;

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        // Multiple chunks should result in multiple sequence increments
        assert!(send_half.sequence_number > seq_before + 1);
    }

    #[tokio::test]
    async fn test_sequence_number_increments_per_chunk() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        assert_eq!(send_half.sequence_number, 0);

        let result1 = send_half
            .send_msg(1, 100000, |buf| {
                buf.extend_from_slice(&vec![0xFF; 100000]);
            })
            .await;

        assert!(result1.is_ok(), "First message failed: {:?}", result1.err());
        let seq_after_first = send_half.sequence_number;
        assert!(seq_after_first > 0);

        let result2 = send_half
            .send_msg(2, 100, |buf| {
                buf.extend_from_slice(&vec![0x00; 100]);
            })
            .await;

        assert!(result2.is_ok());
        assert!(send_half.sequence_number > seq_after_first);
    }

    #[tokio::test]
    async fn test_chunking_with_boundary_size() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        let seq_before = send_half.sequence_number;
        let body_len = 2048_u32;

        let result = send_half
            .send_msg(1, body_len, |buf| {
                buf.extend_from_slice(&vec![0x55; 2048]);
            })
            .await;

        assert!(result.is_ok());
        let seq_after = send_half.sequence_number;
        assert!(seq_after >= seq_before);
    }

    #[tokio::test]
    async fn test_zero_length_message() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        let seq_before = send_half.sequence_number;

        let result = send_half
            .send_msg(1, 0, |_buf| {
                // Empty message
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(send_half.sequence_number, seq_before + 1);
    }

    #[tokio::test]
    async fn test_multiple_small_messages() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        // Send three small messages
        for i in 0..3 {
            let seq_before = send_half.sequence_number;
            let result = send_half
                .send_msg(i, 50, |buf| {
                    buf.extend_from_slice(&vec![i as u8; 50]);
                })
                .await;

            assert!(result.is_ok());
            assert_eq!(send_half.sequence_number, seq_before + 1);
        }

        // Should have incremented by 3
        assert_eq!(send_half.sequence_number, 3);
    }

    #[tokio::test]
    async fn test_chunking_consistency() {
        let mut send_half = send_half();
        send_half.channel_id = 1;
        send_half.token_id = 1;

        // Send a large message
        let test_data = vec![0x42; 200000];
        let seq_before = send_half.sequence_number;

        let result = send_half
            .send_msg(1, 200000, |buf| {
                buf.extend_from_slice(&test_data);
            })
            .await;

        assert!(result.is_ok());
        // Verify that we got multiple chunks
        // With 65536 buffer and headers, we should have about 3-4 chunks
        let chunks_sent = send_half.sequence_number - seq_before;
        assert!(chunks_sent >= 2);
    }
}
