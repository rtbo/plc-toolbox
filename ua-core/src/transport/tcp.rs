use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use crate::encoding::binary;
use crate::ua;

const HEADER_LEN: usize = 8;

/// A TCP transport for raw OPC UA message chunks (UACP or UACS).
#[derive(Debug)]
pub struct Transport {
    stream: net::TcpStream,
    send_buf: Vec<u8>,
    send_buf_start: usize,
    recv_buf: Vec<u8>,
    recv_buf_start: usize,
    recv_buf_end: usize,
    recv_cached_size: Option<usize>,
}

impl Transport {
    pub async fn connect(
        endpoint: &str,
        send_buf_size: usize,
        recv_buf_size: usize,
    ) -> Result<Self, ua::StatusCode> {
        if !endpoint.starts_with("opc.tcp://") {
            return Err(ua::StatusCode::BadInvalidArgument);
        }
        let endpoint = &endpoint[10..]; // Remove "opc.tcp://"
        let stream = net::TcpStream::connect(endpoint)
            .await
            .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
        let send_buf = vec![0u8; send_buf_size];
        let recv_buf = vec![0u8; recv_buf_size];
        Ok(Transport {
            stream,
            send_buf,
            send_buf_start: 0,
            recv_buf,
            recv_buf_start: 0,
            recv_buf_end: 0,
            recv_cached_size: None,
        })
    }

    pub fn max_send_size(&self) -> usize {
        self.send_buf.len()
    }

    pub fn max_recv_size(&self) -> usize {
        self.recv_buf.len()
    }

    pub fn adjust_buffer_sizes(&mut self, send_size: usize, recv_size: usize) {
        if send_size != self.send_buf.len() {
            self.send_buf.resize(send_size, 0);
            self.send_buf_start = 0;
        }
        if recv_size != self.recv_buf.len() {
            self.recv_buf.resize(recv_size, 0);
            self.recv_buf_start = 0;
            self.recv_buf_end = 0;
            self.recv_cached_size = None;
        }
    }

    /// Appends one chunk to the send buffer without sending it.
    ///
    /// `write` must append exactly `expected_size` bytes for one complete chunk (UACP or
    /// UACS), including its 8-byte header whose bytes 4..8 hold the total chunk length.
    /// The bounded view cannot grow the send buffer. If previously buffered chunks leave
    /// no room for the new one, they are flushed first.
    pub async fn send_msg_chunk<F>(
        &mut self,
        expected_size: usize,
        write: F,
    ) -> Result<(), ua::StatusCode>
    where
        F: FnOnce(&mut binary::EncodeBuf<'_>),
    {
        let max = self.max_send_size();
        if expected_size > max {
            return Err(ua::StatusCode::BadEncodingLimitsExceeded);
        }
        if expected_size < HEADER_LEN {
            return Err(ua::StatusCode::BadEncodingError);
        }

        if self.send_buf_start > 0 && self.send_buf_start + expected_size > max {
            self.stream
                .write_all(&self.send_buf[0..self.send_buf_start])
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            self.send_buf_start = 0;
        }

        let start = self.send_buf_start;
        debug_assert!(expected_size <= self.send_buf.len() - start);
        let written = {
            let mut view = &mut self.send_buf[start..start + expected_size];
            write(&mut view);
            expected_size - view.len()
        };
        let declared = self.send_buf[start + 4..start + 8]
            .try_into()
            .map(u32::from_le_bytes)
            .map(|n| n as usize)
            .unwrap_or(0);
        if written != expected_size || written != declared {
            return Err(ua::StatusCode::BadEncodingError);
        }
        self.send_buf_start += written;
        Ok(())
    }

    /// Sends all buffered chunks.
    pub async fn flush(&mut self) -> Result<(), ua::StatusCode> {
        if self.send_buf_start > 0 {
            self.stream
                .write_all(&self.send_buf[0..self.send_buf_start])
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            self.send_buf_start = 0;
        }
        self.stream
            .flush()
            .await
            .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
        Ok(())
    }

    /// Returns the next entire chunk as-is (header included).
    ///
    /// The returned slice borrows the receive buffer and implements `bytes::Buf`.
    /// Any bytes read beyond this chunk remain buffered for the next call.
    pub async fn recv_msg_chunk(&mut self) -> Result<&[u8], ua::StatusCode> {
        if self.recv_buf_start > 0 {
            let (start, end) = (self.recv_buf_start, self.recv_buf_end);
            self.recv_buf.copy_within(start..end, 0);
            self.recv_buf_start = 0;
            self.recv_buf_end = end - start;
        }

        let max_recv_size = self.max_recv_size();
        let cached_size = self.recv_cached_size.take();
        let size = if let Some(size) = cached_size {
            size
        } else {
            self.fill(HEADER_LEN).await?;
            u32::from_le_bytes(self.recv_buf[4..8].try_into().unwrap()) as usize
        };
        if !(HEADER_LEN..max_recv_size).contains(&size) {
            return Err(ua::StatusCode::BadEncodingLimitsExceeded);
        }
        self.fill(size).await?;

        let next_start = size;
        if self.recv_buf_end - next_start >= HEADER_LEN {
            let next_size = u32::from_le_bytes(
                self.recv_buf[next_start + 4..next_start + 8]
                    .try_into()
                    .unwrap(),
            ) as usize;
            if (HEADER_LEN..max_recv_size).contains(&next_size)
                && next_size <= max_recv_size.saturating_sub(size)
            {
                self.recv_cached_size = Some(next_size);
            }
        }

        self.recv_buf_start = size;
        Ok(&self.recv_buf[..size])
    }

    /// Reads from the socket until at least `n` bytes are available in `recv_buf`.
    async fn fill(&mut self, n: usize) -> Result<(), ua::StatusCode> {
        assert!(n <= self.recv_buf.len() - self.recv_buf_start);
        while (self.recv_buf_end - self.recv_buf_start) < n {
            let mut view = &mut self.recv_buf[self.recv_buf_end..];
            let written = self
                .stream
                .read_buf(&mut view)
                .await
                .map_err(|_| ua::StatusCode::BadCommunicationError)?;
            if written == 0 {
                return Err(ua::StatusCode::BadConnectionClosed);
            }
            self.recv_buf_end += written;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Buf;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    use super::Transport;
    use crate::ua;

    async fn pair(buf: usize) -> (Transport, TcpStream) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let writer = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let transport = Transport {
            stream,
            send_buf: vec![0u8; buf],
            send_buf_start: 0,
            recv_buf: vec![0u8; buf],
            recv_buf_start: 0,
            recv_buf_end: 0,
            recv_cached_size: None,
        };
        (transport, writer)
    }

    fn chunk(len: u8, fill: u8) -> Vec<u8> {
        let mut v = vec![fill; len as usize];
        v[4..8].copy_from_slice(&(len as u32).to_le_bytes());
        v
    }

    #[tokio::test]
    async fn keeps_partial_next_chunk_when_compacting() {
        let (mut t, mut w) = pair(32).await;
        let (a, b) = (chunk(10, 1), chunk(12, 2));
        let mut data = a.clone();
        data.extend_from_slice(&b[..5]); // partial header of next chunk
        w.write_all(&data).await.unwrap();
        assert_eq!(t.recv_msg_chunk().await.unwrap(), &a[..]);
        w.write_all(&b[5..]).await.unwrap();
        assert_eq!(t.recv_msg_chunk().await.unwrap(), &b[..]);
    }

    #[tokio::test]
    async fn receives_chunk_after_buffer_fully_consumed() {
        let (mut t, mut w) = pair(32).await;
        for (len, fill) in [(10u8, 1u8), (12, 2), (9, 3)] {
            let c = chunk(len, fill);
            w.write_all(&c).await.unwrap();
            assert_eq!(t.recv_msg_chunk().await.unwrap(), &c[..]);
        }
    }

    #[tokio::test]
    async fn receives_many_coalesced_chunks_in_order() {
        let (mut t, mut w) = pair(32).await;
        let chunks = [chunk(8, 1), chunk(11, 2), chunk(9, 3), chunk(8, 4)];
        w.write_all(&chunks.concat()).await.unwrap();
        for c in &chunks {
            assert_eq!(t.recv_msg_chunk().await.unwrap(), &c[..]);
        }
        assert_eq!(t.max_recv_size(), 32);
    }

    #[tokio::test]
    async fn receives_one_borrowed_chunk_and_keeps_the_next_buffered() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut writer = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut transport = Transport {
            stream,
            send_buf: vec![0u8; 16],
            send_buf_start: 0,
            recv_buf: vec![0u8; 16],
            recv_buf_start: 0,
            recv_buf_end: 0,
            recv_cached_size: None,
        };
        writer
            .write_all(&[
                b'M', b'S', b'G', b'F', 8, 0, 0, 0, b'M', b'S', b'G', b'F', 8, 0, 0, 0,
            ])
            .await
            .unwrap();

        {
            let mut first = transport.recv_msg_chunk().await.unwrap();
            assert_eq!(first.remaining(), 8);
            assert_eq!(first.get_u32_le(), u32::from_le_bytes(*b"MSGF"));
        }

        let second = transport.recv_msg_chunk().await.unwrap();
        assert_eq!(second, b"MSGF\x08\0\0\0");
    }

    #[tokio::test]
    async fn rejects_chunk_equal_to_max_receive_size() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let mut writer = TcpStream::connect(listener.local_addr().unwrap())
            .await
            .unwrap();
        let (stream, _) = listener.accept().await.unwrap();
        let mut transport = Transport {
            stream,
            send_buf: vec![0u8; 16],
            send_buf_start: 0,
            recv_buf: vec![0u8; 16],
            recv_buf_start: 0,
            recv_buf_end: 0,
            recv_cached_size: None,
        };
        writer.write_all(b"MSGF\x10\0\0\0").await.unwrap();

        assert!(matches!(
            transport.recv_msg_chunk().await,
            Err(ua::StatusCode::BadEncodingLimitsExceeded)
        ));
    }
}
