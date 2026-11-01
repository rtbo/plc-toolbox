use bytes::{BufMut, BytesMut};
use std::mem::MaybeUninit;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net;

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
            send_buf_size,
            recv_buf,
            recv_buf_size,
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
    ) -> Result<(), super::Error>
    where
        F: FnOnce(&mut SendBuffer<'_>),
    {
        let max = self.max_send_size();
        if expected_size > max {
            return Err(super::Error::MessageSize {
                size: expected_size,
                max,
            });
        }
        if expected_size < HEADER_LEN {
            return Err(super::Error::MessageSize {
                size: expected_size,
                max,
            });
        }

        let mut start = self.send_buf.len();
        if start > max || expected_size > max.saturating_sub(start) {
            self.stream.write_all(&self.send_buf).await?;
            self.send_buf.clear();
            start = 0;
        }
        if self.send_buf.capacity().saturating_sub(start) < expected_size {
            return Err(super::Error::MessageSize {
                size: expected_size,
                max,
            });
        }

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
            return Err(super::Error::MessageSize { size, max });
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

#[cfg(test)]
mod tests {
    use bytes::{BufMut, BytesMut};

    use super::SendBuffer;

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
}
