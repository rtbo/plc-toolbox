//! Client implementation for the UA Secure Channel Binary protocols:
//!  - UA-TCP UA-SC UA-Binary
//!  - WSS UA-SC UA-Binary (TODO)
use std::sync::Arc;

use tokio::sync::{Mutex, mpsc, oneshot};
use ua_core::encoding::binary;
use ua_core::encoding::binary::{Decode, Encode};
use ua_core::msg::BinaryEncodingId;
use ua_core::transport::uacp::Msg;
use ua_core::transport::{self, RecvChunk, SendChunk, uacp};
use ua_core::{AnySecurityPolicy, BufferPool, SecurityPolicy, SecurityPolicyNone, msg, ua};

use crate::Config;
use crate::secure_channel;
use crate::smallmap::SmallMap;

impl super::BufferConfig {
    async fn hello_ack<S: transport::SendData, R: transport::RecvData>(
        self,
        endpoint_url: String,
        send: &mut SendChunk<S>,
        recv: &mut RecvChunk<R>,
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

#[derive(Debug)]
pub struct ScBinClient<S> {
    // mutex containing the current request id and secure channel send half
    sc_send: Mutex<(u32, secure_channel::SendHalf<AnySecurityPolicy, S>)>,
    decoding_ctx: binary::DecodingCtx,
    tx: mpsc::UnboundedSender<RecvCommand>,
    recv_pool: Arc<BufferPool>,
}

enum RecvCommand {
    GetResponse(
        u32,
        oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
    ),
}

impl<S: transport::SendData> ScBinClient<S> {
    pub async fn connect<R>(config: Config, send: S, recv: R) -> Result<Self, ua::StatusCode>
    where
        R: transport::RecvData + Send + 'static,
    {
        let decoding_ctx = binary::DecodingCtx::default();
        let mut send = SendChunk::new(send, 8192);
        let mut recv = RecvChunk::new(recv, 8192);

        let buffer_config = config
            .buffer
            .hello_ack(config.endpoint.clone(), &mut send, &mut recv, &decoding_ctx)
            .await?;

        send.adjust_max_chunk_size(buffer_config.send_size);
        recv.adjust_max_chunk_size(buffer_config.recv_size);

        let _config = Config {
            buffer: buffer_config,
            ..config
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

        let sc_send = Mutex::new((2, sc_send));

        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(run_secure_channel_recv(sc_recv, recv_pool.clone(), rx));

        Ok(Self {
            sc_send,
            decoding_ctx,
            tx,
            recv_pool,
        })
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: msg::Request + binary::Encode + msg::BinaryEncodingId,
        R::Response: binary::Decode + msg::BinaryEncodingId,
    {
        let encoding = R::binary_encoding_node_id(0);
        let body_len = encoding.encoded_len() + request.encoded_len();

        let req_id = {
            let mut sc_send = self.sc_send.lock().await;
            let req_id = sc_send.0;
            sc_send.0 += 1;
            println!("sending request {} (length: {})", req_id, body_len);
            sc_send
                .1
                .send_msg(req_id, body_len, |buf| {
                    encoding.encode(buf);
                    request.encode(buf);
                })
                .await?;
            req_id
        };

        let (resp_tx, resp_rx) = oneshot::channel();
        self.tx
            .send(RecvCommand::GetResponse(req_id, resp_tx))
            .map_err(|err| {
                eprintln!("Failed to send request: {}", err);
                ua::StatusCode::BadInternalError
            })?;

        let mut buf = resp_rx.await.map_err(|err| {
            eprintln!("Failed to receive response: {}", err);
            ua::StatusCode::BadInternalError
        })??;

        println!(
            "received response for request {} (length: {})",
            req_id,
            buf.len()
        );

        let encoding = ua::NodeId::decode(&mut buf, &self.decoding_ctx)?;

        let result = if encoding == R::Response::binary_encoding_node_id(0) {
            Ok(R::Response::decode(&mut buf, &self.decoding_ctx)?)
        } else {
            Err(ua::StatusCode::BadNodeIdInvalid)
        };

        self.recv_pool.release(buf).await;

        result
    }
}

async fn run_secure_channel_recv<P: SecurityPolicy, R>(
    mut recv_half: secure_channel::RecvHalf<P, R>,
    recv_pool: Arc<BufferPool>,
    mut rx: mpsc::UnboundedReceiver<RecvCommand>,
) -> Result<(), ua::StatusCode>
where
    R: transport::RecvData + Send,
{
    let mut response_cache: SmallMap<u32, Result<bytes::BytesMut, ua::StatusCode>> =
        SmallMap::with_capacity(8);
    let mut pending_requests: SmallMap<
        u32,
        oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
    > = SmallMap::with_capacity(8);

    loop {
        tokio::select! {
            cmd = rx.recv() => {
                match cmd {
                    Some(RecvCommand::GetResponse(req_id, tx)) => {
                        if let Some(result) = response_cache.remove(&req_id) {
                            let _ = tx.send(result);
                        } else {
                            pending_requests.insert(req_id, tx);
                        }
                    }
                    None => break,
                }
            }
            (recv_res, buf) = async {
                let mut buf = recv_pool.acquire().await;
                let recv_res = recv_half.recv_msg(&mut buf).await;
                (recv_res, buf)
            } => {
                match recv_res {
                    Ok(req_id) => {
                        if let Some(tx) = pending_requests.remove(&req_id) {
                            let _ = tx.send(Ok(buf));
                        } else {
                            response_cache.insert(req_id, Ok(buf));
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
