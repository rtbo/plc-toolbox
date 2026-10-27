use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

const HEADER_LEN: usize = 8;

/// A TCP transport for raw OPC UA message chunks (UACP or UACS).
#[derive(Debug)]
pub struct Transport {
    stream: net::TcpStream,
    send_buf: BytesMut,
    recv_buf: BytesMut,
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

    /// Appends one chunk to the send buffer without sending it.
    ///
    /// `write` must append exactly one complete chunk (UACP or UACS) directly to the
    /// buffer, including its 8-byte header whose bytes 4..8 hold the total chunk length.
    /// The transport does not interpret anything else, so the chunk may be encrypted.
    /// If previously buffered chunks leave no room for the new one, they are flushed first.
    /// On error, the buffer is left unchanged.
    pub async fn send_msg_chunk<F>(&mut self, write: F) -> Result<(), super::Error>
    where
        F: FnOnce(&mut BytesMut),
    {
        let start = self.send_buf.len();
        write(&mut self.send_buf);
        let max = self.max_send_size();
        let size = self.send_buf.len() - start;
        let declared = match self.send_buf[start..].get(4..8) {
            Some(b) => u32::from_le_bytes(b.try_into().unwrap()) as usize,
            None => 0,
        };
        if size < HEADER_LEN || size != declared || size > max {
            self.send_buf.truncate(start);
            return Err(super::Error::MessageSize { size, max });
        }
        if start > 0 && start + size > max {
            // Send the previous chunks, then move the new one to the front of the buffer.
            self.stream.write_all(&self.send_buf[..start]).await?;
            self.send_buf.copy_within(start.., 0);
            self.send_buf.truncate(size);
        }
        Ok(())
    }

    /// Sends all buffered chunks.
    pub async fn flush(&mut self) -> Result<(), super::Error> {
        if !self.send_buf.is_empty() {
            self.stream.write_all(&self.send_buf).await?;
            self.send_buf.clear();
        }
        self.stream.flush().await?;
        Ok(())
    }

    /// Returns the next entire chunk as-is (header included).
    ///
    /// Several chunks may be received at once; they are handed out one by one.
    /// The returned buffer is split off without copying, so it can be decoded or
    /// decrypted in place by the upper layers.
    pub async fn recv_msg_chunk(&mut self) -> Result<BytesMut, super::Error> {
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
        Ok(self.recv_buf.split_to(size))
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
