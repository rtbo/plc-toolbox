use std::sync::Arc;

use smallvec::SmallVec;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use ua_core::SecurityPolicy;
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
use crate::small_map::SmallMap;

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
    decoding_ctx: binary::DecodingCtx,
    req_id: u32,
    tx: mpsc::UnboundedSender<RecvCommand>,
    recv_pool: Arc<BufferPool>,
}

enum RecvCommand {
    GetResponse(
        u32,
        oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
    ),
}

impl TcpClient {
    pub async fn connect(endpoint: &str, config: Config) -> Result<Self, ua::StatusCode> {
        let (mut send, mut recv) = tcp::connect(endpoint, 8192, 8192).await?;

        let decoding_ctx = binary::DecodingCtx::default();

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
        )
        .await?;

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(run_secure_channel_recv(sc_recv, recv_pool.clone(), rx));

        Ok(Self {
            sc_send,
            decoding_ctx,
            req_id: 2,
            tx,
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

        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(RecvCommand::GetResponse(self.req_id, resp_tx))
            .map_err(|_| ua::StatusCode::BadInternalError)?;

        let mut buf = resp_rx
            .await
            .map_err(|_| ua::StatusCode::BadInternalError)??;

        let encoding = ua::NodeId::decode(&mut buf, &self.decoding_ctx)?;

        let result = if encoding == R::Response::binary_encoding_node_id(0) {
            Ok(R::Response::decode(&mut buf, &self.decoding_ctx)?)
        } else {
            Err(ua::StatusCode::BadNodeIdInvalid)
        };

        self.req_id += 1;
        self.recv_pool.release(buf).await;

        result
    }
}

type ResponseCache = SmallVec<[Option<(u32, Result<bytes::BytesMut, ua::StatusCode>)>; 8]>;

async fn run_secure_channel_recv<P: SecurityPolicy>(
    mut recv_half: secure_channel::RecvHalf<P>,
    recv_pool: Arc<BufferPool>,
    mut rx: mpsc::UnboundedReceiver<RecvCommand>,
) -> Result<(), ua::StatusCode> {
    let mut cache: ResponseCache = SmallVec::new();
    let mut pending_requests: SmallMap<
        u32,
        oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
    > = SmallMap::with_capacity(8);

    loop {
        tokio::select! {
            cmd = rx.recv() => {
                match cmd {
                    Some(RecvCommand::GetResponse(req_id, tx)) => {
                        if let Some(idx) = cache.iter().position(|entry| {
                            entry.as_ref().map_or(false, |(id, _)| *id == req_id)
                        }) {
                            if let Some((_, result)) = cache.swap_remove(idx) {
                                let _ = tx.send(result);
                            }
                        } else {
                            pending_requests.insert(req_id, tx);
                        }
                    }
                    None => break,
                }
            }
            result = async {
                let mut buf = recv_pool.acquire().await;
                (recv_half.recv_msg(&mut buf).await, buf)
            } => {
                let (msg_result, buf) = result;
                match msg_result {
                    Ok(req_id) => {
                        if let Some(tx) = pending_requests.remove(&req_id) {
                            let _ = tx.send(Ok(buf));
                        } else {
                            cache.push(Some((req_id, Ok(buf))));
                        }
                    }
                    Err(err) => {
                        recv_pool.release(buf).await;
                        return Err(err);
                    }
                }
            }
        }
    }

    Ok(())
}
