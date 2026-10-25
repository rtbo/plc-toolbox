use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use crate::encoding::binary::{self, Decode, Encode};
use crate::uacp::{self, Msg};

const HEADER_LEN: usize = 8;

/// A TCP transport for OPC UA messages.
pub struct Transport {
    stream: net::TcpStream,
    send_buf: BytesMut,
    recv_buf: BytesMut,
    decoding_ctx: binary::DecodingCtx
}

impl Transport {
    pub async fn connect(
        endpoint: &str,
        send_buf_size: usize,
        recv_buf_size: usize,
    ) -> Result<Self, super::Error> {
        if !endpoint.starts_with("opc.tcp://") {
            return Err(super::Error::UnsupportedProtocol(endpoint.to_string()));
        }
        let endpoint = &endpoint[10..]; // Remove "opc.tcp://"
        let stream = net::TcpStream::connect(endpoint).await?;
        let send_buf = BytesMut::with_capacity(send_buf_size);
        let recv_buf = BytesMut::with_capacity(recv_buf_size);
        Ok(Transport {
            stream,
            send_buf,
            recv_buf,
            decoding_ctx: Default::default(),
        })
    }

    fn max_send_size(&self) -> usize {
        self.send_buf.capacity()
    }

    fn max_recv_size(&self) -> usize {
        self.recv_buf.capacity()
    }

    pub fn adjust_buffer_sizes(&mut self, send_size: usize, recv_size: usize) {
        if send_size != self.send_buf.capacity() {
            self.send_buf = BytesMut::with_capacity(send_size);
        }
        if recv_size != self.recv_buf.capacity() {
            self.recv_buf = BytesMut::with_capacity(recv_size);
        }
    }

    pub async fn send_uacp<M>(&mut self, msg: uacp::Msg<M>) -> Result<(), super::Error>
    where
        M: uacp::MsgType + Encode,
    {
        let size = msg.encoded_len() as usize;
        let max_send_size = self.max_send_size();
        if size > max_send_size {
            return Err(super::Error::MessageSize {
                size,
                max: max_send_size,
            });
        }
        self.send_buf.clear();
        msg.encode(&mut self.send_buf);
        self.stream.write_all(&self.send_buf).await?;
        Ok(())
    }

    pub async fn recv_uacp<M>(&mut self) -> Result<Msg<M>, super::Error>
    where
        M: uacp::MsgType + Decode,
    {
        self.fill(HEADER_LEN).await?;
        let size = u32::from_le_bytes(self.recv_buf[4..8].try_into().unwrap()) as usize;
        let max_recv_size = self.max_recv_size();
        if !(HEADER_LEN..=max_recv_size).contains(&size) {
            return Err(super::Error::MessageSize {
                size,
                max: max_recv_size,
            });
        }
        self.fill(size).await?;

        let mut data = &self.recv_buf[..size];
        let msg = Msg::decode(&mut data, &self.decoding_ctx);
        // Consume the message even in case of decoding error to stay synchronized.
        self.recv_buf.advance(size);
        if self.recv_buf.is_empty() {
            // Set the cursor back to the beginning: the initial capacity is reused.
            self.recv_buf.clear();
        }
        Ok(msg?)
    }

    /// Reads from the socket until at least `n` bytes are available in `recv_buf`.
    async fn fill(&mut self, n: usize) -> Result<(), super::Error> {
        // No allocation is done if n <= self.max_recv_size()
        self.recv_buf
            .reserve(self.max_recv_size().saturating_sub(self.recv_buf.len()));
        while self.recv_buf.len() < n {
            if self.stream.read_buf(&mut self.recv_buf).await? == 0 {
                return Err(super::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Connection closed",
                )));
            }
        }
        Ok(())
    }
}
