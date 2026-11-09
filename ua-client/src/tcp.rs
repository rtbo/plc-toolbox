use std::sync::Arc;

use ua_core::encoding::binary;
use ua_core::encoding::binary::Decode;
use ua_core::encoding::binary::Encode;
use ua_core::msg;
use ua_core::msg::BinaryEncodingId;
use ua_core::transport::tcp;
use ua_core::transport::uacp;
use ua_core::transport::uacp::Msg;
use ua_core::ua;
use ua_core::{AnySecurityPolicy, BufferPool, SecurityPolicyNone};

use crate::secure_channel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferConfig {
    pub send_size: usize,
    pub recv_size: usize,
    pub max_msg_chunks: Option<usize>,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            send_size: 65536,
            recv_size: 65536,
            max_msg_chunks: None,
        }
    }
}

impl BufferConfig {
    async fn hello_ack(
        self,
        endpoint_url: String,
        send: &mut tcp::SendHalf,
        recv: &mut tcp::RecvHalf,
        decoding_ctx: &binary::DecodingCtx,
    ) -> Result<Self, ua::StatusCode> {
        let mut buf = bytes::BytesMut::with_capacity(8192);

        let hello = uacp::Hello {
            protocol_version: 0,
            receive_buffer_size: self.recv_size as u32,
            send_buffer_size: self.send_size as u32,
            max_message_size: (self.recv_size * self.max_msg_chunks.unwrap_or(0)) as u32,
            max_chunk_count: self.max_msg_chunks.unwrap_or(0) as u32,
            endpoint_url,
        };
        hello.msg_encode(&mut buf);

        send.send_chunk(&mut buf).await?;

        buf.clear();
        recv.recv_chunk(&mut buf).await?;

        let srv_ack = uacp::Acknowledge::msg_decode(&mut buf, decoding_ctx)?;

        Ok(Self {
            send_size: srv_ack.receive_buffer_size as usize,
            recv_size: srv_ack.send_buffer_size as usize,
            max_msg_chunks: if srv_ack.max_chunk_count > 0 {
                Some(srv_ack.max_chunk_count as usize)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub buffer: BufferConfig,
}

pub struct TcpClient {
    sc_send: secure_channel::SendHalf<AnySecurityPolicy>,
    sc_recv: secure_channel::RecvHalf<AnySecurityPolicy>,
    _send_pool: Arc<BufferPool>,
    recv_pool: Arc<BufferPool>,
    decoding_ctx: Arc<binary::DecodingCtx>,
    req_id: u32,
}

impl TcpClient {
    pub async fn connect(endpoint: &str, config: Config) -> Result<Self, ua::StatusCode> {
        let (mut send, mut recv) = tcp::connect(endpoint, 8192, 8192).await?;

        let decoding_ctx = Arc::new(binary::DecodingCtx::default());

        let buffer_config = config
            .buffer
            .hello_ack(endpoint.to_string(), &mut send, &mut recv, &decoding_ctx)
            .await?;

        send.adjust_max_chunk_size(buffer_config.send_size);
        recv.adjust_max_chunk_size(buffer_config.recv_size);

        let _config = Config {
            buffer: buffer_config,
        };

        let send_pool = Arc::new(BufferPool::new(2, buffer_config.send_size));
        let recv_pool = Arc::new(BufferPool::new(8, buffer_config.recv_size));

        let sp = AnySecurityPolicy::None(SecurityPolicyNone);
        let (sc_send, sc_recv) = secure_channel::open(
            send,
            recv,
            send_pool.clone(),
            recv_pool.clone(),
            sp,
            1,
            ua::OpenSecureChannelRequest {
                request_header: ua::RequestHeader {
                    timestamp: ua::DateTime::now(),
                    ..Default::default()
                },
                client_protocol_version: 0,
                security_mode: ua::MessageSecurityMode::None,
                request_type: ua::SecurityTokenRequestType::Issue,
                client_nonce: vec![],
                requested_lifetime: 60000,
            },
            decoding_ctx.clone(),
        )
        .await?;

        Ok(Self {
            sc_send,
            sc_recv,
            decoding_ctx,
            req_id: 2,
            _send_pool: send_pool,
            recv_pool,
        })
    }

    pub async fn send_request<R>(&mut self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: msg::Request + binary::Encode + msg::BinaryEncodingId,
        R::Response: binary::Decode + msg::BinaryEncodingId,
    {
        let encoding = R::binary_encoding_node_id(0);
        let body_len = encoding.encoded_len() + request.encoded_len();

        self.sc_send
            .send_msg(self.req_id, body_len, |buf| {
                encoding.encode(buf);
                request.encode(buf);
            })
            .await?;

        let mut buf = self.recv_pool.acquire().await;
        let req_id = self.sc_recv.recv_msg(&mut buf).await?;

        if req_id != self.req_id {
            return Err(ua::StatusCode::BadNoMatch);
        }

        let encoding = ua::NodeId::decode(&mut buf, &self.decoding_ctx)?;
        if encoding != R::Response::binary_encoding_node_id(0) {
            return Err(ua::StatusCode::BadNodeIdInvalid);
        }
        let res = R::Response::decode(&mut buf, &self.decoding_ctx)?;

        self.req_id += 1;
        self.recv_pool.release(buf).await;
        Ok(res)
    }
}
