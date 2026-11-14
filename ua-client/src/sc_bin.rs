//! Client implementation for the UA Secure Channel Binary protocols:
//!  - UA-TCP UA-SC UA-Binary
//!  - WSS UA-SC UA-Binary (TODO)
use std::sync::Arc;

use tokio::sync::{Mutex, mpsc, oneshot, watch};
use ua_core::encoding::binary;
use ua_core::encoding::binary::{Decode, Encode};
use ua_core::msg::BinaryEncodingId;
use ua_core::transport::uacp::Msg;
use ua_core::transport::{self, RecvChunk, SendChunk, uacp};
use ua_core::{AnySecurityPolicy, BufferPool, SecurityPolicy, SecurityPolicyNone, msg, ua};

use crate::smallmap::SmallMap;
use crate::{Config, secure_channel};

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
    recv_error: watch::Receiver<Option<ua::StatusCode>>,
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

        let (send_pool, recv_pool) = if buffer_config.send_size == buffer_config.recv_size {
            let pool = Arc::new(BufferPool::new(10, buffer_config.send_size));
            (pool.clone(), pool)
        } else {
            let send_pool = Arc::new(BufferPool::new(2, buffer_config.send_size));
            let recv_pool = Arc::new(BufferPool::new(8, buffer_config.recv_size));
            (send_pool, recv_pool)
        };

        let sp = AnySecurityPolicy::None(SecurityPolicyNone);
        let (sc_send, sc_recv) = secure_channel::open(
            send,
            recv,
            send_pool,
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
        let (error_tx, recv_error) = watch::channel(None);

        tokio::spawn(run_secure_channel_recv(
            sc_recv,
            recv_pool.clone(),
            rx,
            error_tx,
        ));

        Ok(Self {
            sc_send,
            decoding_ctx,
            tx,
            recv_error,
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
            if let Some(err) = *self.recv_error.borrow() {
                return Err(err);
            }
            let req_id = sc_send.0;
            sc_send.0 += 1;
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
                if let Some(status) = *self.recv_error.borrow() {
                    status
                } else {
                    eprintln!("Failed to register response: {}", err);
                    ua::StatusCode::BadInternalError
                }
            })?;

        let mut buf = resp_rx.await.map_err(|err| {
            if let Some(status) = *self.recv_error.borrow() {
                status
            } else {
                eprintln!("Failed to receive response: {}", err);
                ua::StatusCode::BadInternalError
            }
        })??;

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
    error_tx: watch::Sender<Option<ua::StatusCode>>,
) where
    R: transport::RecvData + Send,
{
    let mut response_cache: SmallMap<u32, Result<bytes::BytesMut, ua::StatusCode>> =
        SmallMap::with_capacity(8);
    let mut pending_requests: SmallMap<
        u32,
        oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
    > = SmallMap::with_capacity(8);

    loop {
        // Keep partial reads alive while processing response registrations.
        let recv = async {
            let mut buf = recv_pool.acquire().await;
            let recv_res = recv_half.recv_msg(&mut buf).await;
            (recv_res, buf)
        };
        tokio::pin!(recv);

        let (recv_res, buf) = loop {
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
                        None => {
                            return;
                        }
                    }
                }
                result = &mut recv => break result,
            }
        };

        match recv_res {
            Ok(req_id) => {
                if let Some(tx) = pending_requests.remove(&req_id) {
                    let _ = tx.send(Ok(buf));
                } else {
                    response_cache.insert(req_id, Ok(buf));
                }
            }
            Err(err) => {
                error_tx.send_replace(Some(err));
                rx.close();
                for tx in pending_requests.into_values() {
                    let _ = tx.send(Err(err));
                }
                while let Ok(RecvCommand::GetResponse(req_id, tx)) = rx.try_recv() {
                    let result = response_cache.remove(&req_id).unwrap_or(Err(err));
                    let _ = tx.send(result);
                }
                recv_pool.release(buf).await;
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ua_core::msg::uasc;
    use ua_core::transport::RecvData;

    use super::*;

    struct ControlledRecv {
        data: mpsc::UnboundedReceiver<bytes::BytesMut>,
        consumed: mpsc::UnboundedSender<()>,
    }

    impl RecvData for ControlledRecv {
        async fn recv_data(
            &mut self,
            buf: &mut bytes::BytesMut,
        ) -> Result<usize, tokio::io::Error> {
            match self.data.recv().await {
                Some(data) => {
                    let len = data.len();
                    buf.extend_from_slice(&data);
                    self.consumed.send(()).unwrap();
                    Ok(len)
                }
                None => Ok(0),
            }
        }
    }

    struct NeverSend;

    impl transport::SendData for NeverSend {
        async fn send_data(&mut self, _data: &bytes::BytesMut) -> Result<(), tokio::io::Error> {
            panic!("request was sent after receive failure");
        }
    }

    struct FailWithQueuedRequests {
        commands: mpsc::UnboundedSender<RecvCommand>,
        first_response: Option<bytes::BytesMut>,
        responses: Vec<(
            u32,
            oneshot::Sender<Result<bytes::BytesMut, ua::StatusCode>>,
        )>,
    }

    impl RecvData for FailWithQueuedRequests {
        async fn recv_data(
            &mut self,
            buf: &mut bytes::BytesMut,
        ) -> Result<usize, tokio::io::Error> {
            if let Some(data) = self.first_response.take() {
                let len = data.len();
                buf.extend_from_slice(&data);
                return Ok(len);
            }
            for (id, tx) in self.responses.drain(..) {
                self.commands
                    .send(RecvCommand::GetResponse(id, tx))
                    .unwrap();
            }
            Ok(0)
        }
    }

    fn response(request_id: u32, sequence_number: u32, body: &[u8]) -> bytes::BytesMut {
        let mut buf = bytes::BytesMut::new();
        uasc::Header {
            msg_type: uasc::MsgType::Msg,
            is_final: uasc::IsFinal::Final,
            msg_size: uasc::Header::LEN
                + uasc::SymmetricSecurityHeader::LEN
                + uasc::SequenceHeader::LEN
                + body.len() as u32,
            secure_channel_id: 0,
        }
        .encode(&mut buf);
        uasc::SymmetricSecurityHeader { token_id: 0 }.encode(&mut buf);
        uasc::SequenceHeader {
            sequence_number,
            request_id,
        }
        .encode(&mut buf);
        buf.extend_from_slice(body);
        buf
    }

    #[tokio::test]
    async fn registration_does_not_cancel_a_partially_received_response() {
        tokio::time::timeout(Duration::from_secs(2), async {
            let (data_tx, data_rx) = mpsc::unbounded_channel();
            let (consumed_tx, mut consumed_rx) = mpsc::unbounded_channel();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
            let (error_tx, _error_rx) = watch::channel(None);
            let pool = Arc::new(BufferPool::new(2, 128));
            let recv = secure_channel::RecvHalf::new(
                RecvChunk::new(
                    ControlledRecv {
                        data: data_rx,
                        consumed: consumed_tx,
                    },
                    128,
                ),
                pool.clone(),
                SecurityPolicyNone,
            );
            let task = tokio::spawn(run_secure_channel_recv(recv, pool, cmd_rx, error_tx));

            data_tx.send(response(1, 1, b"first")).unwrap();
            consumed_rx.recv().await.unwrap();
            let mut second = response(2, 2, b"second");
            data_tx.send(second.split_to(10)).unwrap();
            consumed_rx.recv().await.unwrap();

            let (first_tx, first_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(1, first_tx)).unwrap();
            assert_eq!(&first_rx.await.unwrap().unwrap()[..], b"first");

            let (second_tx, second_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(2, second_tx)).unwrap();
            data_tx.send(second).unwrap();
            assert_eq!(&second_rx.await.unwrap().unwrap()[..], b"second");

            drop(cmd_tx);
            task.await.unwrap();
        })
        .await
        .expect("receive task did not complete");
    }

    #[tokio::test]
    async fn routes_coalesced_out_of_order_responses() {
        tokio::time::timeout(Duration::from_secs(2), async {
            let (data_tx, data_rx) = mpsc::unbounded_channel();
            let (consumed_tx, _consumed_rx) = mpsc::unbounded_channel();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
            let (error_tx, _error_rx) = watch::channel(None);
            let pool = Arc::new(BufferPool::new(2, 128));
            let recv = secure_channel::RecvHalf::new(
                RecvChunk::new(
                    ControlledRecv {
                        data: data_rx,
                        consumed: consumed_tx,
                    },
                    128,
                ),
                pool.clone(),
                SecurityPolicyNone,
            );
            let task = tokio::spawn(run_secure_channel_recv(recv, pool, cmd_rx, error_tx));
            let (first_tx, first_rx) = oneshot::channel();
            let (second_tx, second_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(1, first_tx)).unwrap();
            cmd_tx.send(RecvCommand::GetResponse(2, second_tx)).unwrap();

            let mut messages = response(2, 1, b"second");
            messages.extend_from_slice(&response(1, 2, b"first"));
            data_tx.send(messages).unwrap();
            assert_eq!(&first_rx.await.unwrap().unwrap()[..], b"first");
            assert_eq!(&second_rx.await.unwrap().unwrap()[..], b"second");

            drop(cmd_tx);
            task.await.unwrap();
        })
        .await
        .expect("responses were not routed");
    }

    #[tokio::test]
    async fn receive_failure_is_reported_to_pending_and_future_requests() {
        tokio::time::timeout(Duration::from_secs(2), async {
            let (data_tx, data_rx) = mpsc::unbounded_channel();
            let (consumed_tx, mut consumed_rx) = mpsc::unbounded_channel();
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
            let (error_tx, error_rx) = watch::channel(None);
            let pool = Arc::new(BufferPool::new(2, 128));
            let recv = secure_channel::RecvHalf::new(
                RecvChunk::new(
                    ControlledRecv {
                        data: data_rx,
                        consumed: consumed_tx,
                    },
                    128,
                ),
                pool.clone(),
                SecurityPolicyNone,
            );
            let task = tokio::spawn(run_secure_channel_recv(recv, pool, cmd_rx, error_tx));

            data_tx.send(response(1, 1, b"first")).unwrap();
            consumed_rx.recv().await.unwrap();
            let (pending_tx, pending_rx) = oneshot::channel();
            cmd_tx
                .send(RecvCommand::GetResponse(2, pending_tx))
                .unwrap();
            let (first_tx, first_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(1, first_tx)).unwrap();
            assert_eq!(&first_rx.await.unwrap().unwrap()[..], b"first");

            drop(data_tx);
            assert_eq!(
                pending_rx.await.unwrap(),
                Err(ua::StatusCode::BadConnectionClosed)
            );
            task.await.unwrap();
            assert_eq!(
                *error_rx.borrow(),
                Some(ua::StatusCode::BadConnectionClosed)
            );
            assert!(cmd_tx.is_closed());
        })
        .await
        .expect("receive task did not report failure");
    }

    #[tokio::test]
    async fn receive_failure_preserves_cached_and_fails_other_queued_requests() {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (response_tx1, response_rx1) = oneshot::channel();
        let (response_tx2, response_rx2) = oneshot::channel();
        let (error_tx, _error_rx) = watch::channel(None);
        let pool = Arc::new(BufferPool::new(2, 128));
        let recv = secure_channel::RecvHalf::new(
            RecvChunk::new(
                FailWithQueuedRequests {
                    commands: cmd_tx,
                    first_response: Some(response(1, 1, b"cached")),
                    responses: vec![(1, response_tx1), (2, response_tx2)],
                },
                128,
            ),
            pool.clone(),
            SecurityPolicyNone,
        );

        run_secure_channel_recv(recv, pool, cmd_rx, error_tx).await;
        assert_eq!(&response_rx1.await.unwrap().unwrap()[..], b"cached");
        assert_eq!(
            response_rx2.await.unwrap(),
            Err(ua::StatusCode::BadConnectionClosed)
        );
    }

    #[tokio::test]
    async fn subsequent_request_returns_receive_failure_without_sending() {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (error_tx, recv_error) = watch::channel(Some(ua::StatusCode::BadConnectionClosed));
        let pool = Arc::new(BufferPool::new(2, 128));
        let send = secure_channel::SendHalf::new(
            SendChunk::new(NeverSend, 128),
            pool.clone(),
            AnySecurityPolicy::None(SecurityPolicyNone),
        );
        let client = ScBinClient {
            sc_send: Mutex::new((2, send)),
            decoding_ctx: binary::DecodingCtx::default(),
            tx,
            recv_error,
            recv_pool: pool,
        };

        let result = client.send_request(ua::FindServersRequest::default()).await;
        assert!(matches!(result, Err(ua::StatusCode::BadConnectionClosed)));
        drop(error_tx);
    }
}
