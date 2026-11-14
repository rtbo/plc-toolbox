use bytes::Buf;
use tokio::io;

use crate::ua;

pub mod tcp;

pub trait SendData {
    fn send_data(
        &mut self,
        data: &bytes::BytesMut,
    ) -> impl Future<Output = Result<(), io::Error>> + Send;
}

pub trait RecvData {
    fn recv_data(
        &mut self,
        data: &mut bytes::BytesMut,
    ) -> impl Future<Output = Result<usize, io::Error>> + Send;
}

pub trait SplitTransport {
    type Send: SendData;
    type Recv: RecvData;
}

#[derive(Debug)]
pub struct SendChunk<S> {
    inner: S,
    max_chunk_size: usize,
}

impl<S: SendData> SendChunk<S> {
    pub fn new(inner: S, max_chunk_size: usize) -> Self {
        Self {
            inner,
            max_chunk_size,
        }
    }

    pub fn adjust_max_chunk_size(&mut self, new_max_chunk_size: usize) {
        self.max_chunk_size = new_max_chunk_size;
    }

    pub async fn send_chunk(&mut self, buf: &bytes::BytesMut) -> Result<(), ua::StatusCode> {
        self.check_size(buf)?;
        self.inner
            .send_data(buf)
            .await
            .map_err(|_| ua::StatusCode::BadCommunicationError)
    }

    fn check_size(&self, buf: &bytes::BytesMut) -> Result<(), ua::StatusCode> {
        if buf.remaining() < 8 {
            return Err(ua::StatusCode::BadEncodingError);
        }
        let msg_len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if msg_len != buf.remaining() as u32 {
            return Err(ua::StatusCode::BadEncodingError);
        }
        if msg_len as usize > self.max_chunk_size {
            return Err(ua::StatusCode::BadTcpMessageTooLarge);
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RecvChunk<S> {
    inner: S,
    buf: bytes::BytesMut,
    max_chunk_size: usize,
}

impl<S: RecvData> RecvChunk<S> {
    pub fn new(inner: S, max_chunk_size: usize) -> Self {
        Self {
            inner,
            buf: bytes::BytesMut::new(),
            max_chunk_size,
        }
    }

    pub fn adjust_max_chunk_size(&mut self, new_max_chunk_size: usize) {
        self.max_chunk_size = new_max_chunk_size;
        if self.buf.len() > self.max_chunk_size {
            self.buf.truncate(self.max_chunk_size);
        }
    }

    pub async fn recv_chunk(&mut self, buf: &mut bytes::BytesMut) -> Result<(), ua::StatusCode> {
        debug_assert!(buf.is_empty());
        debug_assert!(buf.capacity() >= self.max_chunk_size);

        self.fill_up_to(buf, 8).await?;

        let msg_len = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        if msg_len as usize > self.max_chunk_size {
            return Err(ua::StatusCode::BadTcpMessageTooLarge);
        }

        self.fill_up_to(buf, msg_len as usize).await?;

        if buf.len() > msg_len as usize {
            self.buf.extend_from_slice(&buf[msg_len as usize..]);
            buf.truncate(msg_len as usize);
        }

        Ok(())
    }

    async fn fill_up_to(
        &mut self,
        buf: &mut bytes::BytesMut,
        n: usize,
    ) -> Result<(), ua::StatusCode> {
        if !self.buf.is_empty() {
            let copied_len = self.buf.len().min(n.saturating_sub(buf.len()));
            buf.extend_from_slice(&self.buf[..copied_len]);
            self.buf.advance(copied_len);
        }

        while buf.len() < n {
            let m = self
                .inner
                .recv_data(buf)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionClosed)?;
            if m == 0 {
                return Err(ua::StatusCode::BadConnectionClosed);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    struct ScriptedRecv(VecDeque<bytes::BytesMut>);

    impl RecvData for ScriptedRecv {
        async fn recv_data(&mut self, buf: &mut bytes::BytesMut) -> Result<usize, io::Error> {
            let data = self.0.pop_front().unwrap_or_default();
            buf.extend_from_slice(&data);
            Ok(data.len())
        }
    }

    fn chunk(body: &[u8]) -> bytes::BytesMut {
        let mut buf = bytes::BytesMut::from(&b"MSGF"[..]);
        buf.extend_from_slice(&(8 + body.len() as u32).to_le_bytes());
        buf.extend_from_slice(body);
        buf
    }

    #[tokio::test]
    async fn receives_coalesced_chunks_without_reusing_buffered_bytes() {
        let chunks = [
            chunk(b"one!"),
            chunk(b"two!"),
            chunk(b"three"),
            chunk(b"four"),
        ];
        let mut combined = bytes::BytesMut::new();
        for chunk in &chunks[..3] {
            combined.extend_from_slice(chunk);
        }
        let mut recv = RecvChunk::new(
            ScriptedRecv(VecDeque::from([combined, chunks[3].clone()])),
            128,
        );
        let mut buf = bytes::BytesMut::with_capacity(128);

        for expected in chunks {
            recv.recv_chunk(&mut buf).await.unwrap();
            assert_eq!(buf, expected);
            buf.clear();
        }
        assert!(recv.buf.is_empty());
    }
}
