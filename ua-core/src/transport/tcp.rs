use bytes::{Buf, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use crate::msg::binary::{Decode, Encode};
use crate::msg::uacp::Msg;

const HEADER_LEN: usize = 8;
/// Maximum message size. Buffers are allocated once to this size.
const MAX_MESSAGE_SIZE: usize = 65536;

/// A TCP transport for OPC UA messages. 
pub struct Transport {
    stream: net::TcpStream,
    send_buf: BytesMut,
    recv_buf: BytesMut,
}

impl crate::Transport for Transport {
    async fn connect(endpoint: &str) -> Result<Self, super::Error> {
        if !endpoint.starts_with("opc.tcp://") {
            return Err(super::Error::UnsupportedProtocol(endpoint.to_string()));
        }
        let endpoint = &endpoint[10..]; // Remove "opc.tcp://"
        let stream = net::TcpStream::connect(endpoint).await?;
        let send_buf = BytesMut::with_capacity(MAX_MESSAGE_SIZE);
        let recv_buf = BytesMut::with_capacity(MAX_MESSAGE_SIZE);
        Ok(Transport {
            stream,
            send_buf,
            recv_buf,
        })
    }

    async fn send(&mut self, msg: Msg) -> Result<(), super::Error> {
        let size = msg.encoded_len() as usize;
        if size > MAX_MESSAGE_SIZE {
            return Err(super::Error::MessageSize {
                size,
                max: MAX_MESSAGE_SIZE,
            });
        }
        self.send_buf.clear();
        msg.encode(&mut self.send_buf);
        self.stream.write_all(&self.send_buf).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Msg, super::Error> {
        self.fill(HEADER_LEN).await?;
        let size = u32::from_le_bytes(self.recv_buf[4..8].try_into().unwrap()) as usize;
        if !(HEADER_LEN..=MAX_MESSAGE_SIZE).contains(&size) {
            return Err(super::Error::MessageSize {
                size,
                max: MAX_MESSAGE_SIZE,
            });
        }
        self.fill(size).await?;

        let mut data = &self.recv_buf[..size];
        let msg = Msg::decode(&mut data);
        // Consume the message even in case of decoding error to stay synchronized.
        self.recv_buf.advance(size);
        if self.recv_buf.is_empty() {
            // Set the cursor back to the beginning: the initial capacity is reused.
            self.recv_buf.clear();
        }
        Ok(msg?)
    }
}

impl Transport {
    /// Reads from the socket until at least `n` bytes are available in `recv_buf`.
    async fn fill(&mut self, n: usize) -> Result<(), super::Error> {
        // No allocation is done if n <= MAX_MESSAGE_SIZE
        self.recv_buf
            .reserve(MAX_MESSAGE_SIZE.saturating_sub(self.recv_buf.len()));
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
