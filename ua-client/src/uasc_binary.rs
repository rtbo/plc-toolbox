//! Client implementation for the stack profiles based on UA Secure Conversation and Binary data encoding.
//!  - UA-TCP UA-SC UA-Binary
//!  - WSS UA-SC UA-Binary (TODO)
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock, mpsc, oneshot, watch};
use ua_core::encoding::binary::{self, Decode, Encode};
use ua_core::encoding::{self, BinaryEncodingId, Ns0BinaryEncodingId};
use ua_core::transport::{self, RecvChunk, SendChunk};
use ua_core::uacp::{self, Msg};
use ua_core::{BufferPool, SecurityPolicy, ua, uasc};

use crate::smallmap::SmallMap;
use crate::{Config, config, secure_channel};

impl config::Buffer {
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
            max_message_size: self.max_response_size.unwrap_or(0) as u32,
            max_chunk_count: self.max_recv_chunks().unwrap_or(0) as u32,
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
            max_response_size: self.max_response_size,
            max_request_size: if srv_ack.max_message_size > 0 {
                Some(srv_ack.max_message_size as usize)
            } else {
                None
            },
        })
    }
}

#[derive(Debug)]
pub struct Client<P, S> {
    // mutex containing the current request id and secure channel send half
    sc_send: Mutex<(u32, Option<secure_channel::SendHalf<P, S>>)>,
    config: Arc<Config>,
    decoding_ctx: RwLock<binary::DecodingCtx>,
    ns_array: RwLock<Vec<String>>,
    tx: mpsc::UnboundedSender<RecvCommand>,
    recv_error: watch::Receiver<Option<ua::StatusCode>>,
    recv_pool: Arc<BufferPool>,
}

#[derive(Debug, PartialEq)]
pub struct RecvResponse {
    buf: bytes::BytesMut,
    multi_chunks: bool,
    is_ok: bool,
}

#[derive(Debug)]
enum RecvCommand {
    GetResponse(u32, oneshot::Sender<Result<RecvResponse, ua::StatusCode>>),
    Close(oneshot::Sender<()>),
}

impl<P, S> Client<P, S>
where
    P: SecurityPolicy + Clone + Send + 'static,
    S: transport::SendData + Send,
{
    pub async fn connect<R>(
        config: Config,
        security_policy: P,
        send: S,
        recv: R,
    ) -> Result<Self, ua::StatusCode>
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

        let config = Arc::new(Config {
            buffer: buffer_config,
            ..config
        });

        let (send_pool, recv_pool) = if buffer_config.send_size == buffer_config.recv_size {
            let pool = Arc::new(BufferPool::new(10, buffer_config.send_size));
            (pool.clone(), pool)
        } else {
            let send_pool = Arc::new(BufferPool::new(2, buffer_config.send_size));
            let recv_pool = Arc::new(BufferPool::new(8, buffer_config.recv_size));
            (send_pool, recv_pool)
        };

        let (sc_send, sc_recv) = secure_channel::open(
            security_policy,
            send,
            recv,
            config.clone(),
            send_pool,
            recv_pool.clone(),
            1,
        )
        .await?;

        let sc_send = Mutex::new((2, Some(sc_send)));

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
            config,
            decoding_ctx: RwLock::new(decoding_ctx),
            ns_array: RwLock::new(Vec::new()),
            tx,
            recv_error,
            recv_pool,
        })
    }

    /// Send the CloseSecureChannel request to the server,
    /// and drop the two halves of the secure channel.
    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        let mut sc_send = self.sc_send.lock().await;

        if let Some(err) = *self.recv_error.borrow() {
            return Err(err);
        }

        let request_id = sc_send.0;
        let mut send = sc_send
            .1
            .take()
            .ok_or(ua::StatusCode::BadConnectionClosed)?;
        send.send_close(
            request_id,
            ua::CloseSecureChannelRequest {
                request_header: ua::RequestHeader {
                    timestamp: ua::DateTime::now(),
                    ..Default::default()
                },
            },
        )
        .await?;

        let (close_tx, close_rx) = oneshot::channel();
        let _ = self.tx.send(RecvCommand::Close(close_tx));
        let _ = close_rx.await;

        Ok(())
    }

    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    pub async fn get_ns_array(&self) -> Result<Vec<String>, ua::StatusCode> {
        Ok(self.ns_array.read().await.clone())
    }

    pub async fn set_ns_array(&self, ns_array: Vec<String>) {
        *self.ns_array.write().await = ns_array;
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: encoding::Request + binary::Encode + encoding::BinaryEncodingId + encoding::DataTypeId,
        R::Response: binary::Decode + encoding::BinaryEncodingId,
    {
        let ns_index = self
            .ns_index::<R>()
            .await
            .expect("Namespace index not found for the request type");
        let encoding = R::binary_encoding_node_id(ns_index);
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
                .as_mut()
                .ok_or(ua::StatusCode::BadConnectionClosed)?
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

        let resp = resp_rx.await.map_err(|err| {
            if let Some(status) = *self.recv_error.borrow() {
                status
            } else {
                eprintln!("Failed to receive response: {}", err);
                ua::StatusCode::BadInternalError
            }
        })??;

        let decoding_ctx = self.decoding_ctx.read().await;

        if resp.is_ok {
            let mut buf = resp.buf;

            let encoding = ua::NodeId::decode(&mut buf, &decoding_ctx)?;

            let result = match encoding {
                ua::ServiceFault::BINARY_ENCODING_NODE_ID => {
                    let resp = ua::ServiceFault::decode(&mut buf, &decoding_ctx)?;
                    Err(resp.response_header.service_result)
                }
                _ if encoding == R::Response::binary_encoding_node_id(ns_index) => {
                    Ok(R::Response::decode(&mut buf, &decoding_ctx)?)
                }
                _ => Err(ua::StatusCode::BadNodeIdInvalid),
            };

            // We don't recycle buffers for multi-chunk responses because they are quite rare
            // and will potentially permanently consume large amount of memory if recycled.
            if !resp.multi_chunks {
                self.recv_pool.release(buf).await;
            }

            result
        } else {
            let mut buf = resp.buf;
            let error_msg = uasc::ErrorMsg::decode(&mut buf, &decoding_ctx)?;
            self.recv_pool.release(buf).await;
            Err(error_msg.error_code)
        }
    }

    async fn ns_index<R: encoding::DataTypeId>(&self) -> Option<u16> {
        if R::NS_INDEX.is_some() {
            return R::NS_INDEX;
        }
        self.ns_array
            .read()
            .await
            .iter()
            .skip(1) // NS0 has always NS_INDEX set
            .position(|s| s == R::NS_URI)
            .map(|i| (i + 1) as u16)
    }
}

async fn run_secure_channel_recv<P, R>(
    mut recv_half: secure_channel::RecvHalf<P, R>,
    recv_pool: Arc<BufferPool>,
    mut rx: mpsc::UnboundedReceiver<RecvCommand>,
    error_tx: watch::Sender<Option<ua::StatusCode>>,
) where
    P: SecurityPolicy,
    R: transport::RecvData + Send,
{
    let mut response_cache: SmallMap<u32, Result<RecvResponse, ua::StatusCode>> =
        SmallMap::with_capacity(8);
    let mut partial_responses: SmallMap<u32, bytes::BytesMut> = SmallMap::with_capacity(2);
    let mut pending_requests: SmallMap<u32, oneshot::Sender<Result<RecvResponse, ua::StatusCode>>> =
        SmallMap::with_capacity(8);

    loop {
        // Keep partial reads alive while processing response registrations.
        let recv = async {
            let mut buf = recv_pool.acquire().await;
            let recv_res = recv_half.recv_msg(&mut buf).await;
            (recv_res, buf)
        };
        tokio::pin!(recv);

        let (recv_res, mut buf) = loop {
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
                        Some(RecvCommand::Close(tx)) => {
                            let _ = tx.send(());
                            return;
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
            Ok(info) if info.is_final == uasc::IsFinal::Abort => {
                if let Some(partial) = partial_responses.remove(&info.request_id) {
                    if partial.capacity() == recv_pool.buf_capacity() {
                        recv_pool.release(partial).await;
                    }
                }
                let resp = RecvResponse {
                    buf,
                    multi_chunks: false,
                    is_ok: false,
                };
                if let Some(tx) = pending_requests.remove(&info.request_id) {
                    let _ = tx.send(Ok(resp));
                } else {
                    response_cache.insert(info.request_id, Ok(resp));
                }
            }
            Ok(info) if info.is_final == uasc::IsFinal::Continue => {
                if let Some(partial) = partial_responses.get_mut(&info.request_id) {
                    partial.extend_from_slice(&buf);
                    recv_pool.release(buf).await;
                } else {
                    partial_responses.insert(info.request_id, buf);
                }
            }
            Ok(info) => {
                debug_assert!(info.is_final == uasc::IsFinal::Final);
                let multi_chunks =
                    if let Some(mut partial) = partial_responses.remove(&info.request_id) {
                        partial.extend_from_slice(&buf);
                        recv_pool.release(buf).await;
                        buf = partial;
                        true
                    } else {
                        false
                    };

                let resp = RecvResponse {
                    buf,
                    multi_chunks,
                    is_ok: true,
                };
                if let Some(tx) = pending_requests.remove(&info.request_id) {
                    let _ = tx.send(Ok(resp));
                } else {
                    response_cache.insert(info.request_id, Ok(resp));
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

    use ua_core::transport::RecvData;
    use ua_core::{SecurityPolicyNone, uasc};

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
        responses: Vec<(u32, oneshot::Sender<Result<RecvResponse, ua::StatusCode>>)>,
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
        response_chunk(request_id, sequence_number, uasc::IsFinal::Final, body)
    }

    fn response_chunk(
        request_id: u32,
        sequence_number: u32,
        is_final: uasc::IsFinal,
        body: &[u8],
    ) -> bytes::BytesMut {
        let mut buf = bytes::BytesMut::new();
        uasc::Header {
            msg_type: uasc::MsgType::Msg,
            is_final,
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
    async fn reassembles_interleaved_responses_only_after_final_chunks() {
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
            let (first_tx, mut first_rx) = oneshot::channel();
            let (second_tx, mut second_rx) = oneshot::channel();
            let (barrier_tx, barrier_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(1, first_tx)).unwrap();
            cmd_tx.send(RecvCommand::GetResponse(2, second_tx)).unwrap();
            cmd_tx
                .send(RecvCommand::GetResponse(3, barrier_tx))
                .unwrap();

            let mut chunks = response_chunk(1, 1, uasc::IsFinal::Continue, b"first ");
            chunks.extend_from_slice(&response_chunk(2, 2, uasc::IsFinal::Continue, b"other "));
            chunks.extend_from_slice(&response_chunk(1, 3, uasc::IsFinal::Continue, b"middle "));
            chunks.extend_from_slice(&response(3, 4, b"barrier"));
            data_tx.send(chunks).unwrap();
            assert_eq!(&barrier_rx.await.unwrap().unwrap().buf[..], b"barrier");
            assert!(matches!(
                first_rx.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));
            assert!(matches!(
                second_rx.try_recv(),
                Err(oneshot::error::TryRecvError::Empty)
            ));

            let mut chunks = response(2, 5, b"end");
            chunks.extend_from_slice(&response(1, 6, b"end"));
            data_tx.send(chunks).unwrap();
            let first = first_rx.await.unwrap().unwrap();
            let second = second_rx.await.unwrap().unwrap();
            assert!(first.is_ok);
            assert_eq!(&first.buf[..], b"first middle end");
            assert!(second.is_ok);
            assert_eq!(&second.buf[..], b"other end");

            let (barrier_tx, barrier_rx) = oneshot::channel();
            cmd_tx
                .send(RecvCommand::GetResponse(3, barrier_tx))
                .unwrap();
            let mut chunks = response_chunk(4, 7, uasc::IsFinal::Continue, b"cached ");
            chunks.extend_from_slice(&response(4, 8, b"response"));
            chunks.extend_from_slice(&response(3, 9, b"barrier"));
            data_tx.send(chunks).unwrap();
            barrier_rx.await.unwrap().unwrap();
            let (tx, rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(4, tx)).unwrap();
            let cached = rx.await.unwrap().unwrap();
            assert!(cached.is_ok);
            assert_eq!(&cached.buf[..], b"cached response");

            drop(cmd_tx);
            task.await.unwrap();
        })
        .await
        .expect("chunked responses were not reassembled");
    }

    #[tokio::test]
    async fn abort_discards_partial_chunks_and_preserves_other_responses() {
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

            let error = uasc::ErrorMsg {
                error_code: ua::StatusCode::BadInternalError,
                reason: Some("aborted".to_owned()),
            };
            let mut error_body = bytes::BytesMut::new();
            error.encode(&mut error_body);

            let (other_tx, other_rx) = oneshot::channel();
            let (barrier_tx, barrier_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(2, other_tx)).unwrap();
            cmd_tx
                .send(RecvCommand::GetResponse(3, barrier_tx))
                .unwrap();
            let mut chunks = response_chunk(1, 1, uasc::IsFinal::Continue, b"discard ");
            chunks.extend_from_slice(&response_chunk(2, 2, uasc::IsFinal::Continue, b"keep "));
            chunks.extend_from_slice(&response_chunk(1, 3, uasc::IsFinal::Continue, b"this"));
            chunks.extend_from_slice(&response_chunk(1, 4, uasc::IsFinal::Abort, &error_body));
            chunks.extend_from_slice(&response_chunk(4, 5, uasc::IsFinal::Abort, &error_body));
            chunks.extend_from_slice(&response(2, 6, b"this"));
            chunks.extend_from_slice(&response(3, 7, b"barrier"));
            data_tx.send(chunks).unwrap();
            assert_eq!(&barrier_rx.await.unwrap().unwrap().buf[..], b"barrier");

            for id in [1, 4] {
                let (tx, rx) = oneshot::channel();
                cmd_tx.send(RecvCommand::GetResponse(id, tx)).unwrap();
                let mut aborted = rx.await.unwrap().unwrap();
                assert!(!aborted.is_ok);
                assert_eq!(&aborted.buf[..], &error_body[..]);
                assert_eq!(
                    uasc::ErrorMsg::decode(&mut aborted.buf, &binary::DecodingCtx::new()).unwrap(),
                    error
                );
                assert!(aborted.buf.is_empty());
            }
            let other = other_rx.await.unwrap().unwrap();
            assert!(other.is_ok);
            assert_eq!(&other.buf[..], b"keep this");

            let (tx, rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(1, tx)).unwrap();
            data_tx.send(response(1, 8, b"fresh")).unwrap();
            let fresh = rx.await.unwrap().unwrap();
            assert!(fresh.is_ok);
            assert_eq!(&fresh.buf[..], b"fresh");

            let (pending_tx, pending_rx) = oneshot::channel();
            let (barrier_tx, barrier_rx) = oneshot::channel();
            cmd_tx
                .send(RecvCommand::GetResponse(5, pending_tx))
                .unwrap();
            cmd_tx
                .send(RecvCommand::GetResponse(6, barrier_tx))
                .unwrap();
            data_tx.send(response(6, 9, b"barrier")).unwrap();
            assert_eq!(&barrier_rx.await.unwrap().unwrap().buf[..], b"barrier");
            let mut chunks = response_chunk(5, 10, uasc::IsFinal::Continue, b"discard");
            chunks.extend_from_slice(&response_chunk(5, 11, uasc::IsFinal::Abort, &error_body));
            data_tx.send(chunks).unwrap();
            let mut aborted = pending_rx.await.unwrap().unwrap();
            assert!(!aborted.is_ok);
            assert_eq!(&aborted.buf[..], &error_body[..]);
            assert_eq!(
                uasc::ErrorMsg::decode(&mut aborted.buf, &binary::DecodingCtx::new()).unwrap(),
                error
            );

            drop(cmd_tx);
            task.await.unwrap();
        })
        .await
        .expect("abort responses were not handled");
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
            let resp = first_rx.await.unwrap().unwrap();
            assert_eq!(&resp.buf[..], b"first");
            assert!(resp.is_ok);

            let (second_tx, second_rx) = oneshot::channel();
            cmd_tx.send(RecvCommand::GetResponse(2, second_tx)).unwrap();
            data_tx.send(second).unwrap();
            let resp = second_rx.await.unwrap().unwrap();
            assert_eq!(&resp.buf[..], b"second");
            assert!(resp.is_ok);

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
            let resp = first_rx.await.unwrap().unwrap();
            assert_eq!(&resp.buf[..], b"first");
            assert!(resp.is_ok);
            let resp = second_rx.await.unwrap().unwrap();
            assert_eq!(&resp.buf[..], b"second");
            assert!(resp.is_ok);

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

            let resp = first_rx.await.unwrap().unwrap();
            assert_eq!(&resp.buf[..], b"first");
            assert!(resp.is_ok);

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
        let resp = response_rx1.await.unwrap().unwrap();
        assert_eq!(&resp.buf[..], b"cached");
        assert!(resp.is_ok);
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
        let config = Arc::new(crate::Config::default());
        let send = secure_channel::SendHalf::new(
            SendChunk::new(NeverSend, 128),
            config.clone(),
            pool.clone(),
            SecurityPolicyNone,
        );
        let decoding_ctx = RwLock::new(binary::DecodingCtx::default());
        let ns_array = RwLock::new(Vec::new());

        let client = Client {
            sc_send: Mutex::new((2, Some(send))),
            config,
            decoding_ctx,
            ns_array,
            tx,
            recv_error,
            recv_pool: pool,
        };

        let result = client.send_request(ua::FindServersRequest::default()).await;
        assert!(matches!(result, Err(ua::StatusCode::BadConnectionClosed)));
        drop(error_tx);
    }
}
