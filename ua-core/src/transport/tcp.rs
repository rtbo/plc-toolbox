use std::mem::MaybeUninit;

use bytes::{BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

use crate::ua;

const HEADER_LEN: usize = 8;

/// A bounded view into the unused capacity of a transport send buffer.
pub type SendBuffer<'a> = &'a mut [MaybeUninit<u8>];

/// A TCP transport for raw OPC UA message chunks (UACP or UACS).
#[derive(Debug)]
pub struct Transport {
    stream: net::TcpStream,
    send_buf: BytesMut,
    send_buf_size: usize,
    recv_buf: BytesMut,
    recv_buf_size: usize,
    recv_buf_start: usize,
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
        let send_buf = BytesMut::with_capacity(send_buf_size);
        let recv_buf = BytesMut::with_capacity(recv_buf_size);
        Ok(Transport {
            stream,
            send_buf,
            send_buf_size,
            recv_buf,
            recv_buf_size,
            recv_buf_start: 0,
            recv_cached_size: None,
        })
    }

    pub fn max_send_size(&self) -> usize {
        self.send_buf_size
    }

    pub fn max_recv_size(&self) -> usize {
        self.recv_buf_size
    }

    pub fn adjust_buffer_sizes(&mut self, send_size: usize, recv_size: usize) {
        if send_size != self.send_buf.capacity() {
            self.send_buf = BytesMut::with_capacity(send_size);
            self.send_buf_size = send_size;
        }
        if recv_size != self.recv_buf.capacity() {
            self.recv_buf = BytesMut::with_capacity(recv_size);
            self.recv_buf_size = recv_size;
            self.recv_buf_start = 0;
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
        F: FnOnce(&mut SendBuffer<'_>),
    {
        let max = self.max_send_size();
        if expected_size > max {
            return Err(ua::StatusCode::BadEncodingLimitsExceeded);
        }
        if expected_size < HEADER_LEN {
            return Err(ua::StatusCode::BadEncodingError);
        }

        let mut start = self.send_buf.len();
        if start > 0 && start + expected_size > max {
            self.stream
                .write_all(&self.send_buf)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            self.send_buf.clear();
            start = 0;
        }
        debug_assert!(expected_size <= self.send_buf.capacity());

        let written = {
            let spare = &mut self.send_buf.spare_capacity_mut()[..expected_size];
            // Initialize the view so even a callback that shortens the slice directly
            // cannot cause uninitialized bytes to be exposed when the buffer advances.
            spare.fill(MaybeUninit::new(0));
            let mut view: SendBuffer<'_> = spare;
            write(&mut view);
            expected_size - view.len()
        };
        // SAFETY: The entire exposed region was initialized above, and `written` is
        // bounded by `expected_size`, which is within the send buffer's spare capacity.
        unsafe {
            self.send_buf.advance_mut(written);
        }
        let size = self.send_buf.len() - start;
        let declared = match self.send_buf[start..].get(4..8) {
            Some(b) => u32::from_le_bytes(b.try_into().unwrap()) as usize,
            None => 0,
        };
        if size != expected_size || size != declared {
            self.send_buf.truncate(start);
            return Err(ua::StatusCode::BadEncodingError);
        }
        Ok(())
    }

    /// Sends all buffered chunks.
    pub async fn flush(&mut self) -> Result<(), ua::StatusCode> {
        if !self.send_buf.is_empty() {
            self.stream
                .write_all(&self.send_buf)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            self.send_buf.clear();
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
            let remaining = self.recv_buf.len() - self.recv_buf_start;
            self.recv_buf.copy_within(self.recv_buf_start.., 0);
            self.recv_buf.truncate(remaining);
            self.recv_buf_start = 0;
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
        if self.recv_buf.len() - next_start >= HEADER_LEN {
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
        // No allocation is done if n <= self.max_recv_size()
        self.recv_buf
            .reserve(self.max_recv_size().saturating_sub(self.recv_buf.len()));
        while self.recv_buf.len() < n {
            if self
                .stream
                .read_buf(&mut self.recv_buf)
                .await
                .map_err(|_| ua::StatusCode::BadCommunicationError)?
                == 0
            {
                return Err(ua::StatusCode::BadConnectionClosed);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Buf, BufMut, BytesMut};
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpListener, TcpStream};

    use super::{SendBuffer, Transport};
    use crate::ua;

    #[test]
    fn send_buffer_view_is_bounded_without_growing_storage() {
        let mut buffer = BytesMut::with_capacity(8);
        let capacity = buffer.capacity();

        let written = {
            let spare = &mut buffer.spare_capacity_mut()[..8];
            let mut view: SendBuffer<'_> = spare;
            view.put_slice(b"12345678");
            8 - view.len()
        };
        unsafe {
            buffer.advance_mut(written);
        }

        assert_eq!(&buffer[..], b"12345678");
        assert_eq!(buffer.capacity(), capacity);
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
            send_buf: BytesMut::with_capacity(16),
            send_buf_size: 16,
            recv_buf: BytesMut::with_capacity(16),
            recv_buf_size: 16,
            recv_buf_start: 0,
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
            send_buf: BytesMut::with_capacity(16),
            send_buf_size: 16,
            recv_buf: BytesMut::with_capacity(16),
            recv_buf_size: 16,
            recv_buf_start: 0,
            recv_cached_size: None,
        };
        writer.write_all(b"MSGF\x10\0\0\0").await.unwrap();

        assert!(matches!(
            transport.recv_msg_chunk().await,
            Err(ua::StatusCode::BadEncodingLimitsExceeded)
        ));
    }
}
